//! Meta Signal contract for aggregator configuration.
//!
//! The contract carries configuration operations only. Collection and storage
//! live in the `aggregator` runtime crate.
//!
//! `ethos/signal.ethos` is the schema authority; `build.rs` checks the
//! checked-in Rust projection in `src/generated/signal.rs` against it. The
//! portable rkyv frame, its kinds, and the wire framing come from `signal`.

pub mod generated;
pub use generated::signal::*;

pub use signal_aggregator::{ByteLimit, ItemCount, LimitPolicy, PageLimit, Projection};

/// The authored Ethos source of this contract.
pub const ETHOS: &str = include_str!("../ethos/signal.ethos");
/// The Rust projection generated from [`ETHOS`].
pub const ETHOS_RUST: &str = include_str!("generated/signal.rs");

/// The ceilings a runtime applies when the configuration names none.
pub trait DefaultingPolicy {
    fn default_policy() -> Self;
}

impl DefaultingPolicy for DurableFragileIndexPolicy {
    fn default_policy() -> Self {
        Self {
            durable_fragile_index_storage: DurableFragileIndexStorage::DaemonLocalStorePath,
            fragile_reference_policy: FragileReferencePolicy::OpaqueStaleCapable,
            stable_ordering_tie_breaker: StableOrderingTieBreaker::FragileReferenceAscending,
        }
    }
}

impl DefaultingPolicy for OutputInterfaceLimitPolicy {
    fn default_policy() -> Self {
        Self {
            maximum_page_items: 64,
            maximum_preview_bytes: 4096,
            maximum_read_bytes: 65_536,
            maximum_recovery_files_per_root: 1024,
            maximum_transcript_scan_entries: 131_072,
            maximum_transcript_discovered_files: 32_768,
            maximum_transcript_file_bytes: 8 * 1024 * 1024,
            maximum_transcript_line_bytes: 256 * 1024,
            maximum_transcript_read_failures: 1024,
        }
    }
}

impl DefaultingPolicy for OutputInterfaceConfiguration {
    fn default_policy() -> Self {
        Self {
            durable_fragile_index_policy: DurableFragileIndexPolicy::default_policy(),
            output_interface_limit_policy: OutputInterfaceLimitPolicy::default_policy(),
            legacy_recovery_sources: Vec::new(),
        }
    }
}
