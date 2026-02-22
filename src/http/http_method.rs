use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn from(method: &str) -> Option<Self> {
        if method.eq_ignore_ascii_case("GET") {
            Some(HttpMethod::GET)
        } else if method.eq_ignore_ascii_case("POST") {
            Some(HttpMethod::POST)
        } else if method.eq_ignore_ascii_case("PUT") {
            Some(HttpMethod::PUT)
        } else if method.eq_ignore_ascii_case("DELETE") {
            Some(HttpMethod::DELETE)
        } else if method.eq_ignore_ascii_case("PATCH") {
            Some(HttpMethod::PATCH)
        } else if method.eq_ignore_ascii_case("HEAD") {
            Some(HttpMethod::HEAD)
        } else if method.eq_ignore_ascii_case("OPTIONS") {
            Some(HttpMethod::OPTIONS)
        } else {
            None
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}
