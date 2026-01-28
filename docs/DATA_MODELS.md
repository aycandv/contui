# DockMon Data Models

## Core Types

### Container

```rust
/// Unique identifier for a container
pub type ContainerId = String;

/// Container summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummary {
    pub id: ContainerId,
    pub short_id: String,           // First 12 characters
    pub names: Vec<String>,
    pub image: String,
    pub image_id: String,
    pub command: String,
    pub created: DateTime<Utc>,
    pub ports: Vec<PortMapping>,
    pub size_rw: Option<i64>,
    pub size_root_fs: Option<i64>,
    pub labels: HashMap<String, String>,
    pub state: ContainerState,
    pub status: String,
    pub health: Option<HealthStatus>,
    pub mounts: Vec<MountPoint>,
    pub networks: Vec<String>,
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,
}

/// Container runtime state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
    Unknown,
}

/// Port mapping information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub ip: Option<String>,
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub protocol: String,           // "tcp" or "udp"
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Unhealthy,
    None,
}

/// Mount point information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub source: String,
    pub destination: String,
    pub mode: String,               // "rw", "ro", etc.
    pub rw: bool,
    pub propagation: String,
    pub typ: MountType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MountType {
    Bind,
    Volume,
    Tmpfs,
    Npipe,
}
```

### Container Details

```rust
/// Extended container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDetails {
    pub summary: ContainerSummary,
    pub path: String,                    // Entrypoint path
    pub args: Vec<String>,               // Command arguments
    pub config: ContainerConfig,         // Container configuration
    pub network_settings: NetworkSettings,
    pub host_config: HostConfig,
    pub exec_ids: Vec<String>,
    pub graph_driver: GraphDriverData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub hostname: String,
    pub domainname: String,
    pub user: String,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub exposed_ports: HashMap<String, EmptyStruct>,
    pub tty: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
    pub env: Vec<String>,
    pub cmd: Vec<String>,
    pub healthcheck: Option<HealthConfig>,
    pub image: String,
    pub working_dir: String,
    pub entrypoint: Vec<String>,
    pub on_build: Vec<String>,
    pub labels: HashMap<String, String>,
    pub stop_signal: String,
    pub stop_timeout: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub test: Vec<String>,
    pub interval: i64,              // Nanoseconds
    pub timeout: i64,               // Nanoseconds
    pub retries: i32,
    pub start_period: i64,          // Nanoseconds
    pub start_interval: i64,        // Nanoseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub bridge: String,
    pub sandbox_id: String,
    pub hairpin_mode: bool,
    pub link_local_ipv6_address: String,
    pub link_local_ipv6_prefix_len: i32,
    pub ports: HashMap<String, Vec<PortBinding>>,
    pub sandbox_key: String,
    pub secondary_ip_addresses: Vec<Address>,
    pub secondary_ipv6_addresses: Vec<Address>,
    pub endpoint_id: String,
    pub gateway: String,
    pub global_ipv6_address: String,
    pub global_ipv6_prefix_len: i32,
    pub ip_address: String,
    pub ip_prefix_len: i32,
    pub ipv6_gateway: String,
    pub mac_address: String,
    pub networks: HashMap<String, EndpointSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub cpu_shares: i64,
    pub memory: i64,
    pub memory_swap: i64,
    pub memory_reservation: i64,
    pub kernel_memory: i64,
    pub cpu_percent: i64,
    pub cpu_quota: i64,
    pub cpu_period: i64,
    pub cpu_realtime_period: i64,
    pub cpu_realtime_runtime: i64,
    pub cpuset_cpus: String,
    pub cpuset_mems: String,
    pub memory_swappiness: i64,
    pub oom_kill_disable: bool,
    pub restart_policy: RestartPolicy,
    pub network_mode: String,
    pub pid_mode: String,
    pub privileged: bool,
    pub readonly_rootfs: bool,
    pub runtime: String,
    pub security_opt: Vec<String>,
    pub storage_opt: HashMap<String, String>,
    pub sysctls: HashMap<String, String>,
    pub log_config: LogConfig,
    // ... many more fields
}
```

### Image

