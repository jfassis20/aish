use serde_json;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct WorkspaceContext {
    // Version Control
    pub is_git_repo: bool,
    pub git_branch: Option<String>,

    // Node.js Ecosystem
    pub is_node_project: bool,
    pub node_package_manager: Option<String>,
    pub node_version: Option<String>,
    pub is_typescript_project: bool,
    pub is_monorepo: bool,
    pub monorepo_tool: Option<String>, // lerna, nx, turborepo, pnpm-workspace

    // Python Ecosystem
    pub is_python_project: bool,
    pub python_package_manager: Option<String>,
    pub python_version: Option<String>,
    pub has_virtualenv: bool,

    // Java Ecosystem
    pub is_java_project: bool,
    pub java_project_manager: Option<String>,
    pub java_version: Option<String>,

    // Rust
    pub is_rust_project: bool,

    // Go
    pub is_go_project: bool,
    pub go_version: Option<String>,

    // PHP
    pub is_php_project: bool,

    // Ruby
    pub is_ruby_project: bool,
    pub ruby_version: Option<String>,

    // .NET
    pub is_dotnet_project: bool,

    // Web Frameworks
    pub web_framework: Option<String>, // react, vue, angular, nextjs, nuxt, svelte, remix, etc.
    pub backend_framework: Option<String>, // express, fastapi, django, flask, spring, etc.

    // Build Tools
    pub build_tool: Option<String>, // webpack, vite, rollup, esbuild, etc.

    // Testing
    pub testing_framework: Option<String>, // jest, vitest, pytest, unittest, etc.

    // Docker & Containers
    pub has_docker: bool,
    pub has_docker_compose: bool,
    pub has_kubernetes: bool,
    pub container_registry: Option<String>, // dockerhub, gcr, ecr, acr, etc.

    // CI/CD
    pub ci_cd_platform: Option<String>, // github-actions, gitlab-ci, jenkins, circleci, etc.

    // Cloud Providers
    pub cloud_provider: Option<String>, // aws, gcp, azure, etc.

    // Infrastructure as Code
    pub iac_tool: Option<String>, // terraform, pulumi, cdk, etc.

    // Databases
    pub database: Option<String>, // postgresql, mysql, mongodb, redis, etc.

    // Environment Files
    pub has_env_files: bool,

    // Linters & Formatters
    pub linter: Option<String>,    // eslint, pylint, rustfmt, etc.
    pub formatter: Option<String>, // prettier, black, gofmt, etc.
}

