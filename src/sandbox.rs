use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Represents specific capabilities that can be granted to sandboxed code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Allow reading from a specific file path
    FileRead(PathBuf),
    /// Allow writing to a specific file path
    FileWrite(PathBuf),
    /// Allow HTTP network requests
    NetworkHTTP,
    /// Allow accessing system time
    SystemTime,
    /// Allow spawning child processes
    ProcessSpawn,
    /// Allow using unsafe Rust features
    UnsafeRust,
}

/// Configuration for the sandbox execution environment
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Allowed file paths for read/write operations
    pub allowed_file_paths: Vec<PathBuf>,
    /// Whether network access is permitted
    pub permitted_network_access: bool,
    /// Set of allowed Rust standard library APIs
    pub safe_rust_apis: HashSet<String>,
    /// Set of granted capabilities
    pub capabilities: HashSet<Capability>,
}

impl SandboxConfig {
    /// Create a new sandbox configuration with default strict settings
    pub fn new() -> Self {
        SandboxConfig {
            max_memory: 100 * 1024 * 1024, // 100MB default
            max_execution_time: Duration::from_secs(30), // 30 seconds default
            allowed_file_paths: Vec::new(),
            permitted_network_access: false,
            safe_rust_apis: Self::default_safe_apis(),
            capabilities: HashSet::new(),
        }
    }

    /// Returns the default set of safe Rust APIs
    fn default_safe_apis() -> HashSet<String> {
        let mut apis = HashSet::new();
        // Core safe operations
        apis.insert("std::println".to_string());
        apis.insert("std::print".to_string());
        apis.insert("std::format".to_string());
        apis.insert("std::vec::Vec".to_string());
        apis.insert("std::string::String".to_string());
        apis.insert("std::collections::HashMap".to_string());
        apis.insert("std::collections::HashSet".to_string());
        apis.insert("std::option::Option".to_string());
        apis.insert("std::result::Result".to_string());
        // Math operations
        apis.insert("std::cmp".to_string());
        apis.insert("std::ops".to_string());
        apis
    }

    /// Add a capability to the sandbox
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities.insert(capability);
    }

    /// Check if a capability is granted
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Set maximum memory limit in bytes
    pub fn with_max_memory(mut self, bytes: usize) -> Self {
        self.max_memory = bytes;
        self
    }

    /// Set maximum execution time
    pub fn with_max_execution_time(mut self, duration: Duration) -> Self {
        self.max_execution_time = duration;
        self
    }

    /// Add an allowed file path
    pub fn allow_file_path(mut self, path: PathBuf) -> Self {
        self.allowed_file_paths.push(path);
        self
    }

    /// Enable network access
    pub fn with_network_access(mut self, enabled: bool) -> Self {
        self.permitted_network_access = enabled;
        self
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a violation of sandbox security boundaries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxViolation {
    /// Exceeded maximum memory limit
    MemoryLimitExceeded {
        limit: usize,
        attempted: usize,
    },
    /// Exceeded maximum execution time
    ExecutionTimeExceeded {
        limit: Duration,
        elapsed: Duration,
    },
    /// Attempted to access unauthorized file path
    UnauthorizedFileAccess {
        path: PathBuf,
    },
    /// Attempted network access without permission
    UnauthorizedNetworkAccess,
    /// Attempted to use unsafe Rust feature without permission
    UnsafeRustNotPermitted,
    /// Attempted to spawn process without permission
    ProcessSpawnNotPermitted,
    /// Attempted to use disallowed API
    DisallowedAPIUsage {
        api: String,
    },
    /// Missing required capability
    MissingCapability {
        capability: Capability,
    },
}

impl std::fmt::Display for SandboxViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxViolation::MemoryLimitExceeded { limit, attempted } => {
                write!(
                    f,
                    "Memory limit exceeded: limit={} bytes, attempted={} bytes",
                    limit, attempted
                )
            }
            SandboxViolation::ExecutionTimeExceeded { limit, elapsed } => {
                write!(
                    f,
                    "Execution time exceeded: limit={:?}, elapsed={:?}",
                    limit, elapsed
                )
            }
            SandboxViolation::UnauthorizedFileAccess { path } => {
                write!(f, "Unauthorized file access: {}", path.display())
            }
            SandboxViolation::UnauthorizedNetworkAccess => {
                write!(f, "Unauthorized network access attempted")
            }
            SandboxViolation::UnsafeRustNotPermitted => {
                write!(f, "Unsafe Rust features not permitted in sandbox mode")
            }
            SandboxViolation::ProcessSpawnNotPermitted => {
                write!(f, "Process spawning not permitted in sandbox mode")
            }
            SandboxViolation::DisallowedAPIUsage { api } => {
                write!(f, "Disallowed API usage: {}", api)
            }
            SandboxViolation::MissingCapability { capability } => {
                write!(f, "Missing required capability: {:?}", capability)
            }
        }
    }
}

