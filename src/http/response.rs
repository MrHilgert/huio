use std::collections::HashMap;

pub struct Response {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn status_code(&self) -> u16 {
        self.status
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn into_parts(self) -> (u16, HashMap<String, String>, Vec<u8>) {
        (self.status, self.headers, self.body)
    }

    fn build(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());
        Self {
            status,
            headers,
            body,
        }
    }

    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self::build(200, "text/plain; charset=utf-8", body.into())
    }

    pub fn json(body: impl serde::Serialize) -> Self {
        match serde_json::to_vec(&body) {
            Ok(json) => {
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers.insert("Content-Length".to_string(), json.len().to_string());
                Self {
                    status: 200,
                    headers,
                    body: json,
                }
            }
            Err(_) => Self::internal_error(),
        }
    }

    pub fn not_found() -> Self {
        Self::build(404, "text/plain; charset=utf-8", b"Not Found".to_vec())
    }

    pub fn method_not_allowed(allowed: &[&str]) -> Self {
        let mut res = Self::build(
            405,
            "text/plain; charset=utf-8",
            b"Method Not Allowed".to_vec(),
        );
        res.headers.insert("Allow".to_string(), allowed.join(", "));
        res
    }

    pub fn internal_error() -> Self {
        Self::build(
            500,
            "text/plain; charset=utf-8",
            b"Internal Server Error".to_vec(),
        )
    }

    #[must_use]
    pub fn status(mut self, status: u16) -> Result<Self, &'static str> {
        if (100..=999).contains(&status) {
            self.status = status;
            Ok(self)
        } else {
            Err("Invalid HTTP status code: must be between 100 and 999")
        }
    }

    #[must_use]
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub(crate) fn clear_body(&mut self) {
        self.body = vec![];
        self.headers
            .insert("Content-Length".to_string(), "0".to_string());
    }
}
