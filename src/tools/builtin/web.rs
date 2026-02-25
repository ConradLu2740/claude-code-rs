use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

pub struct WebSearchTool {
    client: Client,
}

impl WebSearchTool {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client }
    }

    async fn search_searxng(&self, query: &str) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://searx.be/search?q={}&format=json",
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send search request to SearXNG")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Search API returned status: {}", response.status()));
        }

        let search_response: SearXNGResponse = response
            .json()
            .await
            .context("Failed to parse search response")?;

        Ok(search_response.results.into_iter().take(5).map(|r| SearchResult {
            title: r.title,
            url: r.url,
            snippet: r.content,
        }).collect())
    }

    async fn search_duckduckgo(&self, query: &str) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send search request to DuckDuckGo")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("DuckDuckGo API returned status: {}", response.status()));
        }

        let ddg_response: DDGResponse = response
            .json()
            .await
            .unwrap_or_default();

        let mut results = Vec::new();

        if let Some(abstract_text) = ddg_response.abstract_text {
            if !abstract_text.is_empty() {
                results.push(SearchResult {
                    title: ddg_response.heading.unwrap_or_default(),
                    url: ddg_response.abstract_url.unwrap_or_default(),
                    snippet: abstract_text,
                });
            }
        }

        for topic in ddg_response.related_topics.iter().take(5) {
            if let Some(text) = &topic.text {
                if !text.is_empty() {
                    results.push(SearchResult {
                        title: topic.first_url.clone().unwrap_or_default(),
                        url: topic.first_url.clone().unwrap_or_default(),
                        snippet: text.clone(),
                    });
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct DDGResponse {
    #[serde(default)]
    abstract_text: Option<String>,
    #[serde(default)]
    abstract_url: Option<String>,
    #[serde(default)]
    heading: Option<String>,
    #[serde(default)]
    related_topics: Vec<RelatedTopic>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RelatedTopic {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    first_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SearXNGResponse {
    results: Vec<SearXNGResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct SearXNGResult {
    title: String,
    url: String,
    content: String,
}

#[async_trait]
impl ToolExecutor for WebSearchTool {
    async fn execute(&self, input: Value, _ctx: &ExecutionContext) -> Result<ToolResult> {
        let query = input.get("query")
            .and_then(|v| v.as_str())
            .context("Missing 'query' parameter")?;

        let results = match self.search_searxng(query).await {
            Ok(r) if !r.is_empty() => r,
            _ => {
                self.search_duckduckgo(query).await.unwrap_or_default()
            }
        };

        if results.is_empty() {
            return Ok(ToolResult::success(format!(
                "No results found for: {}. Note: Web search may be limited in some regions. Try using web_fetch to get content from a specific URL.",
                query
            )));
        }

        let mut output = format!("Search results for '{}':\n\n", query);
        
        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}\n   {}\n   URL: {}\n\n",
                i + 1,
                result.title,
                result.snippet,
                result.url
            ));
        }

        Ok(ToolResult::success(output))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "web_search",
            "Search the web for information. Returns search results with titles, snippets, and URLs. Uses multiple search engines for better results.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["query"]
            }),
        )
    }
}

pub struct WebFetchTool {
    client: Client,
}

impl WebFetchTool {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client }
    }
}

#[async_trait]
impl ToolExecutor for WebFetchTool {
    async fn execute(&self, input: Value, _ctx: &ExecutionContext) -> Result<ToolResult> {
        let url = input.get("url")
            .and_then(|v| v.as_str())
            .context("Missing 'url' parameter")?;

        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch URL. The website may be blocking requests or unavailable.")?;

        if !response.status().is_success() {
            return Ok(ToolResult::error(format!(
                "HTTP Error: {} - The website returned an error",
                response.status()
            )));
        }

        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.contains("text/html") {
            let html = response.text().await.context("Failed to read HTML")?;
            
            let text = extract_text_from_html(&html);
            
            let truncated = if text.len() > 8000 {
                format!("{}...\n\n[Content truncated, {} characters total]", 
                    &text[..8000], text.len())
            } else {
                text
            };

            Ok(ToolResult::success(format!(
                "Content from {}:\n\n{}",
                url, truncated
            )))
        } else if content_type.contains("application/json") {
            let json = response.text().await.context("Failed to read JSON")?;
            Ok(ToolResult::success(format!(
                "JSON content from {}:\n\n{}",
                url, json
            )))
        } else {
            let text = response.text().await.context("Failed to read response")?;
            
            let truncated = if text.len() > 8000 {
                format!("{}...\n\n[Content truncated]", &text[..8000])
            } else {
                text
            };

            Ok(ToolResult::success(format!(
                "Content from {}:\n\n{}",
                url, truncated
            )))
        }
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "web_fetch",
            "Fetch content from a URL. Returns the text content of the webpage. Useful for getting detailed content from a specific URL.",
            json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch (must be a valid URL with http:// or https://)"
                    }
                },
                "required": ["url"]
            }),
        )
    }
}

fn extract_text_from_html(html: &str) -> String {
    let mut text = String::new();
    let mut in_script = false;
    let mut in_style = false;
    let mut in_comment = false;
    
    for line in html.lines() {
        let line_lower = line.to_lowercase();
        
        if line_lower.contains("<script") || line_lower.contains("<script>") {
            in_script = true;
        }
        if line_lower.contains("</script>") {
            in_script = false;
            continue;
        }
        if line_lower.contains("<style") || line_lower.contains("<style>") {
            in_style = true;
        }
        if line_lower.contains("</style>") {
            in_style = false;
            continue;
        }
        if line_lower.contains("<!--") {
            in_comment = true;
        }
        if line_lower.contains("-->") {
            in_comment = false;
            continue;
        }
        
        if in_script || in_style || in_comment {
            continue;
        }
        
        let mut cleaned = line.to_string();
        
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        cleaned = re.replace_all(&cleaned, " ").to_string();
        
        cleaned = cleaned
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");
        
        let cleaned: String = cleaned
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        if !cleaned.is_empty() && cleaned.len() > 3 {
            text.push_str(&cleaned);
            text.push('\n');
        }
    }
    
    text.trim().to_string()
}
