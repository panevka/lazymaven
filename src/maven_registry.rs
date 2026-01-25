use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

pub struct MavenRegistry {}

static API_URL: &str = "https://search.maven.org";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct SearchResponse {
    num_found: u32,
    start: u32,
    pub docs: Vec<SearchResponseDoc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct SearchResponseDoc {
    pub id: String,
    g: String,
    a: String,
    repository_id: String,
    p: String,
    timestamp: u64,
    version_count: u32,
    text: Vec<String>,
    ec: Vec<String>,
    latest_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MavenResponse {
    pub response: SearchResponse,
}

impl MavenRegistry {
    fn api_search_preview_url(query: String, rows: u32) -> String {
        format!(
            "{}/solrsearch/select?q={}&rows={}&wt=json",
            API_URL, query, rows
        )
    }

    pub async fn search_dependencies(search_phrase: String) -> Result<MavenResponse, Error> {
        let client = reqwest::Client::new();

        let request_url = MavenRegistry::api_search_preview_url(search_phrase, 20);

        let response = client
            .get(request_url)
            .header("User-Agent", "LazyMaven")
            .send()
            .await?
            .json::<MavenResponse>()
            .await?;

        return Ok(response);
    }
}
