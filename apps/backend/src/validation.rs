use crate::error::{AppError, Result};
use serde::Deserialize;

pub fn text(value: &str, max: usize) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max || value.contains('\0') {
        return Err(AppError::bad_request(
            "text is empty, too long, or contains invalid characters",
        ));
    }
    Ok(value.to_owned())
}

pub fn slug(value: &str) -> Result<String> {
    let value = value.trim().to_ascii_lowercase();
    if !(3..=80).contains(&value.len())
        || value.split('-').any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
    {
        return Err(AppError::bad_request(
            "slug must be 3-80 lowercase letters or digits separated by single hyphens",
        ));
    }
    Ok(value)
}

#[derive(Default, Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Pagination {
    pub fn bounds(&self) -> Result<(i64, i64)> {
        let limit = self.limit.unwrap_or(50);
        let offset = self.offset.unwrap_or(0);
        if !(1..=100).contains(&limit) || !(0..=10000).contains(&offset) {
            return Err(AppError::bad_request(
                "limit must be 1-100 and offset 0-10000",
            ));
        }
        Ok((limit, offset))
    }
}
