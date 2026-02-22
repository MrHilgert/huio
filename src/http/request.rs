use std::collections::HashMap;

use crate::http::HttpMethod;

pub struct Request {
    pub method: HttpMethod,
    pub(crate) path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub(crate) params: HashMap<String, String>,
    pub(crate) query: HashMap<String, String>,
}

impl Request {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn query_all(&self) -> &HashMap<String, String> {
        &self.query
    }
}
