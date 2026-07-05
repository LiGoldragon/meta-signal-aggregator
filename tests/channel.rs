use meta_signal_aggregator::{
    ActiveRepository, AggregatorConfiguration, ConfigurationCandidate, ConfigurationChange,
    ConfigurationRejected, ConfigurationRejectionReason, ConfigurationValidated,
    ConfigurationValidationOutcome, FilesystemPath, MetaAggregatorFrame, MetaAggregatorFrameBody,
    MetaAggregatorOperationKind, MetaAggregatorReply, MetaAggregatorRequest, ObserveConfiguration,
    RepositoryName, SocketMode, TranscriptRoot, TranscriptSource,
};
use nota::{NotaDecode, NotaEncode, NotaSource};
use signal_aggregator::{ByteLimit, LimitPolicy, Projection, SegmentLimit};
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
    }
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

const EXPECTED_SCHEMA_SKETCH: &str = "{}\n\n[\n  (Configure [ConfigurationChange])\n  (ObserveConfiguration [ObserveConfiguration])\n  (ValidateConfiguration [ConfigurationCandidate])\n]\n\n[\n  (ConfigurationConfigured [ConfigurationConfigured])\n  (ConfigurationObserved [ConfigurationObserved])\n  (ConfigurationValidated [ConfigurationValidated])\n  (ConfigurationRejected [ConfigurationRejected])\n]\n\n[]\n\n{\n  AggregatorConfiguration (FilesystemPath SocketMode FilesystemPath SocketMode FilesystemPath [ActiveRepository] [TranscriptSource] Projection LimitPolicy)\n  ConfigurationChange (AggregatorConfiguration)\n  ObserveConfiguration (?ConfigurationObserver)\n  ConfigurationCandidate (AggregatorConfiguration)\n  ActiveRepository (RepositoryName FilesystemPath)\n  TranscriptRoot (FilesystemPath)\n  TranscriptSource [(Claude TranscriptRoot) (Codex TranscriptRoot) (Pi TranscriptRoot)]\n  SocketMode (u32)\n  ConfigurationValidated (ConfigurationValidationOutcome)\n  ConfigurationValidationOutcome [Accepted Rejected]\n  ConfigurationRejected (OperationKind ConfigurationRejectionReason)\n}\n\n[\n  (Version 0 1)\n  (Status Scaffold)\n]\n";

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
            "ConfigurationRejected",
        ],
    }
    .assert_matches_contract();
}
