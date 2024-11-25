use nixpacks::create_docker_image;
use nixpacks::nixpacks::builder::docker::DockerBuilderOptions;
use nixpacks::nixpacks::plan::phase::{Phase, StartPhase};
use nixpacks::nixpacks::plan::{generator::GeneratePlanOptions, BuildPlan};

use crate::container::Container;
use crate::logger::Logger;
use crate::misc::get_image_name_with_version;
use crate::model::RukuConfig;

pub struct Deploy<'a> {
    log: &'a Logger,
    name: &'a str,
    path: &'a str,
    config: &'a RukuConfig,
    container: &'a Container<'a>,
}

impl<'a> Deploy<'a> {
    pub fn new(
        log: &'a Logger,
        name: &'a str,
        path: &'a str,
        config: &'a RukuConfig,
        container: &'a Container<'a>,
    ) -> Deploy<'a> {
        Deploy {
            log,
            name,
            path,
            config,
            container,
        }
    }

    pub async fn run(&self) {
        self.log.step(&format!("Running from {}", self.path));

        // Nix pack
        let envs: Vec<&str> = vec![];
        let mut cli_plan = BuildPlan::default();
        if let Some(install_cmds) = &self.config.install_cmd {
            let mut install = Phase::install(None);
            install.cmds = Some(vec![install_cmds.clone()]);
            cli_plan.add_phase(install);
        }
        if let Some(build_cmds) = &self.config.build_cmd {
            let mut build = Phase::build(None);
            build.cmds = Some(vec![build_cmds.clone()]);
            cli_plan.add_phase(build);
        }
        if let Some(start_cmd) = &self.config.start_cmd {
            let start = StartPhase::new(start_cmd.clone());
            cli_plan.set_start_phase(start);
        }

        let options = GeneratePlanOptions {
            plan: Some(cli_plan),
            config_file: None,
        };

        let image_name_with_version = get_image_name_with_version(self.name, &self.config.version);

        let build_options = DockerBuilderOptions {
            name: Some(self.name.to_string()),
            out_dir: None,
            print_dockerfile: false,
            tags: vec![image_name_with_version.clone()],
            labels: vec![],
            quiet: false,
            cache_key: None,
            no_cache: false,
            inline_cache: false,
            cache_from: None,
            platform: vec![],
            current_dir: true,
            no_error_without_start: false,
            incremental_cache_image: None,
            cpu_quota: None,
            memory: None,
            verbose: false,
            docker_host: None,
            docker_tls_verify: None,
            docker_output: None,
            add_host: vec![],
            docker_cert_path: None,
        };

        if let Err(e) = create_docker_image(self.path, envs, &options, &build_options).await {
            self.log
                .error(&format!("Error creating Docker image at path {}: {}", self.path, e));
            std::process::exit(1);
        }

        self.log.step(&format!(
            "Image created successfully with tag {}",
            image_name_with_version
        ));

        self.container.run().await;
    }
}
