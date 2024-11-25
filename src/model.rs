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
}

fn validate_port(port: u16) -> Result<(), ValidationError> {
    if !is_free(port) {
        return Err(ValidationError::new("port is already in use"));
    }
    Ok(())
}
