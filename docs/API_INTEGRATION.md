# DockMon API Integration Guide

## Overview

DockMon integrates with Docker through:
1. **Bollard** - Official Rust Docker API client
2. **Docker Engine API** - REST API via Unix socket or TCP
3. **Registry APIs** - Docker Hub and custom registries
4. **Docker Compose** - CLI wrapper for compose operations

## Docker API Integration

### Connection Management

```rust
pub struct DockerClient {
    client: Docker,
    connection_info: ConnectionInfo,
    request_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub host: String,
    pub version: String,
    pub api_version: String,
    pub os: String,
    pub arch: String,
}

impl DockerClient {
    /// Create new client from environment
    pub async fn from_env() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        Self::new(docker).await
    }
    
    /// Create with custom host
    pub async fn with_host(host: &str) -> Result<Self> {
        let docker = Docker::connect_with_http(host, 120, API_DEFAULT_VERSION)?;
        Self::new(docker).await
    }
    
    async fn new(client: Docker) -> Result<Self> {
        let version = client.version().await?;
        let info = ConnectionInfo {
            host: client.docker_host().to_string(),
            version: version.version.unwrap_or_default(),
            api_version: version.api_version.unwrap_or_default(),
            os: version.os.unwrap_or_default(),
            arch: version.arch.unwrap_or_default(),
        };
        
        Ok(Self {
            client,
            connection_info: info,
            request_timeout: Duration::from_secs(30),
        })
    }
    
    /// Health check
    pub async fn ping(&self) -> Result<String> {
        Ok(self.client.ping().await?)
    }
}
```

### Container Operations

```rust
impl DockerClient {
    /// List all containers
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        let options = ListContainersOptions {
            all,
            ..Default::default()
        };
        
        let containers = self.client
            .list_containers(Some(options))
            .await?;
            
        Ok(containers.into_iter()
            .map(|c| c.into())
            .collect())
    }
    
    /// Get container details
    pub async fn inspect_container(&self, id: &str) -> Result<ContainerDetails> {
        let container = self.client
            .inspect_container(id, None)
            .await?;
            
        Ok(container.into())
    }
    
    /// Start container
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.client
            .start_container::<String>(id, None)
            .await?;
        Ok(())
    }
    
    /// Stop container
    pub async fn stop_container(&self, id: &str, timeout: Option<i32>) -> Result<()> {
        let options = StopContainerOptions {
            t: timeout.unwrap_or(10),
        };
        
        self.client
            .stop_container(id, Some(options))
            .await?;
        Ok(())
    }
    
    /// Restart container
    pub async fn restart_container(&self, id: &str, timeout: Option<i32>) -> Result<()> {
        let options = RestartContainerOptions {
            t: timeout.unwrap_or(10),
        };
        
        self.client
            .restart_container(id, Some(options))
            .await?;
        Ok(())
    }
    
    /// Pause container
    pub async fn pause_container(&self, id: &str) -> Result<()> {
        self.client
            .pause_container(id)
            .await?;
        Ok(())
    }
    
    /// Unpause container
    pub async fn unpause_container(&self, id: &str) -> Result<()> {
        self.client
            .unpause_container(id)
            .await?;
        Ok(())
    }
    
    /// Kill container
    pub async fn kill_container(&self, id: &str, signal: Option<&str>) -> Result<()> {
        let options = KillContainerOptions {
            signal: signal.unwrap_or("SIGKILL"),
        };
        
        self.client
            .kill_container(id, Some(options))
            .await?;
        Ok(())
    }
    
    /// Remove container
    pub async fn remove_container(
        &self,
        id: &str,
        force: bool,
        remove_volumes: bool,
    ) -> Result<()> {
        let options = RemoveContainerOptions {
            v: remove_volumes,
            force,
            link: false,
        };
        
        self.client
            .remove_container(id, Some(options))
            .await?;
        Ok(())
    }
    
    /// Rename container
    pub async fn rename_container(&self, id: &str, new_name: &str) -> Result<()> {
        let options = RenameContainerOptions {
            name: new_name,
        };
        
        self.client
            .rename_container(id, options)
            .await?;
        Ok(())
    }
    
    /// Stream container stats
    pub async fn stream_stats(
        &self,
        id: &str,
        stream: bool,
    ) -> Result<impl Stream<Item = Result<ContainerStats>>> {
        let options = StatsOptions {
            stream,
            one_shot: !stream,
        };
        
        let stream = self.client
            .stats(id, Some(options))
            .map(|result| result.map_err(|e| e.into()));
            
        Ok(stream)
    }
    
    /// Stream container logs
    pub async fn stream_logs(
        &self,
        id: &str,
        options: &LogOptions,
    ) -> Result<impl Stream<Item = Result<LogOutput>>> {
        let opts = LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: options.timestamps,
            since: options.since,
            until: options.until,
            tail: options.tail.clone(),
            follow: options.follow,
        };
        
        let stream = self.client
            .logs(id, Some(opts))
            .map(|result| result.map_err(|e| e.into()));
            
        Ok(stream)
    }
    
    /// Execute command in container
    pub async fn exec_command(
        &self,
        id: &str,
        cmd: Vec<String>,
        interactive: bool,
        tty: bool,
    ) -> Result<ExecInstance> {
        let config = CreateExecOptions {
            cmd: Some(cmd),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            attach_stdin: Some(interactive),
            tty: Some(tty),
            ..Default::default()
        };
        
        let exec = self.client
            .create_exec(id, config)
            .await?;
            
        Ok(ExecInstance {
            id: exec.id,
            client: self.client.clone(),
        })
    }
    
    /// Get container processes (top)
    pub async fn container_top(&self, id: &str) -> Result<ProcessList> {
        let top = self.client
            .top_processes(id, None::<TopOptions<String>>)
            .await?;
            
        Ok(top.into())
    }
    
    /// Get container changes (filesystem diff)
    pub async fn container_changes(&self, id: &str) -> Result<Vec<FilesystemChange>> {
        let changes = self.client
            .container_changes(id)
            .await?;
            
        Ok(changes.into_iter().map(|c| c.into()).collect())
    }
}
```

