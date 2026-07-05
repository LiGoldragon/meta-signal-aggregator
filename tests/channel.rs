use meta_signal_aggregator::{
    ActiveRepository, AggregatorConfiguration, ConfigurationCandidate, ConfigurationChange,
    ConfigurationRejected, ConfigurationRejectionReason, ConfigurationValidated,
    ConfigurationValidationOutcome, FilesystemPath, MetaAggregatorFrame, MetaAggregatorFrameBody,
    MetaAggregatorOperationKind, MetaAggregatorReply, MetaAggregatorRequest, ObserveConfiguration,
    RepositoryName, SocketMode, TranscriptFormat, TranscriptSource, TranscriptSourceKind,
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
        transcript_sources: vec![TranscriptSource {
            source_kind: TranscriptSourceKind::Claude,
            path: FilesystemPath::new("/home/li/.claude/projects"),
            format: TranscriptFormat::ClaudeJsonl,
        }],
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

fn assert_canonical_nota<Value>(value: Value, canonical_text: &str)
where
    Value: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let encoded = value.to_nota();
    assert_eq!(encoded, canonical_text, "canonical encode for {value:?}");
    let decoded = NotaSource::new(canonical_text)
        .parse::<Value>()
        .expect("canonical decode");
    assert_eq!(decoded, value, "canonical decode for {canonical_text}");
    assert!(
        include_str!("../examples/canonical.nota").contains(canonical_text),
        "examples/canonical.nota missing line: {canonical_text}"
    );
}

fn canonical_configuration() -> AggregatorConfiguration {
    AggregatorConfiguration {
        ordinary_socket_path: FilesystemPath::new("/run/aggregator/aggregator.sock"),
        ordinary_socket_mode: SocketMode::new(432),
        meta_socket_path: FilesystemPath::new("/run/aggregator/aggregator-meta.sock"),
        meta_socket_mode: SocketMode::new(384),
        store_path: FilesystemPath::new("/var/lib/aggregator/aggregator.sema"),
        active_repositories: vec![],
        transcript_sources: vec![],
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
fn canonical_request_examples_round_trip() {
    assert_canonical_nota(
        MetaAggregatorRequest::Configure(ConfigurationChange {
            configuration: canonical_configuration(),
        }),
        "(Configure ((/run/aggregator/aggregator.sock 432 /run/aggregator/aggregator-meta.sock 384 /var/lib/aggregator/aggregator.sema [] [] MetadataOnly (32 4096))))",
    );
    assert_canonical_nota(
        MetaAggregatorRequest::ObserveConfiguration(ObserveConfiguration { observer: None }),
        "(ObserveConfiguration (None))",
    );
    assert_canonical_nota(
        MetaAggregatorRequest::ValidateConfiguration(ConfigurationCandidate {
            configuration: canonical_configuration(),
        }),
        "(ValidateConfiguration ((/run/aggregator/aggregator.sock 432 /run/aggregator/aggregator-meta.sock 384 /var/lib/aggregator/aggregator.sema [] [] MetadataOnly (32 4096))))",
    );
}

#[test]
fn canonical_reply_examples_round_trip() {
    assert_canonical_nota(
        MetaAggregatorReply::ConfigurationValidated(ConfigurationValidated {
            outcome: ConfigurationValidationOutcome::Accepted,
        }),
        "(ConfigurationValidated (Accepted))",
    );
    assert_canonical_nota(
        MetaAggregatorReply::ConfigurationRejected(ConfigurationRejected {
            operation: MetaAggregatorOperationKind::Configure,
            reason: ConfigurationRejectionReason::InvalidConfiguration,
        }),
        "(ConfigurationRejected (Configure InvalidConfiguration))",
    );
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
