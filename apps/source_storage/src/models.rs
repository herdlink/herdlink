use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

macro_rules! text_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $text)] $variant),+
        }

        impl $name {
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = crate::Error;

            fn from_str(value: &str) -> crate::Result<Self> {
                match value {
                    $($text => Ok(Self::$variant)),+,
                    _ => Err(crate::Error::InvalidValue {
                        field: stringify!($name),
                        value: value.to_owned(),
                    }),
                }
            }
        }
    };
}

text_enum!(FileFormat {
    Pdf => "pdf",
    Xml => "xml",
    Json => "json",
    Html => "html",
});

text_enum!(Category {
    Research => "research",
    Preprint => "preprint",
    ClinicalStudy => "clinical_study",
    Funding => "funding",
    Reference => "reference",
    ModelResource => "model_resource",
    Organization => "organization",
    Experience => "experience",
    News => "news",
    Guideline => "guideline",
});

text_enum!(RecordStatus {
    Active => "active",
    Retracted => "retracted",
    Withdrawn => "withdrawn",
    Terminated => "terminated",
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageIdentity {
    pub id: Uuid,
    pub storage_uri: String,
    pub file_format: FileFormat,
    /// Lowercase hexadecimal SHA-256 digest of the raw file bytes.
    pub content_hash: String,
    pub lastfetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    pub category: Category,
    pub subtype: String,
    pub is_peer_reviewed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub source_name: String,
    pub external_id: String,
    pub doi: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub orcid: Option<String>,
    #[serde(default)]
    pub affiliations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub title: String,
    pub summary: Option<String>,
    pub language: String,
    pub authors: Option<Vec<Author>>,
    pub published_at: Option<NaiveDate>,
    pub source_updated_at: Option<DateTime<Utc>>,
    pub record_status: RecordStatus,
    /// Category-specific fields stored as a JSON object.
    pub metadata: Value,
}

/// One database row. Serde uses the same flat field names as the SQL schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    #[serde(flatten)]
    pub storage: StorageIdentity,
    #[serde(flatten)]
    pub classification: Classification,
    #[serde(flatten)]
    pub source: Source,
    #[serde(flatten)]
    pub content: Content,
}

/// Input for inserting a file and its metadata together. Storage fields are generated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewRecord {
    pub file_format: FileFormat,
    pub classification: Classification,
    pub source: Source,
    pub content: Content,
}
