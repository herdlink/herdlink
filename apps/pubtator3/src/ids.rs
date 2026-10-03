use std::{fmt, num::NonZeroU64, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Error, Result};

fn invalid(kind: &'static str, value: impl ToString) -> Error {
    Error::InvalidId {
        kind,
        value: value.to_string(),
    }
}

/// Positive PubMed identifier, distinct from a PMC full-text identifier.
/// JSON accepts an integer or decimal string and serializes as an integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Pmid(NonZeroU64);

impl Pmid {
    pub fn new(value: u64) -> Result<Self> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or_else(|| invalid("PMID", value))
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl TryFrom<u64> for Pmid {
    type Error = Error;
    fn try_from(value: u64) -> Result<Self> {
        Self::new(value)
    }
}

impl FromStr for Pmid {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid("PMID", value));
        }
        Self::new(value.parse().map_err(|_| invalid("PMID", value))?)
    }
}

impl fmt::Display for Pmid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for Pmid {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Number(u64),
            Text(String),
        }
        match Wire::deserialize(d)? {
            Wire::Number(n) => Self::new(n),
            Wire::Text(s) => s.parse(),
        }
        .map_err(serde::de::Error::custom)
    }
}

macro_rules! string_id {
    ($name:ident, $description:literal, $validate:expr) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                if ($validate)(&value) {
                    Ok(Self(value))
                } else {
                    Err(invalid(stringify!($name), value))
                }
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl FromStr for $name {
            type Err = Error;
            fn from_str(value: &str) -> Result<Self> {
                Self::new(value)
            }
        }
        impl TryFrom<String> for $name {
            type Error = Error;
            fn try_from(value: String) -> Result<Self> {
                Self::new(value)
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

string_id!(
    Pmcid,
    "Canonical PMC identifier, e.g. `PMC6142073`.",
    |s: &str| {
        s.strip_prefix("PMC")
            .is_some_and(|n| n.parse::<Pmid>().is_ok())
    }
);

string_id!(
    EntityId,
    "PubTator search identifier, e.g. `@GENE_BRAF` (not an NCBI Gene ID).",
    |s: &str| {
        s.strip_prefix('@')
            .and_then(|s| s.split_once('_'))
            .is_some_and(|(kind, name)| {
                !kind.is_empty()
                    && kind.bytes().all(|b| b.is_ascii_uppercase())
                    && !name.is_empty()
                    && !name
                        .chars()
                        .any(|c| c.is_whitespace() || c.is_control() || "|\"".contains(c))
            })
    }
);

impl EntityId {
    /// Namespace from the identifier itself, such as `GENE` or `CHEMICAL`.
    pub fn namespace(&self) -> &str {
        self.0[1..].split_once('_').expect("validated entity ID").0
    }
}

string_id!(
    AnnotationId,
    "BioC annotation identifier, scoped to a document.",
    |s: &str| { !s.trim().is_empty() && !s.chars().any(char::is_control) }
);
string_id!(
    RelationId,
    "BioC relation identifier, scoped to a document.",
    |s: &str| { !s.trim().is_empty() && !s.chars().any(char::is_control) }
);
string_id!(
    DocumentId,
    "BioC document ID; full-text IDs are not necessarily PMIDs.",
    |s: &str| { !s.trim().is_empty() && !s.chars().any(char::is_control) }
);

fn mesh_id(value: &str, prefix: char) -> bool {
    value.strip_prefix(prefix).is_some_and(|digits| {
        matches!(digits.len(), 6 | 9)
            && digits.bytes().all(|b| b.is_ascii_digit())
            && digits.bytes().any(|b| b != b'0')
    })
}

string_id!(
    MeshDescriptorId,
    "MeSH descriptor ID, e.g. `D000690` or `D000086382`.",
    |s: &str| mesh_id(s, 'D')
);
string_id!(
    MeshTermId,
    "MeSH term ID, e.g. `T002090` or `T000953133`.",
    |s: &str| mesh_id(s, 'T')
);

macro_rules! mesh_uri {
    ($name:ident, $module:ident) => {
        impl $name {
            /// Parse an unversioned MeSH resource URI from the NLM API.
            pub fn from_uri(uri: &str) -> Result<Self> {
                let id = uri
                    .strip_prefix("http://id.nlm.nih.gov/mesh/")
                    .or_else(|| uri.strip_prefix("https://id.nlm.nih.gov/mesh/"))
                    .ok_or_else(|| invalid(stringify!($name), uri))?;
                Self::new(id)
            }

            pub fn to_uri(&self) -> String {
                format!("http://id.nlm.nih.gov/mesh/{self}")
            }
        }

        pub(crate) mod $module {
            use super::$name;
            use serde::{Deserialize, Deserializer, Serializer};

            pub fn serialize<S: Serializer>(
                id: &$name,
                s: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                s.serialize_str(&id.to_uri())
            }

            pub fn deserialize<'de, D: Deserializer<'de>>(
                d: D,
            ) -> std::result::Result<$name, D::Error> {
                $name::from_uri(&String::deserialize(d)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

mesh_uri!(MeshDescriptorId, mesh_descriptor_uri);
mesh_uri!(MeshTermId, mesh_term_uri);