### Image Operations

```rust
impl DockerClient {
    /// List images
    pub async fn list_images(&self, all: bool) -> Result<Vec<ImageSummary>> {
        let options = ListImagesOptions {
            all,
            ..Default::default()
        };
        
        let images = self.client
            .list_images(Some(options))
            .await?;
            
        Ok(images.into_iter().map(|i| i.into()).collect())
    }
    
    /// Inspect image
    pub async fn inspect_image(&self, id: &str) -> Result<ImageDetails> {
        let image = self.client
            .inspect_image(id)
            .await?;
            
        Ok(image.into())
    }
    
    /// Pull image with progress
    pub async fn pull_image(
        &self,
        image: &str,
        tag: &str,
        auth: Option<RegistryAuth>,
    ) -> Result<impl Stream<Item = Result<PullProgress>>> {
        let options = CreateImageOptions {
            from_image: image,
            tag,
            ..Default::default()
        };
        
        let stream = self.client
            .create_image(Some(options), None, auth)
            .map(|result| result.map_err(|e| e.into()));
            
        Ok(stream)
    }
    
    /// Push image
    pub async fn push_image(
        &self,
        image: &str,
        tag: &str,
        auth: Option<RegistryAuth>,
    ) -> Result<impl Stream<Item = Result<PushProgress>>> {
        let options = PushImageOptions {
            tag,
        };
        
        let stream = self.client
            .push_image(options, None, Some(image), auth)
            .map(|result| result.map_err(|e| e.into()));
            
        Ok(stream)
    }
    
    /// Tag image
    pub async fn tag_image(
        &self,
        id: &str,
        repo: &str,
        tag: &str,
    ) -> Result<()> {
        let options = TagImageOptions {
            repo,
            tag,
        };
        
        self.client
            .tag_image(id, Some(options))
            .await?;
        Ok(())
    }
    
    /// Remove image
    pub async fn remove_image(
        &self,
        id: &str,
        force: bool,
        noprune: bool,
    ) -> Result<Vec<ImageDeleteResponseItem>> {
        let options = RemoveImageOptions {
            force,
            noprune,
        };
        
        let response = self.client
            .remove_image(id, Some(options), None)
            .await?;
            
        Ok(response)
    }
    
    /// Build image from Dockerfile
    pub async fn build_image(
        &self,
        build_context: Vec<u8>, // Tarball
        options: BuildImageOptions,
    ) -> Result<impl Stream<Item = Result<BuildInfo>>> {
        let stream = self.client
            .build_image(options, None, Some(build_context.into()))
            .map(|result| result.map_err(|e| e.into()));
            
        Ok(stream)
    }
    
    /// Get image history
    pub async fn image_history(&self, id: &str) -> Result<Vec<ImageHistoryItem>> {
        let history = self.client
            .image_history(id)
            .await?;
            
        Ok(history.into_iter().map(|h| h.into()).collect())
    }
    
    /// Prune unused images
    pub async fn prune_images(&self, dangling_only: bool) -> Result<ImagePruneResponse> {
        let filters = if dangling_only {
            let mut map = HashMap::new();
            map.insert("dangling", vec!["true"]);
            Some(map)
        } else {
            None
        };
        
        let response = self.client
            .prune_images(PruneImagesOptions { filters })
            .await?;
            
        Ok(response)
    }
}
```

