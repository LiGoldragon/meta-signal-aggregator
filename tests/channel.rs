use meta_signal_aggregator::{
    ActiveRepository, AggregatorConfiguration, ByteLimit, ConfigurationCandidate,
    ConfigurationChange, ConfigurationRejected, ConfigurationRejectionReason,
    ConfigurationValidated, ConfigurationValidationIssue, ConfigurationValidationIssueKind,
    ConfigurationValidationOutcome, ConfigurationValidationReport, DurableFragileIndexPolicy,
    DurableFragileIndexStorage, FilesystemPath, FragileReferencePolicy, ItemCount,
    LegacyRecoveryAccess, LegacyRecoveryRoot, LegacyRecoverySource, MetaAggregatorFrame,
    MetaAggregatorFrameBody, MetaAggregatorOperationKind, MetaAggregatorReply,
    MetaAggregatorRequest, ObserveConfiguration, OutputInterfaceConfiguration,
    OutputInterfaceLimitPolicy, PageLimit, RepositoryName, SocketMode, StableOrderingTieBreaker,
    TranscriptRoot, TranscriptSource, ValidationIssueDetail,
};
use nota::{NotaDecode, NotaEncode, NotaSource};
use signal_aggregator::{LimitPolicy, Projection, SegmentLimit};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SignalOperationHeads, SubReply,
};

fn configuration() -> AggregatorConfiguration {
    AggregatorConfiguration {
        ordinary_socket_path: FilesystemPath::new("/run/aggregator/aggregator.sock"),
        ordinary_socket_mode: SocketMode::new(0o660),
        meta_socket_path: FilesystemPath::new("/run/aggregator/aggregator-meta.sock"),
        meta_socket_mode: SocketMode::new(0o600),
        store_path: FilesystemPath::new("/var/lib/aggregator/aggregator.sema"),
        active_repositories: vec![ActiveRepository {
            name: RepositoryName::new("primary"),
            path: FilesystemPath::new("/home/li/primary"),
        }],
        transcript_sources: vec![TranscriptSource::Claude(TranscriptRoot {
            path: FilesystemPath::new("/home/li/.claude/projects"),
        })],
        default_projection: Projection::MetadataOnly,
        default_limit_policy: LimitPolicy {
            maximum_segments: SegmentLimit::new(32),
            maximum_bytes: ByteLimit::new(4096),
        },
        output_interfaces: OutputInterfaceConfiguration::default(),
    }
}

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(0),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn round_trip_request(request: MetaAggregatorRequest) -> MetaAggregatorRequest {
    let frame = MetaAggregatorFrame::new(MetaAggregatorFrameBody::Request {
        exchange: exchange(),
        request: request.clone().into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = MetaAggregatorFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        MetaAggregatorFrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request, got {other:?}"),
    }
}

fn round_trip_reply(reply_payload: MetaAggregatorReply) -> MetaAggregatorReply {
    let frame = MetaAggregatorFrame::new(MetaAggregatorFrameBody::Reply {
        exchange: exchange(),
        reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply_payload.clone()))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = MetaAggregatorFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        MetaAggregatorFrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected ok reply, got {other:?}"),
            },
            Reply::Rejected { reason } => panic!("unexpected rejected reply: {reason:?}"),
        },
        other => panic!("expected reply, got {other:?}"),
    }
}

fn round_trip_nota<Value>(value: Value)
where
    Value: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let text = value.to_nota();
    let decoded = NotaSource::new(&text).parse::<Value>().expect("decode");
    assert_eq!(decoded, value);
}

enum CanonicalExample {
    Request(MetaAggregatorRequest),
    Reply(MetaAggregatorReply),
}

impl CanonicalExample {
    fn assert_matches_line(&self, line: &str) {
        match self {
            Self::Request(expected) => {
                let decoded = NotaSource::new(line)
                    .parse::<MetaAggregatorRequest>()
                    .expect("canonical request decode");
                assert_eq!(&decoded, expected, "canonical request decode for {line}");
                assert_eq!(decoded.to_nota(), line, "canonical request encode");
            }
            Self::Reply(expected) => {
                let decoded = NotaSource::new(line)
                    .parse::<MetaAggregatorReply>()
                    .expect("canonical reply decode");
                assert_eq!(&decoded, expected, "canonical reply decode for {line}");
                assert_eq!(decoded.to_nota(), line, "canonical reply encode");
            }
        }
    }
}