impl WorkspaceContext {
    pub fn detect(workspace_dir: &PathBuf) -> Self {
        let mut context = WorkspaceContext::default();

        // Detect Git repository
        let git_dir = workspace_dir.join(".git");
        context.is_git_repo = git_dir.exists();
        if context.is_git_repo {
            context.git_branch = get_git_branch(workspace_dir);
        }

        // Detect Node.js project
        if workspace_dir.join("package.json").exists() {
            context.is_node_project = true;
            context.node_package_manager = detect_node_package_manager(workspace_dir);
            context.node_version = detect_node_version(workspace_dir);
            context.is_typescript_project = workspace_dir.join("tsconfig.json").exists()
                || workspace_dir.join("tsconfig.base.json").exists();
            context.web_framework = detect_web_framework(workspace_dir);
            context.backend_framework = detect_backend_framework(workspace_dir);
            context.build_tool = detect_build_tool(workspace_dir);
            context.testing_framework = detect_testing_framework(workspace_dir);
            context.linter = detect_linter(workspace_dir);
            context.formatter = detect_formatter(workspace_dir);
            context.is_monorepo = detect_monorepo(workspace_dir);
            if context.is_monorepo {
                context.monorepo_tool = detect_monorepo_tool(workspace_dir);
            }
        }

        // Detect Python project
        if workspace_dir.join("requirements.txt").exists()
            || workspace_dir.join("pyproject.toml").exists()
            || workspace_dir.join("setup.py").exists()
            || workspace_dir.join("Pipfile").exists()
            || workspace_dir.join("poetry.lock").exists()
            || workspace_dir.join("setup.cfg").exists()
            || workspace_dir.join("pyproject.toml").exists()
        {
            context.is_python_project = true;
            context.python_package_manager = detect_python_package_manager(workspace_dir);
            context.python_version = detect_python_version(workspace_dir);
            context.has_virtualenv = workspace_dir.join("venv").exists()
                || workspace_dir.join(".venv").exists()
                || workspace_dir.join("env").exists()
                || workspace_dir.join(".env").exists();
            context.backend_framework = detect_python_backend_framework(workspace_dir);
            context.testing_framework = detect_python_testing_framework(workspace_dir);
        }

        // Detect Java project
        if workspace_dir.join("pom.xml").exists() {
            context.is_java_project = true;
            context.java_project_manager = Some("maven".to_string());
            context.java_version = detect_java_version_from_pom(workspace_dir);
            context.backend_framework = detect_java_backend_framework(workspace_dir);
        } else if workspace_dir.join("build.gradle").exists()
            || workspace_dir.join("build.gradle.kts").exists()
            || workspace_dir.join("settings.gradle").exists()
            || workspace_dir.join("settings.gradle.kts").exists()
        {
            context.is_java_project = true;
            context.java_project_manager = Some("gradle".to_string());
            context.java_version = detect_java_version_from_gradle(workspace_dir);
            context.backend_framework = detect_java_backend_framework(workspace_dir);
        }

        // Detect Rust project
        if workspace_dir.join("Cargo.toml").exists() {
            context.is_rust_project = true;
            context.linter = Some("clippy".to_string());
            context.formatter = Some("rustfmt".to_string());
        }

        // Detect Go project
        if workspace_dir.join("go.mod").exists() || workspace_dir.join("Gopkg.toml").exists() {
            context.is_go_project = true;
            context.go_version = detect_go_version(workspace_dir);
            context.formatter = Some("gofmt".to_string());
        }

        // Detect PHP project
        if workspace_dir.join("composer.json").exists() {
            context.is_php_project = true;
        }

        // Detect Ruby project
        if workspace_dir.join("Gemfile").exists() || workspace_dir.join("Rakefile").exists() {
            context.is_ruby_project = true;
            context.ruby_version = detect_ruby_version(workspace_dir);
        }

        // Detect .NET project
        if workspace_dir.join("*.csproj").exists()
            || workspace_dir.join("*.sln").exists()
            || workspace_dir.join("*.fsproj").exists()
        {
            context.is_dotnet_project = true;
        }

        // Detect Docker
        context.has_docker = workspace_dir.join("Dockerfile").exists()
            || workspace_dir.join("docker-compose.yml").exists()
            || workspace_dir.join("docker-compose.yaml").exists()
            || workspace_dir.join(".dockerignore").exists();
        context.has_docker_compose = workspace_dir.join("docker-compose.yml").exists()
            || workspace_dir.join("docker-compose.yaml").exists();
        context.has_kubernetes = workspace_dir.join("k8s").exists()
            || workspace_dir.join("kubernetes").exists()
            || workspace_dir.join("deployment.yaml").exists()
            || workspace_dir.join("deployment.yml").exists();

        // Detect CI/CD
        context.ci_cd_platform = detect_ci_cd(workspace_dir);

        // Detect Cloud Providers
        context.cloud_provider = detect_cloud_provider(workspace_dir);

        // Detect Infrastructure as Code
        context.iac_tool = detect_iac_tool(workspace_dir);

        // Detect Databases
        context.database = detect_database(workspace_dir);

        // Detect Environment Files
        context.has_env_files = workspace_dir.join(".env").exists()
            || workspace_dir.join(".env.local").exists()
            || workspace_dir.join(".env.development").exists()
            || workspace_dir.join(".env.production").exists();

        // Detect Container Registry
        context.container_registry = detect_container_registry(workspace_dir);

        context
    }