### Volume Operations

```rust
impl DockerClient {
    /// List volumes
    pub async fn list_volumes(&self) -> Result<Vec<VolumeSummary>> {
        let response = self.client
            .list_volumes::<String>(None)
            .await?;
            
        let containers = self.list_containers(true).await?;
        let volume_usage = Self::build_volume_usage_map(&containers);
        
        Ok(response
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| VolumeSummary::from_docker(v, volume_usage.get(&v.name)))
            .collect())
    }
    
    /// Create volume
    pub async fn create_volume(
        &self,
        name: &str,
        driver: &str,
        driver_opts: HashMap<String, String>,
        labels: HashMap<String, String>,
    ) -> Result<Volume> {
        let config = CreateVolumeOptions {
            name,
            driver,
            driver_opts,
            labels,
        };
        
        let volume = self.client
            .create_volume(config)
            .await?;
            
        Ok(volume)
    }
    
    /// Remove volume
    pub async fn remove_volume(&self, name: &str, force: bool) -> Result<()> {
        let options = RemoveVolumeOptions {
            force,
        };
        
        self.client
            .remove_volume(name, Some(options))
            .await?;
        Ok(())
    }
    
    /// Prune unused volumes
    pub async fn prune_volumes(&self) -> Result<VolumesPruneResponse> {
        let response = self.client
            .prune_volumes::<String>(None)
            .await?;
            
        Ok(response)
    }
}
```

### Network Operations

```rust
impl DockerClient {
    /// List networks
    pub async fn list_networks(&self) -> Result<Vec<NetworkSummary>> {
        let options = ListNetworksOptions {
            ..Default::default()
        };
        
        let networks = self.client
            .list_networks(Some(options))
            .await?;
            
        Ok(networks.into_iter().map(|n| n.into()).collect())
    }
    
    /// Create network
    pub async fn create_network(
        &self,
        name: &str,
        driver: &str,
        options: NetworkCreateOptions,
    ) -> Result<NetworkCreateResponse> {
        let config = CreateNetworkOptions {
            name,
            driver,
            ipam: options.ipam.map(|i| i.into()),
            options: options.driver_opts,
            labels: options.labels,
            check_duplicate: true,
            internal: options.internal,
            attachable: options.attachable,
            ingress: options.ingress,
            enable_ipv6: options.enable_ipv6,
        };
        
        let response = self.client
            .create_network(config)
            .await?;
            
        Ok(response)
    }
    
    /// Remove network
    pub async fn remove_network(&self, id: &str) -> Result<()> {
        self.client
            .remove_network(id)
            .await?;
        Ok(())
    }
    
    /// Prune unused networks
    pub async fn prune_networks(&self) -> Result<NetworksPruneResponse> {
        let response = self.client
            .prune_networks::<String>(None)
            .await?;
            
        Ok(response)
    }
}
```

### Event Streaming