fn canonical_example_lines() -> Vec<&'static str> {
    include_str!("../examples/canonical.nota")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn canonical_configuration() -> AggregatorConfiguration {
    AggregatorConfiguration {
        ordinary_socket_path: FilesystemPath::new("/run/aggregator/aggregator.sock"),
        ordinary_socket_mode: SocketMode::new(432),
        meta_socket_path: FilesystemPath::new("/run/aggregator/aggregator-meta.sock"),
        meta_socket_mode: SocketMode::new(384),
        store_path: FilesystemPath::new("/var/lib/aggregator/aggregator.sema"),
        active_repositories: vec![],
        transcript_sources: vec![TranscriptSource::Claude(TranscriptRoot {
            path: FilesystemPath::new("/home/li/.claude/projects"),
        })],
        default_projection: Projection::MetadataOnly,
        default_limit_policy: LimitPolicy {
            maximum_segments: SegmentLimit::new(32),
            maximum_bytes: ByteLimit::new(4096),
        },
        output_interfaces: OutputInterfaceConfiguration::default(),
    }
}

#[test]
fn output_interface_defaults_keep_index_daemon_local_and_legacy_roots_disabled() {
    let output_interfaces = OutputInterfaceConfiguration::default();
    assert_eq!(
        output_interfaces.fragile_index,
        DurableFragileIndexPolicy {
            storage: DurableFragileIndexStorage::DaemonLocalStorePath,
            references: FragileReferencePolicy::OpaqueStaleCapable,
            ordering_tie_breaker: StableOrderingTieBreaker::FragileReferenceAscending,
        }
    );
    assert_eq!(
        output_interfaces.limits,
        OutputInterfaceLimitPolicy {
            maximum_page_items: PageLimit::new(64),
            maximum_preview_bytes: ByteLimit::new(4096),
            maximum_read_bytes: ByteLimit::new(65_536),
            maximum_recovery_files_per_root: ItemCount::new(1024),
            maximum_transcript_scan_entries: ItemCount::new(131_072),
            maximum_transcript_discovered_files: ItemCount::new(32_768),
            maximum_transcript_file_bytes: ByteLimit::new(8 * 1024 * 1024),
            maximum_transcript_line_bytes: ByteLimit::new(256 * 1024),
            maximum_transcript_read_failures: ItemCount::new(1024),
        }
    );
    assert!(
        output_interfaces.legacy_recovery_sources.is_empty(),
        "legacy reports and agent outputs are opt-in recovery roots"
    );
}

#[test]
fn legacy_recovery_roots_round_trip_as_read_only_sources() {
    let output_interfaces = OutputInterfaceConfiguration {
        legacy_recovery_sources: vec![
            LegacyRecoverySource::LegacyReports(LegacyRecoveryRoot {
                path: FilesystemPath::new("/home/li/primary/reports"),
                access: LegacyRecoveryAccess::ReadOnlyRecovery,
            }),
            LegacyRecoverySource::LegacyAgentOutputs(LegacyRecoveryRoot {
                path: FilesystemPath::new("/home/li/primary/agent-outputs"),
                access: LegacyRecoveryAccess::ReadOnlyRecovery,
            }),
        ],
        ..OutputInterfaceConfiguration::default()
    };
    round_trip_nota(output_interfaces);
}

#[test]
fn output_interface_validation_rejections_round_trip_through_nota() {
    round_trip_nota(MetaAggregatorReply::ConfigurationValidated(
        ConfigurationValidated {
            outcome: ConfigurationValidationOutcome::Rejected(ConfigurationValidationReport {
                issues: vec![
                    ConfigurationValidationIssue {
                        path: None,
                        kind: ConfigurationValidationIssueKind::InvalidFragileIndexConfiguration,
                        detail: Some(ValidationIssueDetail::new(
                            "fragile index policy must be daemon-local durable storage",
                        )),
                    },
                    ConfigurationValidationIssue {
                        path: Some(FilesystemPath::new("/var/lib/aggregator/aggregator.sema")),
                        kind: ConfigurationValidationIssueKind::UnwritableFragileIndexStorage,
                        detail: Some(ValidationIssueDetail::new(
                            "durable fragile index store must be writable",
                        )),
                    },
                    ConfigurationValidationIssue {
                        path: Some(FilesystemPath::new("/home/li/primary/reports")),
                        kind: ConfigurationValidationIssueKind::InvalidLegacyRecoveryRoot,
                        detail: Some(ValidationIssueDetail::new(
                            "legacy roots are read-only recovery inputs",
                        )),
                    },
                ],
            }),
        },
    ));
}

