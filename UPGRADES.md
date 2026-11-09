# 0.9.1 — ethos-zero 22.0.0, bare redundant outputs

Not breaking: the generated Rust and the public API are unchanged. The
build-time ethos-zero is repinned from 5a5daca (19.2.0) to ccacc21 (22.0.0),
and the redundant Signal outputs it refuses as `Payload` are written bare:
`ConfigurationConfigured.ConfigurationConfigured` becomes
`ConfigurationConfigured`, and its three siblings likewise, carrying the
same types. The build-time ethos-zero brings datom-codec 5cd6219 (0.33.1)
into the lock beside the selected eabead4 (0.33.0); the library's own graph
is unchanged.

# 0.9.0 — first protos 0.33 text-cohort adoption

The authored Ethos and build script are unchanged. Generated aliases become
tuple newtypes; configuration fixtures and the library's default-policy
construction now wrap exactly the same strings, limits and empty vectors.
The nine default limit values and their policy choices are unchanged.

Selected cohort: datom-codec eabead4 (0.33.0), protos 3c7ade3 (0.33.0),
Signal 7afb68b (10.3.0), ordinary signal-aggregator 60bcde3 (1.1.0), and
ethos-zero 5a5daca (19.2.0). The selected graph has one codec/protos/Signal
identity, replacing the historical split codec/protos graph. Compatible
registry versions are preserved by `cargo update --workspace`.

Historical baseline f7e9f4ff57f10da0f436ceeddc9aadcc6418f97c (0.8.0) runs
its own original manifest, lock, source and tools. The candidate runs its
selected cohort. Scratch copies add only witness-test suffixes. Exact
source/graph linkage is in `/tmp/01e4a4-runtime/meta-aggregator-source-linkage.json`;
raw outputs, source, witness/cross-read logs and exits use the
`meta-aggregator-{old,new}` names there. `link-witness.py` and
`compare-witness.py` reproduce the linkage/extraction/token inspection.

Eight fixtures cover all three Query roots and all four Response roots,
with both observed-configuration states. They include the default policies.
The raw archives and existing Signal frame bytes match on eight of eight;
all frames restore locally. Both sides restore the counterpart archives and
parse the counterpart printed text to the same typed values. Raw text
matches on four of eight. Separate exact-token and balanced-string-content
inspection verifies whitespace-layout-only differences on the other four,
independently of typed cross-reading. No compact-print normalization is
added to these tests; their Datom round trip remains structural.

Root-variant coverage is not exhaustive coverage of nested variants,
configuration values, boundaries, malformed inputs, source compatibility,
storage consumers or running deployments.
