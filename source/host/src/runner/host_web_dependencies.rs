use crate::extensions::inference::cursor_web_tools::{
    CursorWebFetchService, CursorWebSearchService,
};
use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::runner::tools::sand_web_tools::{
    WebToolExecutor, render_web_fetch_result, render_web_search_result,
};
use serde_json::Value;

pub struct ProductionWebToolExecutor {
    search: CursorWebSearchService,
    fetch: CursorWebFetchService,
}

impl ProductionWebToolExecutor {
    pub fn new(search: CursorWebSearchService, fetch: CursorWebFetchService) -> Self {
        Self { search, fetch }
    }
}

impl WebToolExecutor for ProductionWebToolExecutor {
    fn search(
        &self,
        search_term: &str,
        explanation: Option<&str>,
    ) -> Result<Value, ProviderSessionError> {
        let response = self
            .search
            .search(search_term, explanation.map(str::to_string))
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        let documents = response
            .documents
            .iter()
            .map(|document| (
                document.title.as_str(),
                document.url.as_str(),
                document.text.as_str(),
            ))
            .collect::<Vec<_>>();
        Ok(render_web_search_result(
            response.answer.as_deref(),
            &documents,
        ))
    }

    fn fetch(&self, url: &str) -> Result<Value, ProviderSessionError> {
        let response = self
            .fetch
            .fetch(url)
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        Ok(render_web_fetch_result(
            url,
            response.content.as_deref(),
            response.error.as_deref(),
            response.is_timeout,
        ))
    }
}
