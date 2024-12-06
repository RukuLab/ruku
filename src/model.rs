use port_selector::is_free;
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct RukuConfig {
    #[validate(range(min = 1024, max = 65535), custom(function = "validate_port"))]
    pub port: u16,
    #[validate(length(min = 1, max = 20))]
    pub version: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub install_cmd: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub build_cmd: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub start_cmd: Option<String>,
    #[validate(length(min = 1, max = 50), custom(function = "validate_providers"))]
    pub providers: Option<Vec<String>>,
}

const ALLOWED_PROVIDERS: [&str; 25] = [
    "...",
    "clojure",
    "cobol",
    "crystal",
    "csharp",
    "dart",
    "deno",
    "elixir",
    "fsharp",
    "gleam",
    "go",
    "haskell",
    "java",
    "lunatic",
    "node",
    "php",
    "procfile",
    "python",
    "ruby",
    "rust",
    "scala",
    "scheme",
    "staticfile",
    "swift",
    "zig",
];

fn validate_providers(providers: &[String]) -> Result<(), ValidationError> {
    for provider in providers {
        if !ALLOWED_PROVIDERS.contains(&provider.as_str()) {
            return Err(ValidationError::new("invalid provider"));
        }
    }
    Ok(())
}

fn validate_port(port: u16) -> Result<(), ValidationError> {
    if !is_free(port) {
        return Err(ValidationError::new("port is already in use"));
    }
    Ok(())
}
