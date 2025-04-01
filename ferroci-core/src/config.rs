use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Job {
    pub steps: Vec<String>,
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pipeline {
    #[allow(dead_code)]
    pub version: String,
    pub jobs: std::collections::HashMap<String, Job>,
}

impl Pipeline {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let pipeline: Pipeline = toml::from_str(&content)?;
        Ok(pipeline)
    }
}