impl std::error::Error for SandboxViolation {}

/// Runtime monitor for sandbox execution
pub struct SandboxMonitor {
    config: SandboxConfig,
    start_time: Instant,
    current_memory: usize,
}

impl SandboxMonitor {
    /// Create a new sandbox monitor with the given configuration
    pub fn new(config: SandboxConfig) -> Self {
        SandboxMonitor {
            config,
            start_time: Instant::now(),
            current_memory: 0,
        }
    }

    /// Check if execution time limit has been exceeded
    pub fn check_time_limit(&self) -> Result<(), SandboxViolation> {
        let elapsed = self.start_time.elapsed();
        if elapsed > self.config.max_execution_time {
            return Err(SandboxViolation::ExecutionTimeExceeded {
                limit: self.config.max_execution_time,
                elapsed,
            });
        }
        Ok(())
    }

    /// Check if memory limit would be exceeded by allocation
    pub fn check_memory_limit(&self, allocation_size: usize) -> Result<(), SandboxViolation> {
        let new_total = self.current_memory + allocation_size;
        if new_total > self.config.max_memory {
            return Err(SandboxViolation::MemoryLimitExceeded {
                limit: self.config.max_memory,
                attempted: new_total,
            });
        }
        Ok(())
    }

    /// Record a memory allocation
    pub fn allocate_memory(&mut self, size: usize) -> Result<(), SandboxViolation> {
        self.check_memory_limit(size)?;
        self.current_memory += size;
        Ok(())
    }

    /// Record a memory deallocation
    pub fn deallocate_memory(&mut self, size: usize) {
        self.current_memory = self.current_memory.saturating_sub(size);
    }

    /// Check if file path access is allowed
    pub fn check_file_access(&self, path: &PathBuf) -> Result<(), SandboxViolation> {
        // Check if path matches any allowed paths
        for allowed_path in &self.config.allowed_file_paths {
            if path.starts_with(allowed_path) {
                return Ok(());
            }
        }
        Err(SandboxViolation::UnauthorizedFileAccess { path: path.clone() })
    }

    /// Check if a capability is granted
    pub fn check_capability(&self, capability: &Capability) -> Result<(), SandboxViolation> {
        if self.config.has_capability(capability) {
            Ok(())
        } else {
            Err(SandboxViolation::MissingCapability {
                capability: capability.clone(),
            })
        }
    }

    /// Check if an API is allowed
    pub fn check_api_usage(&self, api: &str) -> Result<(), SandboxViolation> {
        if self.config.safe_rust_apis.contains(api) {
            Ok(())
        } else {
            Err(SandboxViolation::DisallowedAPIUsage {
                api: api.to_string(),
            })
        }
    }

    /// Get current memory usage
    pub fn current_memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Get elapsed execution time
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::new();
        assert_eq!(config.max_memory, 100 * 1024 * 1024);
        assert_eq!(config.max_execution_time, Duration::from_secs(30));
        assert!(!config.permitted_network_access);
        assert!(config.allowed_file_paths.is_empty());
        assert!(!config.safe_rust_apis.is_empty());
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = SandboxConfig::new()
            .with_max_memory(50 * 1024 * 1024)
            .with_max_execution_time(Duration::from_secs(10))
            .with_network_access(true)
            .allow_file_path(PathBuf::from("/tmp"));

