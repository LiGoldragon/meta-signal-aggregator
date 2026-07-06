//! Meta Signal contract for aggregator configuration.
//!
//! This crate carries configuration operations only. Collection and storage
//! live in the `aggregator` runtime crate.

use nota::{NotaDecode, NotaEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
pub use signal_aggregator::{ByteLimit, ItemCount, LimitPolicy, PageLimit, Projection};
use signal_frame::signal_channel;

macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(
            Archive,
            RkyvSerialize,
            RkyvDeserialize,
            NotaEncode,
            NotaDecode,
            Debug,
            Clone,
            PartialEq,
            Eq,
            Hash,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

macro_rules! mode_newtype {
    ($name:ident, $inner:ty, $getter:ident) => {
        #[derive(
            Archive,
            RkyvSerialize,
            RkyvDeserialize,
            NotaEncode,
            NotaDecode,
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
        )]
        pub struct $name($inner);

        impl $name {
            pub fn new(value: $inner) -> Self {
                Self(value)
            }

            pub fn $getter(self) -> $inner {
                self.0
            }
        }
    };
}

string_newtype!(FilesystemPath);
string_newtype!(RepositoryName);
string_newtype!(ConfigurationObserver);
string_newtype!(ValidationIssueDetail);
mode_newtype!(SocketMode, u32, into_u32);

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ActiveRepository {
    pub name: RepositoryName,
    pub path: FilesystemPath,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct TranscriptRoot {
    pub path: FilesystemPath,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum TranscriptSource {
    Claude(TranscriptRoot),
    Codex(TranscriptRoot),
    Pi(TranscriptRoot),
}

/// Fine-controlled output interfaces use a durable daemon-local index whose
/// references remain fragile because backing transcript and artifact files can
/// move, change, or disappear.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
)]
pub struct OutputInterfaceConfiguration {
    pub fragile_index: DurableFragileIndexPolicy,
    pub limits: OutputInterfaceLimitPolicy,
    pub legacy_recovery_sources: Vec<LegacyRecoverySource>,
}

/// Policy for the aggregator-owned opaque index. The policy is durable, but
/// the references it produces are explicitly stale-capable.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
)]
pub struct DurableFragileIndexPolicy {
    pub storage: DurableFragileIndexStorage,
    pub references: FragileReferencePolicy,
    pub ordering_tie_breaker: StableOrderingTieBreaker,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum DurableFragileIndexStorage {
    #[default]
    /// Persist the index in the daemon-local store named by
    /// [`AggregatorConfiguration::store_path`], never in repositories or legacy
    /// recovery roots.
    DaemonLocalStorePath,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum FragileReferencePolicy {
    #[default]
    /// References are daemon-local opaque handles. Runtime readers must reject
    /// stale or broken references instead of promising stable file identity.
    OpaqueStaleCapable,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum StableOrderingTieBreaker {
    #[default]
    /// After the requested listing order, break ties by opaque fragile
    /// reference so pagination stays deterministic for an unchanged index.
    FragileReferenceAscending,
}

/// Runtime ceilings for output listing, preview, read, and legacy recovery
/// source discovery. Exact read ranges are still enforced by the runtime.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct OutputInterfaceLimitPolicy {
    pub maximum_page_items: PageLimit,
    pub maximum_preview_bytes: ByteLimit,
    pub maximum_read_bytes: ByteLimit,
    pub maximum_recovery_files_per_root: ItemCount,
}

impl Default for OutputInterfaceLimitPolicy {
    fn default() -> Self {
        Self {
            maximum_page_items: PageLimit::new(64),
            maximum_preview_bytes: ByteLimit::new(4096),
            maximum_read_bytes: ByteLimit::new(65_536),
            maximum_recovery_files_per_root: ItemCount::new(1024),
        }
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum LegacyRecoverySource {
    LegacyReports(LegacyRecoveryRoot),
    LegacyAgentOutputs(LegacyRecoveryRoot),
}

/// Optional legacy source root. These roots are read-only recovery inputs and
/// are not source-of-truth design surfaces.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct LegacyRecoveryRoot {
    pub path: FilesystemPath,
    pub access: LegacyRecoveryAccess,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum LegacyRecoveryAccess {
    ReadOnlyRecovery,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct AggregatorConfiguration {
    pub ordinary_socket_path: FilesystemPath,
    pub ordinary_socket_mode: SocketMode,
    pub meta_socket_path: FilesystemPath,
    pub meta_socket_mode: SocketMode,
    pub store_path: FilesystemPath,
    pub active_repositories: Vec<ActiveRepository>,
    pub transcript_sources: Vec<TranscriptSource>,
    pub default_projection: Projection,
    pub default_limit_policy: LimitPolicy,
    pub output_interfaces: OutputInterfaceConfiguration,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationChange {
    pub configuration: AggregatorConfiguration,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ObserveConfiguration {
    pub observer: Option<ConfigurationObserver>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationCandidate {
    pub configuration: AggregatorConfiguration,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationConfigured {
    pub configuration: AggregatorConfiguration,
}

#[allow(clippy::large_enum_variant)]
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum ConfigurationObservation {
    /// The meta observation reply intentionally carries the full configuration
    /// inline so its NOTA and rkyv shape matches the configured value.
    Configured(AggregatorConfiguration),
    NotConfigured,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationObserved {
    pub observation: ConfigurationObservation,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum ConfigurationValidationIssueKind {
    MissingTranscriptSource,
    MissingRepository,
    UnreadablePath,
    InvalidSocketMode,
    MissingFragileIndexConfiguration,
    InvalidFragileIndexConfiguration,
    UnwritableFragileIndexStorage,
    InvalidOutputInterfaceLimit,
    InvalidLegacyRecoveryRoot,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationValidationIssue {
    pub path: Option<FilesystemPath>,
    pub kind: ConfigurationValidationIssueKind,
    pub detail: Option<ValidationIssueDetail>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationValidationReport {
    pub issues: Vec<ConfigurationValidationIssue>,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum ConfigurationValidationOutcome {
    Accepted,
    Rejected(ConfigurationValidationReport),
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationValidated {
    pub outcome: ConfigurationValidationOutcome,
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaEncode,
    NotaDecode,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum ConfigurationRejectionReason {
    InvalidConfiguration,
    StoreUnavailable,
    NotAuthorized,
    NotInPrototypeScope,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub struct ConfigurationRejected {
    pub operation: OperationKind,
    pub reason: ConfigurationRejectionReason,
}

signal_channel! {
    channel MetaAggregator {
        operation Configure(ConfigurationChange),
        operation ObserveConfiguration(ObserveConfiguration),
        operation ValidateConfiguration(ConfigurationCandidate),
    }
    reply MetaAggregatorReply {
        ConfigurationConfigured(ConfigurationConfigured),
        ConfigurationObserved(ConfigurationObserved),
        ConfigurationValidated(ConfigurationValidated),
        ConfigurationRejected(ConfigurationRejected),
    }
}

pub type MetaAggregatorRequest = Operation;
pub type MetaAggregatorOperationKind = OperationKind;
pub type MetaAggregatorFrame = Frame;
pub type MetaAggregatorFrameBody = FrameBody;
pub type MetaAggregatorReplyEnvelope = ReplyEnvelope;

impl MetaAggregatorRequest {
    pub fn operation_kind(&self) -> MetaAggregatorOperationKind {
        self.kind()
    }
}