#[test]
fn configure_request_round_trips_through_frame() {
    let request = MetaAggregatorRequest::Configure(ConfigurationChange {
        configuration: configuration(),
    });
    assert_eq!(round_trip_request(request.clone()), request);
}

#[test]
fn observe_configuration_request_round_trips_through_nota() {
    round_trip_nota(MetaAggregatorRequest::ObserveConfiguration(
        ObserveConfiguration { observer: None },
    ));
}

#[test]
fn validate_configuration_request_round_trips_through_frame() {
    let request = MetaAggregatorRequest::ValidateConfiguration(ConfigurationCandidate {
        configuration: configuration(),
    });
    assert_eq!(round_trip_request(request.clone()), request);
}

#[test]
fn validation_reply_round_trips_through_frame() {
    let reply = MetaAggregatorReply::ConfigurationValidated(ConfigurationValidated {
        outcome: ConfigurationValidationOutcome::Accepted,
    });
    assert_eq!(round_trip_reply(reply.clone()), reply);
}

#[test]
fn rejection_reply_round_trips_through_nota() {
    round_trip_nota(MetaAggregatorReply::ConfigurationRejected(
        ConfigurationRejected {
            operation: MetaAggregatorOperationKind::Configure,
            reason: ConfigurationRejectionReason::InvalidConfiguration,
        },
    ));
}

#[test]
fn canonical_examples_match_file_order_and_boundaries() {
    let expected_examples = [
        CanonicalExample::Request(MetaAggregatorRequest::Configure(ConfigurationChange {
            configuration: canonical_configuration(),
        })),
        CanonicalExample::Request(MetaAggregatorRequest::ObserveConfiguration(
            ObserveConfiguration { observer: None },
        )),
        CanonicalExample::Request(MetaAggregatorRequest::ValidateConfiguration(
            ConfigurationCandidate {
                configuration: canonical_configuration(),
            },
        )),
        CanonicalExample::Reply(MetaAggregatorReply::ConfigurationValidated(
            ConfigurationValidated {
                outcome: ConfigurationValidationOutcome::Accepted,
            },
        )),
        CanonicalExample::Reply(MetaAggregatorReply::ConfigurationRejected(
            ConfigurationRejected {
                operation: MetaAggregatorOperationKind::Configure,
                reason: ConfigurationRejectionReason::InvalidConfiguration,
            },
        )),
    ];
    let actual_lines = canonical_example_lines();
    assert_eq!(
        actual_lines.len(),
        expected_examples.len(),
        "canonical example count changed"
    );
    for (expected, line) in expected_examples.iter().zip(actual_lines) {
        expected.assert_matches_line(line);
    }
}

#[test]
fn meta_operations_are_configuration_only() {
    assert_eq!(
        <MetaAggregatorRequest as SignalOperationHeads>::HEADS,
        &["Configure", "ObserveConfiguration", "ValidateConfiguration",]
    );
    assert_eq!(
        MetaAggregatorRequest::Configure(ConfigurationChange {
            configuration: configuration(),
        })
        .operation_kind(),
        MetaAggregatorOperationKind::Configure
    );
}

const EXPECTED_SCHEMA_SKETCH: &str = r#"{}

[
  (Configure [ConfigurationChange])
  (ObserveConfiguration [ObserveConfiguration])
  (ValidateConfiguration [ConfigurationCandidate])
]

[
  (ConfigurationConfigured [ConfigurationConfigured])
  (ConfigurationObserved [ConfigurationObserved])
  (ConfigurationValidated [ConfigurationValidated])
  (ConfigurationRejected [ConfigurationRejected])
]

[]