```rust
impl DockerClient {
    /// Stream Docker events
    pub async fn stream_events(
        &self,
        filters: EventFilters,
    ) -> Result<impl Stream<Item = Result<SystemEventsResponse>>> {
        let options = EventsOptions {
            since: filters.since.map(|t| t.timestamp()),
            until: filters.until.map(|t| t.timestamp()),
            filters: filters.to_map(),
        };
        
        let stream = self.client
            .events(Some(options))
            .map(|result| result.map_err(|e| e.into()));
            
        Ok(stream)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventFilters {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub event_types: Vec<EventType>,
    pub containers: Vec<String>,
    pub images: Vec<String>,
    pub volumes: Vec<String>,
    pub networks: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Container,
    Image,
    Volume,
    Network,
    Daemon,
}
```

### System Operations

```rust
impl DockerClient {
    /// Get system info
    pub async fn system_info(&self) -> Result<SystemInfo> {
        let info = self.client
            .info()
            .await?;
            
        Ok(info.into())
    }
    
    /// Get disk usage
    pub async fn disk_usage(&self) -> Result<DiskUsage> {
        let usage = self.client
            .df()
            .await?;
            
        Ok(usage.into())
    }
    
    /// Prune all unused data
    pub async fn prune_all(&self) -> Result<SystemPruneResponse> {
        let options = SystemPruneOptions {
            all: true,
            volumes: true,
        };
        
        let response = self.client
            .prune_system(Some(options))
            .await?;
            
        Ok(response)
    }
}
```

## Docker Compose Integration

```rust
pub struct ComposeClient {
    project_dir: PathBuf,
    files: Vec<PathBuf>,
    env_file: Option<PathBuf>,
}

impl ComposeClient {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            project_dir,
            files: vec![],
            env_file: None,
        }
    }
    
    pub fn with_files(mut self, files: Vec<PathBuf>) -> Self {
        self.files = files;
        self
    }
    
    /// Parse compose file
    pub async fn parse(&self) -> Result<ComposeProject> {
        let output = Command::new("docker")
            .args(&[
                "compose",
                "-f", self.compose_file().to_str().unwrap(),
                "config",
            ])
            .current_dir(&self.project_dir)
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(ComposeError::ParseError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }
        
        let config: ComposeConfig = serde_yaml::from_slice(&output.stdout)?;
        Ok(ComposeProject::from_config(config))
    }
    
    /// Start services
    pub async fn up(&self, services: Option<Vec<String>>, detach: bool) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        
        for file in &self.files {
            cmd.arg("-f").arg(file);
        }
        
        cmd.arg("up");
        
        if detach {
            cmd.arg("-d");
        }
        
        if let Some(svcs) = services {
            for svc in svcs {
                cmd.arg(svc);
            }
        }
        
        let output = cmd
            .current_dir(&self.project_dir)
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(ComposeError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }
        
        Ok(())
    }
    
    /// Stop services
    pub async fn down(&self, volumes: bool) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("down");
        
        if volumes {
            cmd.arg("-v");
        }
        
        let output = cmd
            .current_dir(&self.project_dir)
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(ComposeError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }
        
        Ok(())
    }
    
    /// Scale service
    pub async fn scale(&self, service: &str, replicas: u32) -> Result<()> {
        let output = Command::new("docker")
            .args(&[
                "compose",
                "up",
                "-d",
                "--scale",
                &format!("{}={}", service, replicas),
                service,
            ])
            .current_dir(&self.project_dir)
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(ComposeError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }
        
        Ok(())
    }
    
    /// Stream compose logs
    pub async fn logs(
        &self,
        services: Option<Vec<String>>,
        follow: bool,
        tail: Option<u64>,
    ) -> Result<Child> {
        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("logs");
        
        if follow {
            cmd.arg("-f");
        }
        
        if let Some(t) = tail {
            cmd.arg("--tail").arg(t.to_string());
        }
        
        if let Some(svcs) = services {
            for svc in svcs {
                cmd.arg(svc);
            }
        }
        
        let child = cmd
            .current_dir(&self.project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
            
        Ok(child)
    }
}
```

## Registry Integration

### Docker Hub API

