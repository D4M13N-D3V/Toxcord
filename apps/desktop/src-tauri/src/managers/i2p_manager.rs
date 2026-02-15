//! I2P Router Manager
//!
//! Embeds an I2P router (emissary) into Toxcord for anonymous networking.
//! When enabled, all Tox traffic is routed through I2P tunnels.
//!
//! ## Usage
//!
//! Enable the `i2p` feature in Cargo.toml:
//! ```toml
//! toxcord = { features = ["i2p"] }
//! ```
//!
//! ## Limitations
//!
//! - I2P adds significant latency (~2-5 seconds per hop)
//! - Outproxy support for clearnet access may be limited
//! - For best results, use with I2P-native Tox bootstrap nodes

use std::path::PathBuf;
#[cfg(feature = "i2p")]
use std::sync::Arc;
#[cfg(feature = "i2p")]
use tracing::{info, warn};
#[cfg(not(feature = "i2p"))]
use tracing::warn;

#[cfg(feature = "i2p")]
use emissary_core::{Config, SamConfig, router::Router};
#[cfg(feature = "i2p")]
use emissary_util::{
    runtime::tokio::Runtime as EmissaryRuntime,
    storage::Storage,
};

/// I2P router configuration
#[derive(Clone, Debug)]
pub struct I2pConfig {
    /// Data directory for I2P router state
    pub data_dir: PathBuf,
    /// SAM TCP port (0 = auto-assign)
    pub sam_tcp_port: u16,
    /// SOCKS proxy port to expose
    pub socks_port: u16,
    /// Enable floodfill mode (helps the network but uses more bandwidth)
    pub floodfill: bool,
}

impl Default for I2pConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("toxcord")
            .join("i2p");

        Self {
            data_dir,
            sam_tcp_port: 0,  // Auto-assign
            socks_port: 4447, // Default emissary SOCKS port
            floodfill: false,
        }
    }
}

/// Manages the embedded I2P router
pub struct I2pManager {
    config: I2pConfig,
    /// The actual SOCKS port after binding (may differ if auto-assigned)
    socks_port: u16,
    /// Router shutdown handle
    #[cfg(feature = "i2p")]
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl I2pManager {
    /// Create a new I2P manager with default configuration
    pub fn new() -> Self {
        Self::with_config(I2pConfig::default())
    }

    /// Create a new I2P manager with custom configuration
    pub fn with_config(config: I2pConfig) -> Self {
        Self {
            socks_port: config.socks_port,
            config,
            #[cfg(feature = "i2p")]
            shutdown_tx: None,
        }
    }

    /// Get the SOCKS proxy port for Tox to connect through
    pub fn socks_port(&self) -> u16 {
        self.socks_port
    }

    /// Start the I2P router
    ///
    /// This is an async operation that spawns the router in a background task.
    /// The router will continue running until `shutdown()` is called.
    #[cfg(feature = "i2p")]
    pub async fn start(&mut self) -> Result<(), String> {
        info!("Starting embedded I2P router...");

        // Ensure data directory exists
        std::fs::create_dir_all(&self.config.data_dir)
            .map_err(|e| format!("Failed to create I2P data directory: {e}"))?;

        // Create storage for router state
        let storage = Storage::new(Some(self.config.data_dir.clone()))
            .await
            .map_err(|e| format!("Failed to create I2P storage: {e}"))?;

        // Build router configuration
        let config = self.build_config();

        // Create the router
        let (mut router, _events, router_info) = Router::<EmissaryRuntime>::new(
            config,
            None,
            Some(Arc::new(storage)),
        )
        .await
        .map_err(|e| format!("Failed to create I2P router: {e}"))?;

        info!("I2P router created, router info size: {} bytes", router_info.len());

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn router in background task
        tokio::spawn(async move {
            tokio::select! {
                _ = &mut router => {
                    info!("I2P router stopped");
                }
                _ = &mut shutdown_rx => {
                    info!("I2P router shutdown requested");
                    router.shutdown();
                }
            }
        });

        info!("I2P router started, SOCKS proxy available on 127.0.0.1:{}", self.socks_port);
        Ok(())
    }

    /// Start the I2P router (no-op when i2p feature is disabled)
    #[cfg(not(feature = "i2p"))]
    pub async fn start(&mut self) -> Result<(), String> {
        warn!("I2P support not compiled in. Enable the 'i2p' feature to use embedded I2P.");
        Err("I2P feature not enabled".to_string())
    }

    /// Shutdown the I2P router gracefully
    #[cfg(feature = "i2p")]
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            info!("I2P router shutdown signal sent");
        }
    }

    /// Shutdown (no-op when i2p feature is disabled)
    #[cfg(not(feature = "i2p"))]
    pub fn shutdown(&mut self) {}

    /// Build emissary Config from our I2pConfig
    #[cfg(feature = "i2p")]
    fn build_config(&self) -> Config {
        Config {
            samv3_config: Some(SamConfig {
                tcp_port: self.config.sam_tcp_port,
                udp_port: 0,
                host: "127.0.0.1".to_string(),
            }),
            // Use defaults for other settings
            ..Default::default()
        }
    }

    /// Check if the I2P router is running
    #[cfg(feature = "i2p")]
    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }

    /// Check if running (always false when i2p feature is disabled)
    #[cfg(not(feature = "i2p"))]
    pub fn is_running(&self) -> bool {
        false
    }

    /// Log the current I2P router status for verification
    #[cfg(feature = "i2p")]
    pub fn log_status(&self) {
        if self.is_running() {
            info!("[I2P-CHECK] I2P router RUNNING - SOCKS proxy available at 127.0.0.1:{}", self.socks_port);
        } else {
            warn!("[I2P-CHECK] I2P router NOT running - traffic will NOT be anonymized");
        }
    }

    /// Log status (no-op when i2p feature is disabled)
    #[cfg(not(feature = "i2p"))]
    pub fn log_status(&self) {
        warn!("[I2P-CHECK] I2P support not compiled in - traffic will NOT be anonymized");
    }
}

impl Default for I2pManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for I2pManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Check if I2P support is compiled in
pub fn is_i2p_available() -> bool {
    cfg!(feature = "i2p")
}
