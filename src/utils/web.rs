use std::process::Command;

use indexmap::IndexMap;

use crate::utils::logger::{LogLevel, Logger};

lazy_static::lazy_static! {
    static ref LOG: Logger = Logger::new("ui", LogLevel::Debug);
}

#[derive(Debug, Clone)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
    pub search_engine: String,
}

pub struct WebSearchManager {
    search_engines: IndexMap<String, String>,
}

impl WebSearchManager {
    pub fn new() -> Self {
        let mut search_engines = IndexMap::new();
        
        search_engines.insert("google".to_string(), "https://www.google.com/search?q={}".to_string());
        search_engines.insert("duckduckgo".to_string(), "https://duckduckgo.com/?q={}".to_string());
        search_engines.insert("youtube".to_string(), "https://www.youtube.com/results?search_query={}".to_string());
        search_engines.insert("stackoverflow".to_string(), "https://stackoverflow.com/search?q={}".to_string());
        
        Self { search_engines }
    }
    
    pub fn get_search_engines(&self) -> Vec<String> {
        self.search_engines.keys().cloned().collect()
    }
    
    pub fn search_engines_for_query(&self, query: &str) -> Vec<WebSearchResult> {
        let mut results = Vec::new();
        
        for (engine_name, url_template) in &self.search_engines {
                let description = match engine_name.as_str() {
                    "google" => "Search with Google",
                    "duckduckgo" => "Search with DuckDuckGo (Privacy-focused)",
                    "youtube" => "Search YouTube videos",
                    "stackoverflow" => "Search Stack Overflow",
                    _ => "Web search",
                };
                
                results.push(WebSearchResult {
                    title: format!("Search \"{}\" on {}", query, engine_name),
                    url: url_template.replace("{}", &urlencoding::encode(query)),
                    description: description.to_string(),
                    search_engine: engine_name.clone(),
                });
        }
        
        results
    }
    
    pub fn open_url(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
            Command::new("xdg-open").arg(url).spawn()?;
        Ok(())
    }
}
