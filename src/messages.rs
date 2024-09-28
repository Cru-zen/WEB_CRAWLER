use url::Url;
use reqwest::Response as ReqwestResponse;
use std::error::Error;

/// Represents the raw HTML response downloaded from a URL.
#[derive(Debug)]
pub struct Response {
    pub source: Url,
    pub response: ReqwestResponse,
}

impl Response {
    /// Creates a new `Response` object with a source URL and the corresponding `reqwest::Response`.
    pub fn new(source: Url, response: ReqwestResponse) -> Self {
        Self { source, response }
    }

    /// Optional future function to retrieve the body as text.
    pub async fn body_text(self) -> Result<String, Box<dyn Error>> {
        let body = self.response.text().await?;
        Ok(body)
    }
}

/// Represents a request for a URL to be downloaded next.
#[derive(Debug)]
pub struct Request {
    pub url: Url,
}

impl Request {
    /// Creates a new `Request` object with the provided URL.
    ///
    /// This can be useful for handling the next URL to be crawled.
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    /// Try to create a `Request` from a URL string, with error handling for invalid URLs.
    pub fn from_str(url_str: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(url_str)?;
        Ok(Self { url })
    }
}