    pub fn to_flags_string(&self) -> String {
        let mut flags = Vec::new();

        if self.is_git_repo {
            flags.push("is_git_repo: true".to_string());
            if let Some(ref branch) = self.git_branch {
                flags.push(format!("git_branch: {}", branch));
            }
        }

        if self.is_node_project {
            flags.push("is_node_project: true".to_string());
            if let Some(ref pm) = self.node_package_manager {
                flags.push(format!("node_package_manager: {}", pm));
            }
            if let Some(ref version) = self.node_version {
                flags.push(format!("node_version: {}", version));
            }
            if self.is_typescript_project {
                flags.push("is_typescript_project: true".to_string());
            }
            if self.is_monorepo {
                flags.push("is_monorepo: true".to_string());
                if let Some(ref tool) = self.monorepo_tool {
                    flags.push(format!("monorepo_tool: {}", tool));
                }
            }
        }

        if self.is_java_project {
            flags.push("is_java_project: true".to_string());
            if let Some(ref pm) = self.java_project_manager {
                flags.push(format!("java_project_manager: {}", pm));
            }
            if let Some(ref version) = self.java_version {
                flags.push(format!("java_version: {}", version));
            }
        }

        if self.is_python_project {
            flags.push("is_python_project: true".to_string());
            if let Some(ref pm) = self.python_package_manager {
                flags.push(format!("python_package_manager: {}", pm));
            }
            if let Some(ref version) = self.python_version {
                flags.push(format!("python_version: {}", version));
            }
            if self.has_virtualenv {
                flags.push("has_virtualenv: true".to_string());
            }
        }

        if self.is_rust_project {
            flags.push("is_rust_project: true".to_string());
        }

        if self.is_go_project {
            flags.push("is_go_project: true".to_string());
            if let Some(ref version) = self.go_version {
                flags.push(format!("go_version: {}", version));
            }
        }

        if self.is_php_project {
            flags.push("is_php_project: true".to_string());
        }

        if self.is_ruby_project {
            flags.push("is_ruby_project: true".to_string());
            if let Some(ref version) = self.ruby_version {
                flags.push(format!("ruby_version: {}", version));
            }
        }

        if self.is_dotnet_project {
            flags.push("is_dotnet_project: true".to_string());
        }

        if let Some(ref framework) = self.web_framework {
            flags.push(format!("web_framework: {}", framework));
        }

        if let Some(ref framework) = self.backend_framework {
            flags.push(format!("backend_framework: {}", framework));
        }

        if let Some(ref tool) = self.build_tool {
            flags.push(format!("build_tool: {}", tool));
        }

        if let Some(ref framework) = self.testing_framework {
            flags.push(format!("testing_framework: {}", framework));
        }

        if self.has_docker {
            flags.push("has_docker: true".to_string());
        }

        if self.has_docker_compose {
            flags.push("has_docker_compose: true".to_string());
        }

        if self.has_kubernetes {
            flags.push("has_kubernetes: true".to_string());
        }

        if let Some(ref platform) = self.ci_cd_platform {
            flags.push(format!("ci_cd_platform: {}", platform));
        }

        if let Some(ref provider) = self.cloud_provider {
            flags.push(format!("cloud_provider: {}", provider));
        }

        if let Some(ref tool) = self.iac_tool {
            flags.push(format!("iac_tool: {}", tool));
        }

        if let Some(ref db) = self.database {
            flags.push(format!("database: {}", db));
        }

        if self.has_env_files {
            flags.push("has_env_files: true".to_string());
        }

        if let Some(ref linter) = self.linter {
            flags.push(format!("linter: {}", linter));
        }

        if let Some(ref formatter) = self.formatter {
            flags.push(format!("formatter: {}", formatter));
        }

        if let Some(ref registry) = self.container_registry {
            flags.push(format!("container_registry: {}", registry));
        }

        flags.join("\n")
    }
}

// Helper functions for detection

fn get_git_branch(workspace_dir: &PathBuf) -> Option<String> {
    use std::process::Command;
    Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(workspace_dir)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
}

fn detect_node_package_manager(workspace_dir: &PathBuf) -> Option<String> {
    if workspace_dir.join("bun.lockb").exists() {
        Some("bun".to_string())
    } else if workspace_dir.join("pnpm-lock.yaml").exists() {
        Some("pnpm".to_string())
    } else if workspace_dir.join("yarn.lock").exists() {
        Some("yarn".to_string())
    } else if workspace_dir.join("package-lock.json").exists() {
        Some("npm".to_string())
    } else {
        Some("npm".to_string()) // default
    }
}

fn detect_node_version(workspace_dir: &PathBuf) -> Option<String> {
    // Check .nvmrc
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join(".nvmrc")) {
        return Some(content.trim().to_string());
    }
    // Check .node-version
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join(".node-version")) {
        return Some(content.trim().to_string());
    }
    // Check package.json engines
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(engines) = json.get("engines").and_then(|e| e.get("node")) {
                if let Some(version) = engines.as_str() {
                    return Some(version.to_string());
                }
            }
        }
    }
    None
}