{
  AggregatorConfiguration (FilesystemPath SocketMode FilesystemPath SocketMode FilesystemPath [ActiveRepository] [TranscriptSource] Projection LimitPolicy OutputInterfaceConfiguration)
  ConfigurationChange (AggregatorConfiguration)
  ObserveConfiguration (?ConfigurationObserver)
  ConfigurationCandidate (AggregatorConfiguration)
  ActiveRepository (RepositoryName FilesystemPath)
  TranscriptRoot (FilesystemPath)
  TranscriptSource [(Claude TranscriptRoot) (ClaudeSubagentOutput TranscriptRoot) (Codex TranscriptRoot) (Pi TranscriptRoot)]
  OutputInterfaceConfiguration (DurableFragileIndexPolicy OutputInterfaceLimitPolicy [LegacyRecoverySource])
  DurableFragileIndexPolicy (DurableFragileIndexStorage FragileReferencePolicy StableOrderingTieBreaker)
  DurableFragileIndexStorage [DaemonLocalStorePath]
  FragileReferencePolicy [OpaqueStaleCapable]
  StableOrderingTieBreaker [FragileReferenceAscending]
  OutputInterfaceLimitPolicy (PageLimit ByteLimit ByteLimit ItemCount)
  LegacyRecoverySource [(LegacyReports LegacyRecoveryRoot) (LegacyAgentOutputs LegacyRecoveryRoot)]
  LegacyRecoveryRoot (FilesystemPath LegacyRecoveryAccess)
  LegacyRecoveryAccess [ReadOnlyRecovery]
  SocketMode (u32)
  ConfigurationValidated (ConfigurationValidationOutcome)
  ConfigurationValidationOutcome [Accepted (Rejected ConfigurationValidationReport)]
  ConfigurationValidationReport ([ConfigurationValidationIssue])
  ConfigurationValidationIssue (?FilesystemPath ConfigurationValidationIssueKind ?ValidationIssueDetail)
  ConfigurationValidationIssueKind [MissingTranscriptSource MissingRepository UnreadablePath InvalidSocketMode MissingFragileIndexConfiguration InvalidFragileIndexConfiguration UnwritableFragileIndexStorage InvalidOutputInterfaceLimit InvalidLegacyRecoveryRoot]
  ConfigurationRejected (OperationKind ConfigurationRejectionReason)
}

[
  (Version 0 2)
  (Status Scaffold)
]
"#;

struct SchemaSketchWitness {
    full_text: &'static str,
    expected_operation_heads: &'static [&'static str],
    expected_reply_heads: &'static [&'static str],
    expected_data_heads: &'static [&'static str],
}

impl SchemaSketchWitness {
    fn assert_matches_contract(self) {
        assert_eq!(
            self.full_text, EXPECTED_SCHEMA_SKETCH,
            "schema sketch drifted; update the complete manual witness with any intentional schema change"
        );
        assert_eq!(
            <MetaAggregatorRequest as SignalOperationHeads>::HEADS,
            self.expected_operation_heads,
            "exported operation heads drifted from the schema sketch"
        );
        for head in self.expected_reply_heads {
            assert!(
                self.full_text.contains(&format!("  ({head} [")),
                "schema sketch is missing reply head {head}"
            );
        }
        for head in self.expected_data_heads {
            assert!(
                self.full_text.contains(&format!("  {head} ")),
                "schema sketch is missing configuration head {head}"
            );
        }
        assert!(
            self.full_text.ends_with("  (Status Scaffold)\n]\n"),
            "schema sketch scaffold status drifted"
        );
    }
}

#[test]
fn schema_sketch_matches_complete_manual_contract_witness() {
    SchemaSketchWitness {
        full_text: include_str!("../schema/meta-signal.schema"),
        expected_operation_heads: &["Configure", "ObserveConfiguration", "ValidateConfiguration"],
        expected_reply_heads: &[
            "ConfigurationConfigured",
            "ConfigurationObserved",
            "ConfigurationValidated",
            "ConfigurationRejected",
        ],
        expected_data_heads: &[
            "AggregatorConfiguration",
            "ConfigurationChange",
            "ObserveConfiguration",
            "ConfigurationCandidate",
            "ConfigurationValidated",
            "ConfigurationValidationOutcome",
            "ConfigurationValidationReport",
            "ConfigurationValidationIssue",
            "ConfigurationValidationIssueKind",
            "ConfigurationRejected",
            "OutputInterfaceConfiguration",
            "DurableFragileIndexPolicy",
            "DurableFragileIndexStorage",
            "FragileReferencePolicy",
            "StableOrderingTieBreaker",
            "OutputInterfaceLimitPolicy",
            "LegacyRecoverySource",
            "LegacyRecoveryRoot",
            "LegacyRecoveryAccess",
        ],
    }
    .assert_matches_contract();
}
