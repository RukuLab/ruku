use std::fs;

use bollard::Docker;
use clap::{Parser, Subcommand};
use validator::Validate;

use logger::Logger;
use server_config::ServerConfig;

use crate::container::Container;
use crate::deploy::Deploy;
use crate::git::Git;
use crate::misc::sanitize_app_name;
use crate::model::RukuConfig;

mod container;
mod deploy;
mod git;
mod logger;
mod misc;
mod model;
mod server_config;

#[derive(Parser)]
#[command(version, about = "A CLI app for managing your server.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Enum representing the various commands that can be executed by the CLI.
#[derive(Subcommand)]
enum Command {
    /// Show logs
    Logs,
    /// Set a configuration variable, e.g, VAR=12
    #[command(name = "config:set")]
    ConfigSet {
        /// The configuration variable in the form KEY=VALUE
        var: String,
    },
    /// Get a configuration variable
    #[command(name = "config:get")]
    ConfigGet {
        /// The configuration variable name
        key: String,
    },
    /// Run the application
    Run,
    /// Deploy the application
    Deploy,
    /// Stop the application
    Stop,
    /// Destroy the application
    Destroy,
    /// Git hook
    #[command(name = "git-hook")]
    GitHook {
        /// The git repository name
        repo: String,
    },
    /// Git receive pack
    #[command(name = "git-receive-pack")]
    GitReceivePack {
        /// The git repository name
        repo: String,
    },
    /// Git upload pack
    #[command(name = "git-upload-pack")]
    GitUploadPack {
        /// The git repository name
        repo: String,
    },
}

#[tokio::main]
async fn main() {
    let log = Logger::default();

    let server_config = ServerConfig::new().unwrap_or_else(|e| {
        log.error(&format!("Error loading server config: {}", e));
        std::process::exit(1);
    });

    let git = Git::new(&log, &server_config);
    let cli = Cli::parse();

    match &cli.command {
        Command::Logs => {
            println!("Showing logs...");
        }
        Command::ConfigSet { var } => {
            println!("Setting configuration: {}", var);
            // Parse `var` into key and value
            let parts: Vec<&str> = var.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0];
                let value = parts[1];
                println!("Setting {} to {}", key, value);
            } else {
                log.error("Invalid format. Use KEY=VALUE");
            }
        }
        Command::ConfigGet { key } => {
            println!("Getting configuration for: {}", key);
        }
        Command::Run => {
            log.section("Running application");
        }
        Command::Deploy => {
            log.section("Starting deployment");
        }
        Command::Stop => {
            log.section("Stopping application...");
        }
        Command::Destroy => {
            println!("Destroying application...");
        }
        Command::GitHook { repo } => {
            git.cmd_git_hook(repo);
            deploy(&log, repo, &server_config).await;
        }
        Command::GitReceivePack { repo } => {
            log.section("... RUKU ...");
            git.cmd_git_receive_pack(repo);
        }
        Command::GitUploadPack { repo } => {
            log.section("... RUKU ...");
            git.cmd_git_upload_pack(repo);
        }
    }
}

async fn deploy(log: &Logger, repo: &str, server_config: &ServerConfig) {
    log.section("Deploying application");
    let config = get_ruku_config(log, repo, server_config);
    let docker = get_docker(log).await;

    let app = sanitize_app_name(repo);
    let app_path = server_config.apps_root.join(&app);

    let container = Container::new(log, repo, &docker, &config);
    let deploy = Deploy::new(log, repo, app_path.as_path().to_str().unwrap(), &config, &container);
    deploy.run().await;
}

async fn get_docker(log: &Logger) -> Docker {
    let docker = load_docker(log).await;

    let version = docker
        .version()
        .await
        .unwrap_or_else(|_| {
            log.error("Ruku was unable to connect to docker");
            std::process::exit(1);
        })
        .version
        .unwrap();
    log.step(&format!("Docker engine version: {}", version));

    docker
}

async fn load_docker(log: &Logger) -> Docker {
    Docker::connect_with_local_defaults().unwrap_or_else(|_| {
        log.error("Ruku was unable to connect to docker");
        std::process::exit(1);
    })
}

fn get_ruku_config(log: &Logger, repo: &str, server_config: &ServerConfig) -> RukuConfig {
    let repo_path = server_config.apps_root.join(repo);

    // Check for the presence of ruku.yml file
    let config_path = repo_path.join("ruku.yml");
    if !config_path.exists() {
        log.error("ruku.yml file is missing in the repository");
        std::process::exit(1);
    }

    // Parse the ruku.yml file
    let config_content = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        log.error(&format!("Error reading ruku.yml file: {}", e));
        std::process::exit(1);
    });

    let config: RukuConfig = serde_yaml::from_str(&config_content).unwrap_or_else(|e| {
        log.error(&format!("Error parsing ruku.yml file: {}", e));
        std::process::exit(1);
    });

    match config.validate() {
        Ok(_) => (),
        Err(e) => {
            log.error(&format!("Error validating ruku.yml file: {}", e));
            std::process::exit(1);
        }
    };

    config
}