fn detect_web_framework(workspace_dir: &PathBuf) -> Option<String> {
    // Check package.json dependencies
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let deps = json
                .get("dependencies")
                .and_then(|d| d.as_object())
                .or_else(|| json.get("devDependencies").and_then(|d| d.as_object()));

            if let Some(deps) = deps {
                if deps.contains_key("next") {
                    return Some("nextjs".to_string());
                }
                if deps.contains_key("nuxt") || deps.contains_key("@nuxt/core") {
                    return Some("nuxt".to_string());
                }
                if deps.contains_key("remix") || deps.contains_key("@remix-run/react") {
                    return Some("remix".to_string());
                }
                if deps.contains_key("svelte") || deps.contains_key("sveltekit") {
                    return Some("svelte".to_string());
                }
                if deps.contains_key("react") {
                    return Some("react".to_string());
                }
                if deps.contains_key("vue") {
                    return Some("vue".to_string());
                }
                if deps.contains_key("@angular/core") {
                    return Some("angular".to_string());
                }
            }
        }
    }
    None
}

fn detect_backend_framework(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let deps = json
                .get("dependencies")
                .and_then(|d| d.as_object())
                .or_else(|| json.get("devDependencies").and_then(|d| d.as_object()));

            if let Some(deps) = deps {
                if deps.contains_key("express") {
                    return Some("express".to_string());
                }
                if deps.contains_key("fastify") {
                    return Some("fastify".to_string());
                }
                if deps.contains_key("koa") {
                    return Some("koa".to_string());
                }
                if deps.contains_key("nest") || deps.contains_key("@nestjs/core") {
                    return Some("nestjs".to_string());
                }
            }
        }
    }
    None
}

fn detect_python_backend_framework(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("requirements.txt")) {
        if content.contains("django") {
            return Some("django".to_string());
        }
        if content.contains("flask") {
            return Some("flask".to_string());
        }
        if content.contains("fastapi") {
            return Some("fastapi".to_string());
        }
    }
    if workspace_dir.join("pyproject.toml").exists() {
        if let Ok(content) = std::fs::read_to_string(workspace_dir.join("pyproject.toml")) {
            if content.contains("django") {
                return Some("django".to_string());
            }
            if content.contains("flask") {
                return Some("flask".to_string());
            }
            if content.contains("fastapi") {
                return Some("fastapi".to_string());
            }
        }
    }
    None
}

fn detect_java_backend_framework(workspace_dir: &PathBuf) -> Option<String> {
    // Check for Spring Boot
    if workspace_dir.join("pom.xml").exists() {
        if let Ok(content) = std::fs::read_to_string(workspace_dir.join("pom.xml")) {
            if content.contains("spring-boot") {
                return Some("spring-boot".to_string());
            }
        }
    }
    let gradle_file = if workspace_dir.join("build.gradle").exists() {
        Some(workspace_dir.join("build.gradle"))
    } else if workspace_dir.join("build.gradle.kts").exists() {
        Some(workspace_dir.join("build.gradle.kts"))
    } else {
        None
    };
    if let Some(file) = gradle_file {
        if let Ok(content) = std::fs::read_to_string(&file) {
            if content.contains("spring-boot") {
                return Some("spring-boot".to_string());
            }
        }
    }
    None
}

