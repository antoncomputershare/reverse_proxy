use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub listen: String,
    pub control: ControlConfig,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControlConfig {
    pub listen: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Route {
    pub name: String,
    pub hosts: Vec<String>,
    pub path_prefix: String,
    #[serde(default)]
    pub strip_prefix: bool,
    #[serde(default)]
    pub rewrite_prefix: Option<String>,
    pub upstreams: Vec<Upstream>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Upstream {
    pub url: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default = "default_fail_threshold")]
    pub fail_threshold: u32,
    #[serde(default = "default_cooldown_secs")]
    pub cooldown_secs: u64,
}

fn default_weight() -> u32 {
    1
}

fn default_fail_threshold() -> u32 {
    3
}

fn default_cooldown_secs() -> u64 {
    15
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
            listen = "127.0.0.1:8080"

            [control]
            listen = "127.0.0.1:9000"

            [[routes]]
            name = "echo"
            hosts = ["example.com"]
            path_prefix = "/"
            strip_prefix = true
            rewrite_prefix = "/"

            [[routes.upstreams]]
            url = "http://httpbin.org"
            weight = 2
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.listen, "127.0.0.1:8080");
        assert_eq!(config.control.listen, "127.0.0.1:9000");
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].upstreams[0].weight, 2);
    }
}
