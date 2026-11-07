use meta_signal_aggregator::{
    ActiveRepositories, AggregatorConfiguration, ConfigurationChange, ConfigurationObservation,
    ConfigurationObserved, ConfigurationRejectionReason, DefaultLimitPolicy, DefaultProjection,
    DefaultingPolicy, FilesystemPath, MetaSocketMode, MetaSocketPath, OperationKind,
    OrdinarySocketMode, OrdinarySocketPath, OutputInterfaceConfiguration, Query, Response,
    SocketMode, StorePath, TranscriptRoot, TranscriptSource, TranscriptSources,
};
use signal::{ByteViewable, Restorable, Signal, Signalizable};
use signal_aggregator::{
    ByteLimit, LimitPolicy, MaximumBytes, MaximumSegments, Projection, SegmentLimit,
};

fn configuration() -> AggregatorConfiguration {
    AggregatorConfiguration {
        ordinary_socket_path: OrdinarySocketPath(FilesystemPath(
            "/run/aggregator/ordinary.socket".to_string(),
        )),
        ordinary_socket_mode: OrdinarySocketMode(SocketMode(0o660)),
        meta_socket_path: MetaSocketPath(FilesystemPath("/run/aggregator/meta.socket".to_string())),
        meta_socket_mode: MetaSocketMode(SocketMode(0o600)),
        store_path: StorePath(FilesystemPath("/var/lib/aggregator".to_string())),
        active_repositories: ActiveRepositories(Vec::new()),
        transcript_sources: TranscriptSources(vec![TranscriptSource::Claude(TranscriptRoot {
            filesystem_path: FilesystemPath("/home/li/.claude/projects".to_string()),
        })]),
        default_projection: DefaultProjection(Projection::MetadataOnly),
        default_limit_policy: DefaultLimitPolicy(LimitPolicy {
            maximum_segments: MaximumSegments(SegmentLimit(64)),
            maximum_bytes: MaximumBytes(ByteLimit(65_536)),
        }),
        output_interface_configuration: OutputInterfaceConfiguration::default_policy(),
    }
}

#[test]
fn meta_query_restores_from_fresh_peer_bytes() {
    let query = Query::Configure(ConfigurationChange {
        aggregator_configuration: configuration(),
    });
    let outgoing = query.signalize().expect("archive query");
    assert!(!outgoing.bytes().is_empty());
    let incoming = Signal::<Query>::from(outgoing.bytes().to_vec());
    assert_eq!(incoming.restore().expect("restore query"), query);
}

#[test]
fn meta_response_restores_from_fresh_peer_bytes() {
    let response = Response::ConfigurationObserved(ConfigurationObserved {
        configuration_observation: ConfigurationObservation::Configured(configuration()),
    });
    let outgoing = response.signalize().expect("archive response");
    let incoming = Signal::<Response>::from(outgoing.bytes().to_vec());
    assert_eq!(incoming.restore().expect("restore response"), response);
}

#[test]
fn a_rejection_names_the_operation_it_refuses() {
    let response = Response::ConfigurationRejected(meta_signal_aggregator::ConfigurationRejected {
        operation_kind: OperationKind::ValidateConfiguration,
        configuration_rejection_reason: ConfigurationRejectionReason::InvalidConfiguration,
    });
    let outgoing = response.signalize().expect("archive response");
    let incoming = Signal::<Response>::from(outgoing.bytes().to_vec());
    assert_eq!(incoming.restore().expect("restore response"), response);
}

#[cfg(feature = "datom")]
#[test]
fn datom_round_trip_preserves_the_configuration() {
    use datom_codec::{Actualizing, Budget, Datomizable, Potential};
    use protos::{Protosizable, ReaderBudget, Textualizable};
    let query = Query::Configure(ConfigurationChange {
        aggregator_configuration: configuration(),
    });
    let rendered = query.clone().datomize(vec![]).protosize().textualize();
    let mut pending = Potential::<Query>::from(rendered);
    let restored = pending
        .actualize(&mut Budget {
            remaining: 65536,
            reader: ReaderBudget::try_from(65536).expect("bounded reader budget"),
            depth: 0,
            maximum_depth: 256,
        })
        .expect("actualize");
    assert_eq!(restored, query);
}