```rust
pub struct DockerHubClient {
    client: reqwest::Client,
    base_url: String,
}

impl DockerHubClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://hub.docker.com/v2".to_string(),
        }
    }
    
    /// Search for images
    pub async fn search(
        &self,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResponse> {
        let url = format!("{}/search/repositories", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("query", query),
                ("page", &page.to_string()),
                ("page_size", &page_size.to_string()),
            ])
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(RegistryError::ApiError(
                response.status(),
                response.text().await.unwrap_or_default(),
            ).into());
        }
        
        Ok(response.json().await?)
    }
    
    /// Get image tags
    pub async fn list_tags(&self, repository: &str, page: u32) -> Result<TagResponse> {
        let url = format!("{}/repositories/{}/tags", self.base_url, repository);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("page", &page.to_string()),
                ("page_size", "100"),
            ])
            .send()
            .await?;
            
        Ok(response.json().await?)
    }
}
```

### Custom Registry

```rust
pub struct RegistryClient {
    client: reqwest::Client,
    url: String,
    auth: Option<RegistryAuth>,
}

impl RegistryClient {
    pub fn new(url: String, auth: Option<RegistryAuth>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            auth,
        }
    }
    
    /// Authenticate with registry
    pub async fn authenticate(&self) -> Result<String> {
        // Implement OAuth2 or basic auth flow
        // Return token for subsequent requests
        todo!()
    }
    
    /// List repositories
    pub async fn list_repositories(&self) -> Result<Vec<String>> {
        let url = format!("{}/v2/_catalog", self.url);
        
        let mut request = self.client.get(&url);
        if let Some(auth) = &self.auth {
            request = request.bearer_auth(&auth.token.as_ref().unwrap());
        }
        
        let response = request.send().await?;
        let catalog: CatalogResponse = response.json().await?;
        
        Ok(catalog.repositories)
    }
    
    /// List tags for repository
    pub async fn list_tags(&self, repository: &str) -> Result<Vec<String>> {
        let url = format!("{}/v2/{}/tags/list", self.url, repository);
        
        let mut request = self.client.get(&url);
        if let Some(auth) = &self.auth {
            request = request.bearer_auth(&auth.token.as_ref().unwrap());
        }
        
        let response = request.send().await?;
        let tags: TagsResponse = response.json().await?;
        
        Ok(tags.tags)
    }
    
    /// Get manifest (image details)
    pub async fn get_manifest(&self, repository: &str, tag: &str) -> Result<Manifest> {
        let url = format!("{}/v2/{}/manifests/{}", self.url, repository, tag);
        
        let mut request = self.client
            .get(&url)
            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json");
            
        if let Some(auth) = &self.auth {
            request = request.bearer_auth(&auth.token.as_ref().unwrap());
        }
        
        let response = request.send().await?;
        let manifest: Manifest = response.json().await?;
        
        Ok(manifest)
    }
}
```

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Docker API error: {0}")]
    Docker(#[from] bollard::errors::Error),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Timeout after {duration}s")]
    Timeout { duration: u64 },
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Compose error: {0}")]
    Compose(#[from] ComposeError),
    
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),
}

#[derive(Debug, Error)]
pub enum ComposeError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Command error: {0}")]
    CommandError(String),
    
    #[error("Not a compose project")]
    NotAComposeProject,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("API error {0}: {1}")]
    ApiError(reqwest::StatusCode, String),
    
    #[error("Authentication failed")]
    AuthFailed,
    
    #[error("Not found")]
    NotFound,
}
```

## Rate Limiting & Caching

```rust
pub struct CachedClient {
    inner: DockerClient,
    cache: Arc<Mutex<Cache>>,
}

struct Cache {
    containers: Option<(Vec<ContainerSummary>, Instant)>,
    images: Option<(Vec<ImageSummary>, Instant)>,
    ttl: Duration,
}

impl CachedClient {
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>> {
        let mut cache = self.cache.lock().await;
        
        if let Some((containers, timestamp)) = &cache.containers {
            if timestamp.elapsed() < cache.ttl {
                return Ok(containers.clone());
            }
        }
        
        let containers = self.inner.list_containers(all).await?;
        cache.containers = Some((containers.clone(), Instant::now()));
        
        Ok(containers)
    }
    
    pub fn invalidate_cache(&self) {
        // Called after mutations
        let mut cache = self.cache.blocking_lock();
        cache.containers = None;
        cache.images = None;
    }
}
```