```rust
pub type ImageId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSummary {
    pub id: ImageId,
    pub short_id: String,
    pub repo_tags: Vec<String>,
    pub repo_digests: Vec<String>,
    pub created: DateTime<Utc>,
    pub size: i64,
    pub shared_size: i64,
    pub virtual_size: i64,
    pub labels: HashMap<String, String>,
    pub containers: i32,              // Number of containers using this image
    pub dangling: bool,               // <none>:<none>
    pub parent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDetails {
    pub summary: ImageSummary,
    pub architecture: String,
    pub author: String,
    pub comment: String,
    pub config: ImageConfig,
    pub container: String,
    pub container_config: ContainerConfig,
    pub created: String,
    pub docker_version: String,
    pub graph_driver: GraphDriverData,
    pub os: String,
    pub os_version: String,
    pub root_fs: RootFs,
    pub history: Vec<ImageHistoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub hostname: String,
    pub domainname: String,
    pub user: String,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub exposed_ports: HashMap<String, EmptyStruct>,
    pub tty: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
    pub env: Vec<String>,
    pub cmd: Vec<String>,
    pub healthcheck: Option<HealthConfig>,
    pub args_escaped: bool,
    pub image: String,
    pub volumes: HashMap<String, EmptyStruct>,
    pub working_dir: String,
    pub entrypoint: Vec<String>,
    pub network_disabled: bool,
    pub mac_address: String,
    pub on_build: Vec<String>,
    pub labels: HashMap<String, String>,
    pub stop_signal: String,
    pub stop_timeout: i32,
    pub shell: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageHistoryItem {
    pub created: DateTime<Utc>,
    pub created_by: String,
    pub empty_layer: bool,
    pub comment: String,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    pub typ: String,
    pub layers: Vec<String>,
    pub base_layer: String,
}
```

### Volume

```rust
pub type VolumeName = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSummary {
    pub name: VolumeName,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: DateTime<Utc>,
    pub status: HashMap<String, String>,
    pub labels: HashMap<String, String>,
    pub scope: VolumeScope,
    pub options: HashMap<String, String>,
    pub usage_data: Option<VolumeUsage>,
    pub in_use: Vec<ContainerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeScope {
    Local,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeUsage {
    pub size: i64,
    pub ref_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDetails {
    pub summary: VolumeSummary,
    pub contents: Option<Vec<FileEntry>>,  // If browsing enabled
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: i64,
    pub mode: u32,
    pub modified: DateTime<Utc>,
    pub is_dir: bool,
}
```

### Network

```rust
pub type NetworkId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub id: NetworkId,
    pub name: String,
    pub driver: String,
    pub scope: NetworkScope,
    pub created: DateTime<Utc>,
    pub internal: bool,
    pub attachable: bool,
    pub ingress: bool,
    pub config_only: bool,
    pub enable_ipv6: bool,
    pub ipam: Ipam,
    pub labels: HashMap<String, String>,
    pub options: HashMap<String, String>,
    pub connected_containers: Vec<NetworkConnection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkScope {
    Local,
    Global,
    Swarm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipam {
    pub driver: String,
    pub config: Vec<IpamConfig>,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpamConfig {
    pub subnet: String,
    pub ip_range: String,
    pub gateway: String,
    pub auxiliary_addresses: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub container_id: ContainerId,
    pub container_name: String,
    pub endpoint_id: String,
    pub mac_address: String,
    pub ipv4_address: String,
    pub ipv6_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub network_id: NetworkId,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
}
```

