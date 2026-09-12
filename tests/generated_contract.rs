use meta_signal_aggregator::{
    AggregatorConfiguration, ConfigurationChange, ConfigurationObservation, ConfigurationObserved,
    ConfigurationRejectionReason, DefaultingPolicy, OperationKind, OutputInterfaceConfiguration,
    Query, Response, TranscriptRoot, TranscriptSource,
};
use signal::{ByteViewable, Restorable, Signal, Signalizable};
use signal_aggregator::{LimitPolicy, Projection};

fn configuration() -> AggregatorConfiguration {
    AggregatorConfiguration {
        ordinary_socket_path: String::from("/run/aggregator/ordinary.socket"),
        ordinary_socket_mode: 0o660,
        meta_socket_path: String::from("/run/aggregator/meta.socket"),
        meta_socket_mode: 0o600,
        store_path: String::from("/var/lib/aggregator"),
        active_repositories: Vec::new(),
        transcript_sources: vec![TranscriptSource::Claude(TranscriptRoot {
            filesystem_path: String::from("/home/li/.claude/projects"),
        })],
        default_projection: Projection::MetadataOnly,
        default_limit_policy: LimitPolicy {
            maximum_segments: 64,
            maximum_bytes: 65_536,
        },
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
            reader: ReaderBudget { remaining: 65536 },
            depth: 0,
            maximum_depth: 256,
        })
        .expect("actualize");
    assert_eq!(restored, query);
}
