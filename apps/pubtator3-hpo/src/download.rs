use crate::{Dataset, Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFile {
    pub name: String,
    pub url: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataManifest {
    pub hpo_release: String,
    pub mondo_release: String,
    pub files: Vec<DataFile>,
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
    #[serde(default)]
    digest: Option<String>,
}

/// Download and validate current official releases into an empty directory.
/// Files are staged; the manifest is installed last. Existing snapshots are never overwritten.
/// The manifest pins release URLs and checksums for reproducibility.
pub async fn download_data(directory: impl AsRef<Path>) -> Result<DataManifest> {
    let directory = directory.as_ref();
    tokio::fs::create_dir_all(directory).await?;
    if tokio::fs::read_dir(directory)
        .await?
        .next_entry()
        .await?
        .is_some()
    {
        return Err(Error::InvalidRequest(
            "download directory must be empty; load existing snapshots with Dataset::from_dir"
                .into(),
        ));
    }
    let http = reqwest::Client::builder()
        .user_agent("pubtator3-hpo/0.1.0")
        .timeout(Duration::from_secs(600))
        .build()?;
    let hpo: Release = http
        .get("https://api.github.com/repos/obophenotype/human-phenotype-ontology/releases/latest")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let mondo: Release = http
        .get("https://api.github.com/repos/monarch-initiative/mondo/releases/latest")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let stage = tempfile::tempdir_in(directory)?;
    let mut files = Vec::new();
    for (release, name) in [
        (&hpo, "hp.obo"),
        (&hpo, "phenotype.hpoa"),
        (&mondo, "mondo.json"),
    ] {
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| Error::Data(format!("release {} lacks {name}", release.tag_name)))?;
        let mut response = http
            .get(&asset.browser_download_url)
            .send()
            .await?
            .error_for_status()?;
        let mut file = tokio::fs::File::create(stage.path().join(name)).await?;
        let mut hasher = Sha256::new();
        let mut bytes = 0_u64;
        while let Some(chunk) = response.chunk().await? {
            bytes += chunk.len() as u64;
            if bytes > asset.size {
                return Err(Error::Data(format!("{name} exceeds release asset size")));
            }
            hasher.update(&chunk);
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        drop(file);
        let sha256 = format!("{:x}", hasher.finalize());
        if bytes != asset.size {
            return Err(Error::Data(format!("truncated download of {name}")));
        }
        if let Some(expected) = asset
            .digest
            .as_deref()
            .and_then(|d| d.strip_prefix("sha256:"))
            && expected != sha256
        {
            return Err(Error::Data(format!("checksum mismatch for {name}")));
        }
        files.push(DataFile {
            name: name.into(),
            url: asset.browser_download_url.clone(),
            bytes,
            sha256,
        });
    }
    let snapshot = stage.path().to_owned();
    tokio::task::spawn_blocking(move || Dataset::from_dir(snapshot)).await??;
    let manifest = DataManifest {
        hpo_release: hpo.tag_name,
        mondo_release: mondo.tag_name,
        files,
    };
    tokio::fs::write(
        stage.path().join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .await?;
    for file in &manifest.files {
        tokio::fs::rename(stage.path().join(&file.name), directory.join(&file.name)).await?;
    }
    tokio::fs::rename(
        stage.path().join("manifest.json"),
        directory.join("manifest.json"),
    )
    .await?;
    Ok(manifest)
}