### Docker Compose

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeProject {
    pub name: String,
    pub directory: PathBuf,
    pub files: Vec<PathBuf>,
    pub services: Vec<ComposeService>,
    pub networks: Vec<String>,
    pub volumes: Vec<String>,
    pub status: ProjectStatus,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Running,
    Partial,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeService {
    pub name: String,
    pub container_name: Option<String>,
    pub image: String,
    pub build: Option<BuildConfig>,
    pub command: Option<Vec<String>>,
    pub environment: HashMap<String, String>,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<ServiceVolume>,
    pub networks: Vec<String>,
    pub depends_on: Vec<String>,
    pub healthcheck: Option<HealthConfig>,
    pub replicas: u32,
    pub restart: String,
    pub status: ServiceStatus,
    pub containers: Vec<ContainerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Running(u32),        // Count of running containers
    Starting,
    Stopped,
    Unhealthy,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub context: PathBuf,
    pub dockerfile: Option<String>,
    pub args: HashMap<String, String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceVolume {
    pub source: String,
    pub target: String,
    pub typ: VolumeType,
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeType {
    Volume,
    Bind,
    Tmpfs,
}
```

### Metrics and Statistics

```rust
/// Container statistics from Docker API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub read: DateTime<Utc>,
    pub pids_stats: PidsStats,
    pub networks: HashMap<String, NetworkStats>,
    pub memory_stats: MemoryStats,
    pub blkio_stats: BlkioStats,
    pub cpu_stats: CpuStats,
    pub precpu_stats: CpuStats,
    pub storage_stats: StorageStats,
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidsStats {
    pub current: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub usage: u64,
    pub max_usage: u64,
    pub limit: u64,
    pub stats: MemoryStatDetails,
    pub cache: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatDetails {
    pub active_anon: u64,
    pub active_file: u64,
    pub cache: u64,
    pub hierarchical_memory_limit: u64,
    pub inactive_anon: u64,
    pub inactive_file: u64,
    pub mapped_file: u64,
    pub pgfault: u64,
    pub pgmajfault: u64,
    pub pgpgin: u64,
    pub pgpgout: u64,
    pub rss: u64,
    pub rss_huge: u64,
    pub total_active_anon: u64,
    pub total_active_file: u64,
    pub total_cache: u64,
    pub total_inactive_anon: u64,
    pub total_inactive_file: u64,
    pub total_mapped_file: u64,
    pub total_pgfault: u64,
    pub total_pgmajfault: u64,
    pub total_pgpgin: u64,
    pub total_pgpgout: u64,
    pub total_rss: u64,
    pub total_rss_huge: u64,
    pub total_unevictable: u64,
    pub total_writeback: u64,
    pub unevictable: u64,
    pub writeback: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub cpu_usage: CpuUsage,
    pub system_cpu_usage: u64,
    pub online_cpus: u64,
    pub throttling_data: ThrottlingData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub total_usage: u64,
    pub percpu_usage: Vec<u64>,
    pub usage_in_kernelmode: u64,
    pub usage_in_usermode: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottlingData {
    pub periods: u64,
    pub throttled_periods: u64,
    pub throttled_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlkioStats {
    pub io_service_bytes_recursive: Vec<BlkioStatEntry>,
    pub io_serviced_recursive: Vec<BlkioStatEntry>,
    pub io_queue_recursive: Vec<BlkioStatEntry>,
    pub io_service_time_recursive: Vec<BlkioStatEntry>,
    pub io_wait_time_recursive: Vec<BlkioStatEntry>,
    pub io_merged_recursive: Vec<BlkioStatEntry>,
    pub io_time_recursive: Vec<BlkioStatEntry>,
    pub sectors_recursive: Vec<BlkioStatEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlkioStatEntry {
    pub major: u64,
    pub minor: u64,
    pub op: String,
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub read_count_normalized: u64,
    pub read_size_bytes: u64,
    pub write_count_normalized: u64,
    pub write_size_bytes: u64,
}

/// Processed metrics for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMetrics {
    pub timestamp: DateTime<Utc>,
    pub container_id: ContainerId,
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub network_rx_rate: f64,       // bytes per second
    pub network_tx_rate: f64,       // bytes per second
    pub block_read: u64,
    pub block_write: u64,
    pub pids: u64,
}

/// Time-series metrics store
#[derive(Debug, Clone)]
pub struct MetricsStore {
    pub retention: Duration,
    pub data: HashMap<ContainerId, Vec<ProcessedMetrics>>,
}

impl MetricsStore {
    pub fn new(retention_seconds: u64) -> Self {
        Self {
            retention: Duration::from_secs(retention_seconds),
            data: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, metrics: ProcessedMetrics) {
        let entry = self.data.entry(metrics.container_id.clone()).or_default();
        entry.push(metrics);
        self.cleanup_old(entry);
    }
    
    fn cleanup_old(&self, entry: &mut Vec<ProcessedMetrics>) {
        let cutoff = Utc::now() - self.retention;
        entry.retain(|m| m.timestamp > cutoff);
    }
    
    pub fn get_history(&self, container_id: &str) -> Option<&[ProcessedMetrics]> {
        self.data.get(container_id).map(|v| v.as_slice())
    }
}
```

### Logs

```rust
/// Log line with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub timestamp: DateTime<Utc>,
    pub source: LogSource,
    pub stream: StreamType,
    pub message: String,
    pub level: Option<LogLevel>,
    pub parsed: Option<ParsedLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSource {
    Container(ContainerId),
    Service { project: String, service: String },
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Unknown,
}

/// Structured log parsing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLog {
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Log buffer with ring buffer semantics
#[derive(Debug)]
pub struct LogBuffer {
    pub capacity: usize,
    pub buffer: VecDeque<LogLine>,
    pub filters: Vec<LogFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub field: FilterField,
    pub operation: FilterOp,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterField {
    Message,
    Container,
    Level,
    Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOp {
    Contains,
    Equals,
    Regex,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub regex: Regex,
    pub matches: Vec<usize>,      // Indices into buffer
    pub current_match: Option<usize>,
    pub case_sensitive: bool,
}
```

### Registry

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub name: String,
    pub url: String,
    pub auth: Option<RegistryAuth>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryAuth {
    pub username: String,
    pub password: Option<String>,  // Loaded from credential store
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSearchResult {
    pub name: String,
    pub description: String,
    pub is_official: bool,
    pub is_automated: bool,
    pub star_count: i32,
    pub pull_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTagInfo {
    pub name: String,
    pub size: i64,
    pub last_updated: DateTime<Utc>,
    pub digest: String,
    pub architecture: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVulnerability {
    pub cve_id: String,
    pub severity: VulnerabilitySeverity,
    pub description: String,
    pub package: String,
    pub fixed_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Negligible,
    Unknown,
}
```

### Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub docker: DockerConfig,
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub registries: Vec<Registry>,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub logging: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_metrics_retention")]
    pub metrics_retention_seconds: u64,
    #[serde(default = "default_log_tail")]
    pub default_log_tail: u64,
    #[serde(default)]
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub colors: CustomColors,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub mouse_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomColors {
    pub running: Option<String>,
    pub stopped: Option<String>,
    pub paused: Option<String>,
    pub healthy: Option<String>,
    pub unhealthy: Option<String>,
    pub selection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutConfig {
    pub sidebar_width: Option<u16>,
    pub log_panel_height: Option<u16>,
    pub show_header: bool,
    pub show_footer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub tls_verify: bool,
    #[serde(default)]
    pub cert_path: Option<PathBuf>,
    #[serde(default)]
    pub compose_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyBindings {
    pub global: HashMap<String, String>,
    pub containers: HashMap<String, String>,
    pub images: HashMap<String, String>,
    pub logs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    #[serde(default)]
    pub alerts_enabled: bool,
    #[serde(default)]
    pub cpu_threshold: Option<f64>,
    #[serde(default)]
    pub memory_threshold: Option<f64>,
    #[serde(default)]
    pub alert_cooldown_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default)]
    pub file: Option<PathBuf>,
    #[serde(default)]
    pub max_size_mb: u64,
    #[serde(default)]
    pub max_files: u32,
}

// Default functions for serde
fn default_poll_interval() -> u64 { 1000 }
fn default_metrics_retention() -> u64 { 3600 }
fn default_log_tail() -> u64 { 1000 }
fn default_theme() -> String { "dark".to_string() }
```

### UI State

```rust
#[derive(Debug, Clone)]
pub struct AppState {
    // Navigation
    pub current_tab: Tab,
    pub previous_tab: Option<Tab>,
    pub focused_panel: Panel,
    pub modal_stack: Vec<Modal>,
    
    // Data
    pub containers: Vec<ContainerSummary>,
    pub selected_container: Option<ContainerId>,
    pub container_sort: SortConfig,
    pub container_filter: String,
    
    pub images: Vec<ImageSummary>,
    pub selected_image: Option<ImageId>,
    pub image_sort: SortConfig,
    pub image_filter: String,
    
    pub volumes: Vec<VolumeSummary>,
    pub selected_volume: Option<VolumeName>,
    
    pub networks: Vec<NetworkSummary>,
    pub selected_network: Option<NetworkId>,
    
    pub compose_projects: Vec<ComposeProject>,
    pub selected_project: Option<String>,
    
    // Real-time data
    pub metrics: MetricsStore,
    pub log_buffer: LogBuffer,
    pub selected_streams: Vec<LogSource>,
    pub log_search: Option<SearchState>,
    
    // UI state
    pub show_help: bool,
    pub notifications: Vec<Notification>,
    pub loading: bool,
    pub last_error: Option<String>,
    
    // Async tracking
    pub pending_operations: HashMap<OperationId, Operation>,
    
    // Config
    pub config: Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Containers,
    Images,
    Volumes,
    Networks,
    Compose,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    List,
    Detail,
    Logs,
    Stats,
}

#[derive(Debug, Clone)]
pub enum Modal {
    Confirm(ConfirmDialog),
    Input(InputDialog),
    Search(SearchDialog),
    Help(HelpContent),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_action: Box<Action>,
    pub cancel_action: Option<Box<Action>>,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: Uuid,
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: DateTime<Utc>,
    pub auto_dismiss: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub id: OperationId,
    pub description: String,
    pub started_at: DateTime<Utc>,
    pub progress: Option<f64>,
}

pub type OperationId = Uuid;

#[derive(Debug, Clone)]
pub struct SortConfig {
    pub column: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}
```

## Type Conversions

### From Bollard Types

```rust
impl From<bollard::models::ContainerSummary> for ContainerSummary {
    fn from(c: bollard::models::ContainerSummary) -> Self {
        Self {
            id: c.id.unwrap_or_default(),
            short_id: c.id.as_ref()
                .map(|id| id.chars().take(12).collect())
                .unwrap_or_default(),
            names: c.names.unwrap_or_default()
                .iter()
                .map(|n| n.trim_start_matches('/').to_string())
                .collect(),
            image: c.image.unwrap_or_default(),
            image_id: c.image_id.unwrap_or_default(),
            command: c.command.unwrap_or_default(),
            created: DateTime::from_timestamp(c.created.unwrap_or_default(), 0)
                .unwrap_or_else(|| Utc::now()),
            ports: c.ports.unwrap_or_default()
                .into_iter()
                .filter_map(|p| p.try_into().ok())
                .collect(),
            // ...
        }
    }
}

impl From<bollard::models::ContainerState> for ContainerState {
    fn from(state: bollard::models::ContainerState) -> Self {
        match state.status {
            Some(ContainerStateStatusEnum::CREATED) => Self::Created,
            Some(ContainerStateStatusEnum::RUNNING) => Self::Running,
            Some(ContainerStateStatusEnum::PAUSED) => Self::Paused,
            Some(ContainerStateStatusEnum::RESTARTING) => Self::Restarting,
            Some(ContainerStateStatusEnum::REMOVING) => Self::Removing,
            Some(ContainerStateStatusEnum::EXITED) => Self::Exited,
            Some(ContainerStateStatusEnum::DEAD) => Self::Dead,
            _ => Self::Unknown,
        }
    }
}
```

### Metrics Calculation

```rust
impl ProcessedMetrics {
    pub fn from_docker_stats(stats: &ContainerStats, prev: Option<&ProcessedMetrics>) -> Self {
        let cpu_percent = calculate_cpu_percent(&stats.cpu_stats, &stats.precpu_stats);
        let memory_percent = calculate_memory_percent(&stats.memory_stats);
        
        let (network_rx_rate, network_tx_rate) = prev
            .map(|p| calculate_network_rates(p, stats))
            .unwrap_or((0.0, 0.0));
        
        Self {
            timestamp: stats.read,
            container_id: stats.id.clone(),
            cpu_percent,
            memory_usage: stats.memory_stats.usage.unwrap_or(0),
            memory_limit: stats.memory_stats.limit.unwrap_or(1),
            memory_percent,
            network_rx: stats.networks.values().map(|n| n.rx_bytes).sum(),
            network_tx: stats.networks.values().map(|n| n.tx_bytes).sum(),
            network_rx_rate,
            network_tx_rate,
            block_read: stats.blkio_stats.io_service_bytes_recursive
                .iter()
                .filter(|e| e.op == "Read")
                .map(|e| e.value)
                .sum(),
            block_write: stats.blkio_stats.io_service_bytes_recursive
                .iter()
                .filter(|e| e.op == "Write")
                .map(|e| e.value)
                .sum(),
            pids: stats.pids_stats.current.unwrap_or(0),
        }
    }
}

fn calculate_cpu_percent(current: &CpuStats, previous: &CpuStats) -> f64 {
    let cpu_delta = current.cpu_usage.total_usage - previous.cpu_usage.total_usage;
    let system_delta = current.system_cpu_usage - previous.system_cpu_usage;
    
    if system_delta > 0 && cpu_delta > 0 {
        let cpu_count = current.online_cpus as f64;
        (cpu_delta as f64 / system_delta as f64) * cpu_count * 100.0
    } else {
        0.0
    }
}
```

## Validation

```rust
pub trait Validatable {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl Validatable for Config {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate poll interval
        if self.general.poll_interval_ms < 100 {
            return Err(ValidationError {
                field: "poll_interval_ms".to_string(),
                message: "Poll interval must be at least 100ms".to_string(),
            });
        }
        
        // Validate thresholds
        if let Some(cpu) = self.monitoring.cpu_threshold {
            if !(0.0..=100.0).contains(&cpu) {
                return Err(ValidationError {
                    field: "cpu_threshold".to_string(),
                    message: "CPU threshold must be between 0 and 100".to_string(),
                });
            }
        }
        
        Ok(())
    }
}
```
