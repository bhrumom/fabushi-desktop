use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDocument {
    pub url: String,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSearchRequest {
    pub search_term: String,
    pub explanation: Option<String>,
    pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSearchResponse {
    pub answer: String,
    pub documents: Vec<WebDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebFetchResponse {
    Success { content: String },
    Error { error: String, is_timeout: Option<bool> },
    MissingResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebFetchResult {
    pub content: Option<String>,
    pub error: Option<String>,
    pub is_timeout: Option<bool>,
}

pub trait CursorWebBackend: Send + Sync {
    fn run_web_search(&self, request: WebSearchRequest) -> Result<WebSearchResponse, String>;
    fn run_web_fetch(&self, url: &str) -> Result<WebFetchResponse, String>;
}

#[derive(Clone)]
pub struct CursorWebSearchService {
    backend: Arc<dyn CursorWebBackend>,
    model_id: String,
}

impl CursorWebSearchService {
    pub fn new(backend: Arc<dyn CursorWebBackend>, model_id: impl Into<String>) -> Self {
        Self {
            backend,
            model_id: model_id.into(),
        }
    }

    pub fn search(
        &self,
        search_term: impl Into<String>,
        explanation: Option<String>,
    ) -> Result<WebSearchResponse, String> {
        self.backend.run_web_search(WebSearchRequest {
            search_term: search_term.into(),
            explanation,
            model_id: self.model_id.clone(),
        })
    }
}

#[derive(Clone)]
pub struct CursorWebFetchService {
    backend: Arc<dyn CursorWebBackend>,
}

impl CursorWebFetchService {
    pub fn new(backend: Arc<dyn CursorWebBackend>) -> Self {
        Self { backend }
    }

    pub fn fetch(&self, url: &str) -> Result<WebFetchResult, String> {
        match self.backend.run_web_fetch(url)? {
            WebFetchResponse::Success { content } => Ok(WebFetchResult {
                content: Some(content),
                error: None,
                is_timeout: None,
            }),
            WebFetchResponse::Error { error, is_timeout } => Ok(WebFetchResult {
                content: None,
                error: Some(error),
                is_timeout,
            }),
            WebFetchResponse::MissingResult => Ok(WebFetchResult {
                content: None,
                error: Some("Web fetch returned no result.".to_string()),
                is_timeout: None,
            }),
        }
    }
}
