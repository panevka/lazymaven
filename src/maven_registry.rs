use anyhow::Result;
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct SearchResponseDoc {
    pub id: String,
    pub g: String,
    pub a: String,
    repository_id: String,
    p: String,
    timestamp: u64,
    version_count: u32,
    text: Vec<String>,
    ec: Vec<String>,
    latest_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct GetVersionsResponse {
    num_found: u32,
    start: u32,
    pub docs: Vec<GetVersionsResponseDoc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct GetVersionsResponseDoc {
    pub id: String,
    pub g: String,
    pub a: String,
    pub v: String,
    pub p: String,
    pub timestamp: u64,
    pub ec: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct MavenResponse<T> {
    pub response: T,
}

impl MavenRegistry {
    fn api_search_preview_url(query: String, rows: u32) -> String {
        format!(
            "{}/solrsearch/select?q={}&rows={}&wt=json",
            API_URL, query, rows
        )
    }

    pub async fn search_dependencies(search_phrase: String) -> Result<MavenResponse<SearchResponse>> {
        let client = reqwest::Client::new();

        let request_url = MavenRegistry::api_search_preview_url(search_phrase, 20);

        let response = client
            .get(request_url)
            .header("User-Agent", "LazyMaven")
            .send()
            .await?
            .json::<MavenResponse<SearchResponse>>()
            .await?;

        return Ok(response);
    }

    pub async fn get_available_dependency_versions(group_id: String, artifact_id: String) -> Result<MavenResponse<GetVersionsResponse>> {
        let client = reqwest::Client::new();

        let request_url = format!(
            "{}/solrsearch/select?q=g:{}+AND+a:{}&core=gav&rows={}&wt=json",
            API_URL, group_id, artifact_id, 20
        );

        let response = client
            .get(request_url)
            .header("User-Agent", "LazyMaven")
            .send()
            .await?
            .json::<MavenResponse<GetVersionsResponse>>()
            .await?;

        return Ok(response);
    }

}
