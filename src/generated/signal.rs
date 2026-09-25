#![allow(dead_code, non_camel_case_types, non_snake_case)]
#[rustfmt::skip]
pub type FilesystemPath = String;
#[rustfmt::skip]
pub type RepositoryName = String;
#[rustfmt::skip]
pub type ConfigurationObserver = String;
#[rustfmt::skip]
pub type ValidationIssueDetail = String;
#[rustfmt::skip]
pub type SocketMode = i64;
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum OperationKind {
    Configure,
    ObserveConfiguration,
    ValidateConfiguration,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ActiveRepository {
    pub repository_name: RepositoryName,
    pub filesystem_path: FilesystemPath,
}
#[rustfmt::skip]
pub type ActiveRepositories = std::vec::Vec<ActiveRepository>;
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct TranscriptRoot {
    pub filesystem_path: FilesystemPath,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum TranscriptSource {
    Claude(TranscriptRoot),
    ClaudeSubagentOutput(TranscriptRoot),
    Codex(TranscriptRoot),
    Pi(TranscriptRoot),
    PiSubagentOutput(TranscriptRoot),
}
#[rustfmt::skip]
pub type TranscriptSources = std::vec::Vec<TranscriptSource>;
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum DurableFragileIndexStorage {
    DaemonLocalStorePath,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum FragileReferencePolicy {
    OpaqueStaleCapable,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum StableOrderingTieBreaker {
    FragileReferenceAscending,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct DurableFragileIndexPolicy {
    pub durable_fragile_index_storage: DurableFragileIndexStorage,
    pub fragile_reference_policy: FragileReferencePolicy,
    pub stable_ordering_tie_breaker: StableOrderingTieBreaker,
}
#[rustfmt::skip]
pub type MaximumPageItems = signal_aggregator::PageLimit;
#[rustfmt::skip]
pub type MaximumPreviewBytes = signal_aggregator::ByteLimit;
#[rustfmt::skip]
pub type MaximumReadBytes = signal_aggregator::ByteLimit;
#[rustfmt::skip]
pub type MaximumRecoveryFilesPerRoot = signal_aggregator::ItemCount;
#[rustfmt::skip]
pub type MaximumTranscriptScanEntries = signal_aggregator::ItemCount;
#[rustfmt::skip]
pub type MaximumTranscriptDiscoveredFiles = signal_aggregator::ItemCount;
#[rustfmt::skip]
pub type MaximumTranscriptFileBytes = signal_aggregator::ByteLimit;
#[rustfmt::skip]
pub type MaximumTranscriptLineBytes = signal_aggregator::ByteLimit;
#[rustfmt::skip]
pub type MaximumTranscriptReadFailures = signal_aggregator::ItemCount;
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct OutputInterfaceLimitPolicy {
    pub maximum_page_items: MaximumPageItems,
    pub maximum_preview_bytes: MaximumPreviewBytes,
    pub maximum_read_bytes: MaximumReadBytes,
    pub maximum_recovery_files_per_root: MaximumRecoveryFilesPerRoot,
    pub maximum_transcript_scan_entries: MaximumTranscriptScanEntries,
    pub maximum_transcript_discovered_files: MaximumTranscriptDiscoveredFiles,
    pub maximum_transcript_file_bytes: MaximumTranscriptFileBytes,
    pub maximum_transcript_line_bytes: MaximumTranscriptLineBytes,
    pub maximum_transcript_read_failures: MaximumTranscriptReadFailures,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum LegacyRecoveryAccess {
    ReadOnlyRecovery,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct LegacyRecoveryRoot {
    pub filesystem_path: FilesystemPath,
    pub legacy_recovery_access: LegacyRecoveryAccess,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum LegacyRecoverySource {
    LegacyReports(LegacyRecoveryRoot),
    LegacyAgentOutputs(LegacyRecoveryRoot),
}
#[rustfmt::skip]
pub type LegacyRecoverySources = std::vec::Vec<LegacyRecoverySource>;
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct OutputInterfaceConfiguration {
    pub durable_fragile_index_policy: DurableFragileIndexPolicy,
    pub output_interface_limit_policy: OutputInterfaceLimitPolicy,
    pub legacy_recovery_sources: LegacyRecoverySources,
}
#[rustfmt::skip]
pub type OrdinarySocketPath = FilesystemPath;
#[rustfmt::skip]
pub type OrdinarySocketMode = SocketMode;
#[rustfmt::skip]
pub type MetaSocketPath = FilesystemPath;
#[rustfmt::skip]
pub type MetaSocketMode = SocketMode;
#[rustfmt::skip]
pub type StorePath = FilesystemPath;
#[rustfmt::skip]
pub type DefaultProjection = signal_aggregator::Projection;
#[rustfmt::skip]
pub type DefaultLimitPolicy = signal_aggregator::LimitPolicy;
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct AggregatorConfiguration {
    pub ordinary_socket_path: OrdinarySocketPath,
    pub ordinary_socket_mode: OrdinarySocketMode,
    pub meta_socket_path: MetaSocketPath,
    pub meta_socket_mode: MetaSocketMode,
    pub store_path: StorePath,
    pub active_repositories: ActiveRepositories,
    pub transcript_sources: TranscriptSources,
    pub default_projection: DefaultProjection,
    pub default_limit_policy: DefaultLimitPolicy,
    pub output_interface_configuration: OutputInterfaceConfiguration,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationChange {
    pub aggregator_configuration: AggregatorConfiguration,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationObservationQuery {
    pub configuration_observer_option: std::option::Option<ConfigurationObserver>,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationCandidate {
    pub aggregator_configuration: AggregatorConfiguration,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationConfigured {
    pub aggregator_configuration: AggregatorConfiguration,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum ConfigurationObservation {
    Configured(AggregatorConfiguration),
    NotConfigured,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationObserved {
    pub configuration_observation: ConfigurationObservation,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
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
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationValidationIssue {
    pub filesystem_path_option: std::option::Option<FilesystemPath>,
    pub configuration_validation_issue_kind: ConfigurationValidationIssueKind,
    pub validation_issue_detail_option: std::option::Option<ValidationIssueDetail>,
}
#[rustfmt::skip]
pub type ConfigurationValidationIssues = std::vec::Vec<ConfigurationValidationIssue>;
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationValidationReport {
    pub configuration_validation_issues: ConfigurationValidationIssues,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum ConfigurationValidationOutcome {
    Accepted,
    Rejected(ConfigurationValidationReport),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationValidated {
    pub configuration_validation_outcome: ConfigurationValidationOutcome,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum ConfigurationRejectionReason {
    InvalidConfiguration,
    StoreUnavailable,
    NotAuthorized,
    NotInPrototypeScope,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ConfigurationRejected {
    pub operation_kind: OperationKind,
    pub configuration_rejection_reason: ConfigurationRejectionReason,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Query {
    Configure(ConfigurationChange),
    ObserveConfiguration(ConfigurationObservationQuery),
    ValidateConfiguration(ConfigurationCandidate),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Response {
    ConfigurationConfigured(ConfigurationConfigured),
    ConfigurationObserved(ConfigurationObserved),
    ConfigurationValidated(ConfigurationValidated),
    ConfigurationRejected(ConfigurationRejected),
}