        assert_eq!(config.max_memory, 50 * 1024 * 1024);
        assert_eq!(config.max_execution_time, Duration::from_secs(10));
        assert!(config.permitted_network_access);
        assert_eq!(config.allowed_file_paths.len(), 1);
    }

    #[test]
    fn test_capability_management() {
        let mut config = SandboxConfig::new();
        let cap = Capability::FileRead(PathBuf::from("/tmp/test.txt"));

        assert!(!config.has_capability(&cap));
        config.add_capability(cap.clone());
        assert!(config.has_capability(&cap));
    }

    #[test]
    fn test_sandbox_monitor_time_limit() {
        let config = SandboxConfig::new()
            .with_max_execution_time(Duration::from_millis(1));
        let monitor = SandboxMonitor::new(config);

        // Immediate check should pass
        assert!(monitor.check_time_limit().is_ok());

        // After delay, should fail
        std::thread::sleep(Duration::from_millis(10));
        assert!(monitor.check_time_limit().is_err());
    }

    #[test]
    fn test_sandbox_monitor_memory_limit() {
        let config = SandboxConfig::new().with_max_memory(1000);
        let mut monitor = SandboxMonitor::new(config);

        // Allocation within limit should succeed
        assert!(monitor.allocate_memory(500).is_ok());
        assert_eq!(monitor.current_memory_usage(), 500);

        // Allocation exceeding limit should fail
        assert!(monitor.allocate_memory(600).is_err());
        assert_eq!(monitor.current_memory_usage(), 500);

        // Deallocation should work
        monitor.deallocate_memory(200);
        assert_eq!(monitor.current_memory_usage(), 300);
    }

    #[test]
    fn test_file_access_check() {
        let config = SandboxConfig::new()
            .allow_file_path(PathBuf::from("/tmp"));
        let monitor = SandboxMonitor::new(config);

        // Allowed path should pass
        assert!(monitor.check_file_access(&PathBuf::from("/tmp/test.txt")).is_ok());

        // Disallowed path should fail
        assert!(monitor.check_file_access(&PathBuf::from("/etc/passwd")).is_err());
    }

    #[test]
    fn test_capability_check() {
        let mut config = SandboxConfig::new();
        config.add_capability(Capability::SystemTime);
        let monitor = SandboxMonitor::new(config);

        // Granted capability should pass
        assert!(monitor.check_capability(&Capability::SystemTime).is_ok());

        // Missing capability should fail
        assert!(monitor.check_capability(&Capability::NetworkHTTP).is_err());
    }

    #[test]
    fn test_api_usage_check() {
        let config = SandboxConfig::new();
        let monitor = SandboxMonitor::new(config);

        // Safe API should pass
        assert!(monitor.check_api_usage("std::println").is_ok());

        // Unsafe API should fail
        assert!(monitor.check_api_usage("std::fs::remove_file").is_err());
    }

    #[test]
    fn test_sandbox_violation_display() {
        let violation = SandboxViolation::MemoryLimitExceeded {
            limit: 1000,
            attempted: 1500,
        };
        let message = format!("{}", violation);
        assert!(message.contains("Memory limit exceeded"));
        assert!(message.contains("1000"));
        assert!(message.contains("1500"));
    }

    #[test]
    fn test_memory_allocation_tracking() {
        let config = SandboxConfig::new().with_max_memory(1000);
        let mut monitor = SandboxMonitor::new(config);

        // Multiple allocations
        assert!(monitor.allocate_memory(300).is_ok());
        assert!(monitor.allocate_memory(400).is_ok());
        assert_eq!(monitor.current_memory_usage(), 700);

        // Should still have room
        assert!(monitor.allocate_memory(200).is_ok());
        assert_eq!(monitor.current_memory_usage(), 900);

        // Exceeding limit
        assert!(monitor.allocate_memory(200).is_err());
        assert_eq!(monitor.current_memory_usage(), 900);
    }

    #[test]
    fn test_default_safe_apis() {
        let config = SandboxConfig::new();
        assert!(config.safe_rust_apis.contains("std::println"));
        assert!(config.safe_rust_apis.contains("std::vec::Vec"));
        assert!(config.safe_rust_apis.contains("std::string::String"));
    }
}
