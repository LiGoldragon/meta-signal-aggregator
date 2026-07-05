//! Meta Signal contract for aggregator configuration.
//!
//! This crate carries configuration operations only. Collection and storage
//! live in the `aggregator` runtime crate.

use nota::{NotaDecode, NotaEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
pub use signal_aggregator::{LimitPolicy, Projection};
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

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEncode, NotaDecode, Debug, Clone, PartialEq, Eq,
)]
pub enum ConfigurationObservation {
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
