use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

fn numeric(s: &str, prefix: &str, width: Option<usize>) -> bool {
    s.strip_prefix(prefix).is_some_and(|n| {
        !n.is_empty()
            && width.is_none_or(|w| n.len() == w)
            && n.bytes().all(|b| b.is_ascii_digit())
            && n.parse::<u32>().is_ok_and(|n| n > 0)
    })
}

macro_rules! id {
    ($name:ident, $validate:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);
        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl FromStr for $name {
            type Err = Error;
            fn from_str(s: &str) -> Result<Self> {
                if ($validate)(s) {
                    Ok(Self(s.into()))
                } else {
                    Err(Error::InvalidId(s.into()))
                }
            }
        }
        impl TryFrom<String> for $name {
            type Error = Error;
            fn try_from(s: String) -> Result<Self> {
                s.parse()
            }
        }
        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.0
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

// Includes both descriptors (D...) and supplementary concepts (C...).
id!(MeshId, |s: &str| ["D", "C"].iter().any(|p| {
    numeric(s, p, Some(6)) || numeric(s, p, Some(9))
}));
id!(MondoId, |s: &str| numeric(s, "MONDO:", Some(7)));
id!(HpoId, |s: &str| numeric(s, "HP:", Some(7)));

impl HpoId {
    pub fn number(&self) -> u32 {
        self.0[3..].parse().expect("validated HPO ID")
    }
    pub(crate) fn from_number(n: u32) -> Self {
        Self(format!("HP:{n:07}"))
    }
}

/// Identifier of an HPO disease annotation record; HP IDs identify features instead.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DiseaseId(String);
impl DiseaseId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for DiseaseId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let s = if let Some(n) = s.strip_prefix("MIM:") {
            format!("OMIM:{n}")
        } else if let Some(n) = s.strip_prefix("Orphanet:") {
            format!("ORPHA:{n}")
        } else {
            s.into()
        };
        let valid = numeric(&s, "OMIM:", Some(6))
            || numeric(&s, "ORPHA:", None)
            || numeric(&s, "DECIPHER:", None)
            || s.parse::<MondoId>().is_ok();
        if valid {
            Ok(Self(s))
        } else {
            Err(Error::InvalidId(s))
        }
    }
}
impl TryFrom<String> for DiseaseId {
    type Error = Error;
    fn try_from(s: String) -> Result<Self> {
        s.parse()
    }
}
impl From<DiseaseId> for String {
    fn from(id: DiseaseId) -> String {
        id.0
    }
}
impl fmt::Display for DiseaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