fn detect_build_tool(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let scripts = json.get("scripts").and_then(|s| s.as_object());
            let deps = json
                .get("dependencies")
                .and_then(|d| d.as_object())
                .or_else(|| json.get("devDependencies").and_then(|d| d.as_object()));

            if let Some(deps) = deps {
                if deps.contains_key("vite") {
                    return Some("vite".to_string());
                }
                if deps.contains_key("webpack") {
                    return Some("webpack".to_string());
                }
                if deps.contains_key("rollup") {
                    return Some("rollup".to_string());
                }
                if deps.contains_key("esbuild") {
                    return Some("esbuild".to_string());
                }
            }

            if let Some(scripts) = scripts {
                if scripts.contains_key("build") {
                    if let Some(build_cmd) = scripts.get("build").and_then(|s| s.as_str()) {
                        if build_cmd.contains("vite") {
                            return Some("vite".to_string());
                        }
                        if build_cmd.contains("webpack") {
                            return Some("webpack".to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn detect_testing_framework(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let deps = json
                .get("dependencies")
                .and_then(|d| d.as_object())
                .or_else(|| json.get("devDependencies").and_then(|d| d.as_object()));

            if let Some(deps) = deps {
                if deps.contains_key("vitest") {
                    return Some("vitest".to_string());
                }
                if deps.contains_key("jest") {
                    return Some("jest".to_string());
                }
                if deps.contains_key("mocha") {
                    return Some("mocha".to_string());
                }
                if deps.contains_key("jasmine") {
                    return Some("jasmine".to_string());
                }
            }
        }
    }
    None
}

fn detect_python_testing_framework(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("requirements.txt")) {
        if content.contains("pytest") {
            return Some("pytest".to_string());
        }
        if content.contains("unittest") {
            return Some("unittest".to_string());
        }
    }
    if workspace_dir.join("pytest.ini").exists() || workspace_dir.join("pyproject.toml").exists() {
        return Some("pytest".to_string());
    }
    None
}

fn detect_linter(workspace_dir: &PathBuf) -> Option<String> {
    if workspace_dir.join(".eslintrc").exists()
        || workspace_dir.join(".eslintrc.js").exists()
        || workspace_dir.join(".eslintrc.json").exists()
        || workspace_dir.join("eslint.config.js").exists()
    {
        return Some("eslint".to_string());
    }
    if workspace_dir.join(".pylintrc").exists()
        || workspace_dir.join("pylintrc").exists()
        || workspace_dir.join("setup.cfg").exists()
    {
        return Some("pylint".to_string());
    }
    if workspace_dir.join("Cargo.toml").exists() {
        return Some("clippy".to_string());
    }
    None
}

fn detect_formatter(workspace_dir: &PathBuf) -> Option<String> {
    if workspace_dir.join(".prettierrc").exists()
        || workspace_dir.join(".prettierrc.json").exists()
        || workspace_dir.join("prettier.config.js").exists()
        || workspace_dir.join("package.json").exists()
    {
        if let Ok(content) = std::fs::read_to_string(workspace_dir.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let deps = json
                    .get("dependencies")
                    .and_then(|d| d.as_object())
                    .or_else(|| json.get("devDependencies").and_then(|d| d.as_object()));
                if let Some(deps) = deps {
                    if deps.contains_key("prettier") {
                        return Some("prettier".to_string());
                    }
                }
            }
        }
    }
    if workspace_dir.join(".black").exists() || workspace_dir.join("pyproject.toml").exists() {
        if let Ok(content) = std::fs::read_to_string(workspace_dir.join("pyproject.toml")) {
            if content.contains("[tool.black]") {
                return Some("black".to_string());
            }
        }
    }
    if workspace_dir.join("Cargo.toml").exists() {
        return Some("rustfmt".to_string());
    }
    if workspace_dir.join("go.mod").exists() {
        return Some("gofmt".to_string());
    }
    None
}

fn detect_monorepo(workspace_dir: &PathBuf) -> bool {
    workspace_dir.join("lerna.json").exists()
        || workspace_dir.join("nx.json").exists()
        || workspace_dir.join("turbo.json").exists()
        || workspace_dir.join("pnpm-workspace.yaml").exists()
        || workspace_dir.join("yarn.lock").exists() && workspace_dir.join("packages").exists()
}

fn detect_monorepo_tool(workspace_dir: &PathBuf) -> Option<String> {
    if workspace_dir.join("lerna.json").exists() {
        Some("lerna".to_string())
    } else if workspace_dir.join("nx.json").exists() {
        Some("nx".to_string())
    } else if workspace_dir.join("turbo.json").exists() {
        Some("turborepo".to_string())
    } else if workspace_dir.join("pnpm-workspace.yaml").exists() {
        Some("pnpm-workspace".to_string())
    } else {
        Some("yarn-workspace".to_string())
    }
}

fn detect_python_package_manager(workspace_dir: &PathBuf) -> Option<String> {
    if workspace_dir.join("dependencies.yml").exists()
        || workspace_dir.join("environment.yml").exists()
    {
        Some("conda".to_string())
    } else if workspace_dir.join("poetry.lock").exists() {
        Some("poetry".to_string())
    } else if workspace_dir.join("Pipfile").exists() {
        Some("pipenv".to_string())
    } else {
        Some("pip".to_string())
    }
}

fn detect_python_version(workspace_dir: &PathBuf) -> Option<String> {
    // Check .python-version
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join(".python-version")) {
        return Some(content.trim().to_string());
    }
    // Check runtime.txt (for Heroku/Platform.sh)
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("runtime.txt")) {
        if content.starts_with("python-") {
            return Some(content.trim().replace("python-", ""));
        }
    }
    // Check pyproject.toml
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("pyproject.toml")) {
        if content.contains("requires-python") {
            // Simple extraction
            for line in content.lines() {
                if line.contains("requires-python") {
                    if let Some(version) = line.split('=').nth(1) {
                        return Some(version.trim().trim_matches('"').to_string());
                    }
                }
            }
        }
    }
    None
}

