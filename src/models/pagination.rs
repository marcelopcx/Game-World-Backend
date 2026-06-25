use serde::Serialize;

#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub page: i64,
    pub limit: i64,
}

impl PaginationParams {
    pub fn from_query(page: Option<i64>, limit: Option<i64>) -> Self {
        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(20).clamp(1, 100);
        Self { page, limit }
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}
