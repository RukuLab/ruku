use bollard::container::ListContainersOptions;
use bollard::Docker;
use futures::executor::block_on;
use port_selector::is_free;
use serde::Deserialize;
use std::collections::HashMap;
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
    // First check if the port is being used by our own container
    if let Ok(docker) = Docker::connect_with_local_defaults() {
        let mut filters = HashMap::new();
        filters.insert("publish".to_string(), vec![port.to_string()]);

        let options = Some(ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        });

        if let Ok(containers) = block_on(docker.list_containers(options)) {
            // If we found containers using this port, it's okay - we'll handle it in the Container::run
            if !containers.is_empty() {
                return Ok(());
            }
        }
    }

    // If we get here, check if the port is free for other applications
    if !is_free(port) {
        return Err(ValidationError::new("port is already in use by another application"));
    }
    Ok(())
}
