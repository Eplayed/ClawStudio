// enterprise/mod.rs - Enterprise Edition Interface Stubs
// Open-Core Architecture: Community Edition implements local-only versions
// Enterprise Edition (closed-source) can replace these with cloud implementations

pub mod audit_logger;
pub mod sandbox_provider;
pub mod auth_provider;
pub mod storage_backend;
pub mod screenshot_store;

// Re-export traits for use across the codebase
pub use audit_logger::AuditLogger;
pub use sandbox_provider::SandboxProvider;
pub use auth_provider::AuthProvider;
pub use storage_backend::StorageBackend;
pub use screenshot_store::ScreenshotStore;