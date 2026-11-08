use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct PageParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl PageParams {
    pub fn limit_offset(&self) -> (i64, i64) {
        let size = self.page_size.unwrap_or(25).clamp(1, 100);
        let page = self.page.unwrap_or(1).max(1);
        (size, (page - 1) * size)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Paged<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