fn detect_java_version_from_pom(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("pom.xml")) {
        // Look for <java.version> or <maven.compiler.source>
        for line in content.lines() {
            if line.contains("<java.version>") {
                if let Some(version) = line.split('>').nth(1).and_then(|s| s.split('<').next()) {
                    return Some(version.trim().to_string());
                }
            }
            if line.contains("<maven.compiler.source>") {
                if let Some(version) = line.split('>').nth(1).and_then(|s| s.split('<').next()) {
                    return Some(version.trim().to_string());
                }
            }
        }
    }
    None
}

fn detect_java_version_from_gradle(workspace_dir: &PathBuf) -> Option<String> {
    let gradle_file = if workspace_dir.join("build.gradle").exists() {
        Some(workspace_dir.join("build.gradle"))
    } else if workspace_dir.join("build.gradle.kts").exists() {
        Some(workspace_dir.join("build.gradle.kts"))
    } else {
        None
    };
    if let Some(file) = gradle_file {
        if let Ok(content) = std::fs::read_to_string(&file) {
            for line in content.lines() {
                if line.contains("sourceCompatibility") || line.contains("java.toolchain") {
                    // Simple extraction
                    if let Some(version) = line.split('=').nth(1).or_else(|| line.split(':').nth(1))
                    {
                        let version = version.trim().trim_matches('"').trim_matches('\'');
                        if !version.is_empty() {
                            return Some(version.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn detect_go_version(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("go.mod")) {
        for line in content.lines() {
            if line.starts_with("go ") {
                return Some(line.split_whitespace().nth(1).unwrap_or("").to_string());
            }
        }
    }
    None
}

fn detect_ruby_version(workspace_dir: &PathBuf) -> Option<String> {
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join(".ruby-version")) {
        return Some(content.trim().to_string());
    }
    if let Ok(content) = std::fs::read_to_string(workspace_dir.join("Gemfile")) {
        for line in content.lines() {
            if line.contains("ruby") {
                if let Some(version) = line.split('"').nth(1).or_else(|| line.split('\'').nth(1)) {
                    return Some(version.to_string());
                }
            }
        }
    }
    None
}

fn detect_ci_cd(workspace_dir: &PathBuf) -> Option<String> {
    if workspace_dir.join(".github").join("workflows").exists() {
        return Some("github-actions".to_string());
    }
    if workspace_dir.join(".gitlab-ci.yml").exists() {
        return Some("gitlab-ci".to_string());
    }
    if workspace_dir.join("Jenkinsfile").exists() {
        return Some("jenkins".to_string());
    }
    if workspace_dir.join(".circleci").exists() {
        return Some("circleci".to_string());
    }
    if workspace_dir.join(".travis.yml").exists() {
        return Some("travis-ci".to_string());
    }
    if workspace_dir.join(".azure-pipelines.yml").exists() {
        return Some("azure-pipelines".to_string());
    }
    None
}

fn detect_cloud_provider(workspace_dir: &PathBuf) -> Option<String> {
    // AWS
    if workspace_dir.join(".aws").exists()
        || workspace_dir.join("serverless.yml").exists()
        || workspace_dir.join("serverless.yaml").exists()
        || workspace_dir.join("samconfig.toml").exists()
    {
        return Some("aws".to_string());
    }
    // GCP
    if workspace_dir.join("app.yaml").exists()
        || workspace_dir.join(".gcloud").exists()
        || workspace_dir.join("cloudbuild.yaml").exists()
    {
        return Some("gcp".to_string());
    }
    // Azure
    if workspace_dir.join("azure.yaml").exists()
        || workspace_dir.join(".azure").exists()
        || workspace_dir.join("azure-pipelines.yml").exists()
    {
        return Some("azure".to_string());
    }
    None
}

fn detect_iac_tool(workspace_dir: &PathBuf) -> Option<String> {
    if workspace_dir.join("terraform").exists()
        || workspace_dir.join("*.tf").exists()
        || workspace_dir.join(".terraform").exists()
    {
        return Some("terraform".to_string());
    }
    if workspace_dir.join("Pulumi.yaml").exists() || workspace_dir.join("Pulumi.yml").exists() {
        return Some("pulumi".to_string());
    }
    if workspace_dir.join("cdk.json").exists() {
        return Some("cdk".to_string());
    }
    if workspace_dir.join("ansible.cfg").exists() || workspace_dir.join("playbook.yml").exists() {
        return Some("ansible".to_string());
    }
    None
}

fn detect_database(workspace_dir: &PathBuf) -> Option<String> {
    // Check for database config files
    if workspace_dir.join("postgresql.conf").exists() || workspace_dir.join("pg_hba.conf").exists()
    {
        return Some("postgresql".to_string());
    }
    if workspace_dir.join("my.cnf").exists() || workspace_dir.join("mysql.cnf").exists() {
        return Some("mysql".to_string());
    }
    // Check package.json for database drivers
    if workspace_dir.join("package.json").exists() {
        if let Ok(content) = std::fs::read_to_string(workspace_dir.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let deps = json
                    .get("dependencies")
                    .and_then(|d| d.as_object())
                    .or_else(|| json.get("devDependencies").and_then(|d| d.as_object()));
                if let Some(deps) = deps {
                    if deps.contains_key("pg") || deps.contains_key("postgres") {
                        return Some("postgresql".to_string());
                    }
                    if deps.contains_key("mysql") || deps.contains_key("mysql2") {
                        return Some("mysql".to_string());
                    }
                    if deps.contains_key("mongodb") || deps.contains_key("mongoose") {
                        return Some("mongodb".to_string());
                    }
                    if deps.contains_key("redis") || deps.contains_key("ioredis") {
                        return Some("redis".to_string());
                    }
                    if deps.contains_key("sqlite3") {
                        return Some("sqlite".to_string());
                    }
                }
            }
        }
    }
    // Check Python requirements
    if workspace_dir.join("requirements.txt").exists() {
        if let Ok(content) = std::fs::read_to_string(workspace_dir.join("requirements.txt")) {
            if content.contains("psycopg") {
                return Some("postgresql".to_string());
            }
            if content.contains("mysql") || content.contains("pymysql") {
                return Some("mysql".to_string());
            }
            if content.contains("pymongo") {
                return Some("mongodb".to_string());
            }
            if content.contains("redis") {
                return Some("redis".to_string());
            }
        }
    }
    None
}

fn detect_container_registry(workspace_dir: &PathBuf) -> Option<String> {
    // Check docker-compose for registry
    let docker_compose_content = std::fs::read_to_string(workspace_dir.join("docker-compose.yml"))
        .or_else(|_| std::fs::read_to_string(workspace_dir.join("docker-compose.yaml")));
    if let Ok(content) = docker_compose_content {
        if content.contains("gcr.io") || content.contains("gcr.io") {
            return Some("gcr".to_string());
        }
        if content.contains("ecr") || content.contains("amazonaws.com") {
            return Some("ecr".to_string());
        }
        if content.contains("azurecr.io") {
            return Some("acr".to_string());
        }
        if content.contains("docker.io") || content.contains("dockerhub") {
            return Some("dockerhub".to_string());
        }
    }
    // Check for cloud provider configs
    if workspace_dir.join(".aws").exists() {
        return Some("ecr".to_string());
    }
    if workspace_dir.join(".gcloud").exists() {
        return Some("gcr".to_string());
    }
    if workspace_dir.join(".azure").exists() {
        return Some("acr".to_string());
    }
    None
}
