use std::collections::HashMap;
use std::str::FromStr;

use bollard::container::{CreateContainerOptions, ListContainersOptions, StartContainerOptions};
use bollard::models::{
    ContainerCreateResponse, ContainerStateStatusEnum, ContainerSummary, HostConfig, PortBinding, PortMap,
};
use bollard::Docker;

use crate::logger::Logger;
use crate::misc::get_image_name_with_version;
use crate::model::RukuConfig;

pub struct Container<'a> {
    log: &'a Logger,
    name: &'a str,
    docker: &'a Docker,
    config: &'a RukuConfig,
}

impl<'a> Container<'a> {
    pub fn new(log: &'a Logger, name: &'a str, docker: &'a Docker, config: &'a RukuConfig) -> Container<'a> {
        Container {
            log,
            name,
            docker,
            config,
        }
    }

    pub async fn run(&self) {
        let image_name_with_version = get_image_name_with_version(self.name, &self.config.version);

        if let Some(container) = self.get().await {
            let container_id = container.id.as_deref().unwrap_or_else(|| {
                self.log.error("Failed to get container id");
                std::process::exit(1);
            });
            let container_state = container.state.as_deref().unwrap_or_else(|| {
                self.log.error("Failed to get container state");
                std::process::exit(1);
            });
            match ContainerStateStatusEnum::from_str(container_state).unwrap() {
                ContainerStateStatusEnum::EMPTY => {}
                ContainerStateStatusEnum::RUNNING | ContainerStateStatusEnum::RESTARTING => {
                    self.stop_and_remove(container_id).await;
                }
                ContainerStateStatusEnum::REMOVING => {}
                ContainerStateStatusEnum::CREATED
                | ContainerStateStatusEnum::PAUSED
                | ContainerStateStatusEnum::EXITED
                | ContainerStateStatusEnum::DEAD => {
                    self.remove(container_id).await;
                }
            }
            let new_container = self.create(image_name_with_version).await;
            self.start(&new_container.id).await;
        } else {
            let container = self.create(image_name_with_version).await;
            self.start(&container.id).await;
        }
    }

    pub async fn end(&self) {
        if let Some(container) = self.get().await {
            let container_id = container.id.as_deref().unwrap_or_else(|| {
                self.log.error("Failed to get container id");
                std::process::exit(1);
            });
            self.stop_and_remove(container_id).await;
        } else {
            self.log.error("No application is running");
        }
    }

    async fn stop_and_remove(&self, container_id: &str) {
        self.stop(container_id).await;
        self.remove(container_id).await;
    }

    async fn stop(&self, container_id: &str) {
        self.docker
            .stop_container(container_id, None)
            .await
            .unwrap_or_else(|_| {
                self.log.error("Failed to stop container");
                std::process::exit(1);
            });
        self.log.step(&format!("Stopped container with id: {}", container_id));
    }

    async fn remove(&self, container_id: &str) {
        self.docker
            .remove_container(container_id, None)
            .await
            .unwrap_or_else(|_| {
                self.log.error("Failed to remove container");
                std::process::exit(1);
            });
        self.log.step(&format!("Removed container with id: {}", container_id));
    }

    async fn start(&self, container_id: &str) {
        self.docker
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .unwrap_or_else(|_| {
                self.log.error("Failed to start container");
                std::process::exit(1);
            });
        self.log.step(&format!("Started container with id: {}", container_id));
    }

    pub async fn get(&self) -> Option<ContainerSummary> {
        let mut filters = HashMap::new();
        filters.insert("name", vec![self.name]);

        let options = Some(ListContainersOptions {
            all: true,
            filters,
            limit: Some(1),
            ..Default::default()
        });
        let containers = self.docker.list_containers(options).await.unwrap_or_else(|_| {
            self.log.error("Failed to list containers");
            std::process::exit(1);
        });
        containers.into_iter().next()
    }

    pub async fn create(&self, image_name: String) -> ContainerCreateResponse {
        let create_options = CreateContainerOptions {
            name: self.name,
            platform: None,
        };

        let exposed_port = format!("{}/tcp", self.config.port);
        let mut host_config = HostConfig::default();
        let mut port_bindings = PortMap::new();
        port_bindings.insert(
            exposed_port.clone(),
            Some(vec![PortBinding {
                host_ip: None,
                host_port: Some(self.config.port.to_string()),
            }]),
        );
        host_config.port_bindings = Some(port_bindings);

        let mut exposed_ports_map: HashMap<String, HashMap<(), ()>> = HashMap::new();
        exposed_ports_map.insert(exposed_port, HashMap::new());

        let create_container_config = bollard::container::Config {
            image: Some(image_name),
            host_config: Some(host_config),
            exposed_ports: Some(exposed_ports_map),
            ..Default::default()
        };

        // Create the container
        let container = self
            .docker
            .create_container(Some(create_options), create_container_config)
            .await
            .unwrap_or_else(|_| {
                self.log.error("Failed to create container");
                std::process::exit(1);
            });
        self.log.step(&format!("Created container with id: {}", container.id));
        container
    }
}
