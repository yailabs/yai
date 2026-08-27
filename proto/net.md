# NET Protocol Vocabulary

Authority: compatibility vocabulary for the current versioned NET scaffold.
It does not override YAI's canonical [semantics](../docs/reference/semantics.md)
or [execution boundaries](../docs/reference/boundaries.md), and it is not proof
that a transport/runtime capability exists.

The word “canonical” below is local to a named NET schema/fixture version, not
to YAI domain ontology.

YAI controls authority.
NET moves streams.
External nodes execute capabilities.

CLORI extension:
YAI controls authority.
NET moves streams.
CLORI executes neural computation.

## Terms

- net module: Integrated YAI runtime communication module. It owns stream movement surfaces; it does not own YAI authority, policy or execution eligibility.
- node: A local or external execution participant addressable by NET vocabulary. A node advertises capability; it does not grant YAI authority.
- local node: Node projected inside the local YAI process boundary. It does not imply external service lifecycle ownership.
- localhost service: Service reachable through loopback posture. It is not a production transport contract or approval path.
- LAN node: Node projected on a local network. It is not discovered until a later discovery wave.
- remote node: Node projected beyond localhost/LAN posture. It is not trusted by placement alone.
- external node: Node outside YAI that may execute advertised capabilities. It does not become part of YAI authority.
- endpoint: Addressable contact surface for a node. An endpoint does not approve operations or imply liveness.
- capability: Description of what an external node can do. A capability does not say the operation is allowed.
- stream: Ordered movement channel for invocation events. A stream does not decide policy.
- stream event: Named item in a stream such as request, chunk, metric or receipt. It is transport vocabulary, not authority.
- invocation: Correlated request lifecycle through NET surfaces. It is not a YAI decision record by itself.
- route: Selected communication path candidate. A route selects a communication path; it does not approve the operation.
- health: Observed service or node condition. Health does not authorize use.
- readiness: Condition that a node or endpoint may accept work. Readiness is not policy approval.
- liveness: Condition that a node or endpoint appears alive. Liveness is not readiness and not authority.
- trace: Correlation vocabulary for stream movement and boundaries. It is not a tracing backend.
- receipt transport: Movement of receipt material through NET. A receipt can be transported by NET; NET does not make it authoritative.
- metric: Measurement vocabulary for NET movement and waiting. A metric is not a control decision.
- transport adapter: Future adapter shape for a communication path. It does not implement sockets, servers or discovery in this wave.
- NET-compatible node: External node target that can speak the NET contract surface. Compatibility does not grant trust.
- CLORI-compatible node target: External neural computation target aligned with CLORI compatibility. CLORI remains external and is not vendored into YAI.

## Stream Envelope

A stream envelope carries event material through NET stream surfaces. A stream
envelope does not approve an operation. A receipt event carried by NET is not
made authoritative by NET. Ordering is represented by sequence number, not by
filesystem order. Transport adapters may carry stream envelopes later, but this
wave does not implement adapters.

- stream envelope: Versioned event container for stream movement. It is not a policy decision.
- stream id: Stable stream correlation id. It does not imply storage ownership.
- invocation id: Correlation id for an invocation lifecycle. It is not authorization.
- trace id: Correlation id for trace propagation. It is not a tracing backend.
- sequence number: Integer event ordering value within a stream. Filesystem order is not ordering truth.
- stream event kind: One of request, response, chunk, metric, receipt, error or complete. It names event shape, not permission.
- payload kind: One of none, bytes, text, json, metric, receipt or error. It does not define serialization law beyond this scaffold.
- timestamp: Millisecond Unix timestamp supplied as envelope metadata. It is not trust proof.
- origin node: Node id that emitted the envelope. It does not grant authority.
- target node: Node id intended to receive the envelope. It does not prove reachability.
- completion state: Stream lifecycle state such as completed or cancelled. It does not make receipts authoritative.
- error state: Stream lifecycle state or event describing failure. It does not decide recovery policy.

Required event kinds:
request, response, chunk, metric, receipt, error, complete.

Required stream lifecycle:
created, open, half_closed, completed, failed, cancelled.

Stream envelopes use origin_node_id and target_node_id as references to NET
node identities. NET.SPINE.3 does not require a live registry; it only makes
the reference vocabulary explicit.

## Node Identity

A node is a communication participant, not an authority. A node can advertise
capabilities, but capability does not grant permission. A node id is a stable
NET-facing identifier, not necessarily a hostname. A local machine projection is
a safe view, not raw host inventory. NET may represent node health later, but
NET does not approve operations. system/ must not perform LAN discovery
directly. engine/ must not store live endpoint truth as authority. Real
hostnames, IPs, usernames, MAC addresses and hardware serials must not appear in
canonical fixtures.

- node: Communication participant represented by NET. It does not own authority.
- node id: Stable NET-facing identifier for a node. It is not necessarily a hostname.
- node label: Human-readable label for display or fixture clarity. It is not identity proof.
- node kind: One of local, localhost_service, lan, remote or external. It does not imply trust.
- node scope: One of process, machine, lan, remote or external. It does not imply discovery.
- node status: One of unknown, declared, observed, healthy, degraded, unavailable or retired. It does not approve operations.
- node projection: Safe NET-facing view of node identity material. It is not raw machine inventory.
- local machine projection: Safe local machine view with fake or scrubbed metadata. It must not expose hostname, username, IP, MAC or hardware serial data.
- endpoint reference: Identifier reference to future endpoint material. It is not live endpoint truth.
- capability reference: Identifier reference to advertised capability material. It does not grant permission.
- identity source: Description of where the node identity was declared or observed. It is not proof of trust.
- safe metadata: Small generic metadata safe for canonical fixtures. It must not include private machine metadata.
- private machine metadata: Hostnames, IPs, usernames, MAC addresses, hardware serials, real filesystem paths or secrets. It must not appear in canonical NET fixtures.

Required node kinds:
local, localhost_service, lan, remote, external.

Required node scopes:
process, machine, lan, remote, external.

Required node states:
unknown, declared, observed, healthy, degraded, unavailable, retired.

## Capability Advertisement

A capability says what a node can do. A capability does not approve an
operation, grant case authority or resolve policy. Routing may later use
capabilities as input, but routing is not implemented here. YAI remains
responsible for authority and policy decisions. NET may move invocation material
to a capable node later; this wave only creates the advertised vocabulary.

- capability: Describes a node's advertised work surface. It does not approve an operation.
- capability id: Stable NET-facing identifier for a capability advertisement. It is not authority.
- capability kind: One of neural_llm_decode, neural_embedding_encode, metrics_stream, receipt_emit or generic_external.
- capability family: Broad grouping such as neural, metrics, receipt or generic. It does not imply routing.
- capability scope: Where the advertised capability applies: process, node, service or external.
- capability state: One of unknown, declared, available, degraded, unavailable or retired. It is not policy approval.
- advertised capability: Capability material associated with a node id. It does not grant permission.
- capability constraints: Safe limits or requirements advertised with the capability. They do not replace YAI policy.
- capability metadata: Safe descriptive metadata for compatibility and notes. It must not claim production implementation.
- node capability reference: Node fixture reference to a capability id. It is not live registry resolution.
- registry seed: Static protocol fixture material for future registry work. It is not a runtime registry.
- compatibility tag: Descriptive tag for compatibility posture. It does not vendor or certify an external implementation.

Required capability kinds:
neural_llm_decode, neural_embedding_encode, metrics_stream, receipt_emit, generic_external.

Required capability families:
neural, metrics, receipt, generic.

Required capability scopes:
process, node, service, external.

Required capability states:
unknown, declared, available, degraded, unavailable, retired.

## Endpoint Descriptor

An endpoint is a communication address surface, not an authority. An endpoint
descriptor may be declared before it is observed. NET may later route through
endpoints, but this wave does not route. NET may later probe endpoints, but
this wave does not probe. Endpoint availability does not approve any YAI
operation. Endpoint metadata must not expose secrets or real private machine
identity in canonical fixtures.

- endpoint: Communication address surface associated with a node. It does not approve operations.
- endpoint id: Stable NET-facing identifier for an endpoint descriptor. It is not proof of reachability.
- endpoint kind: One of local_process, localhost_http, local_ipc, lan_http, remote_http or future_transport.
- endpoint scope: One of process, machine, lan, remote or external. It does not imply discovery.
- endpoint state: One of unknown, declared, available, degraded, unavailable or retired. It is not policy approval.
- endpoint security: One of unknown, local_only, trusted_local, declared_remote or external_untrusted. It is not trust authority.
- address reference: Abstract address surface such as local-process://demo. It must not expose real host, port, path or private machine data.
- local endpoint: Process-local endpoint declaration. It is not service lifecycle ownership.
- localhost endpoint: Machine-local loopback posture declaration. It does not implement HTTP in this wave.
- LAN endpoint: LAN posture declaration. It is not discovered in this wave.
- remote endpoint: Remote posture declaration. It is not trusted by placement alone.
- external endpoint: External node endpoint declaration. It does not become part of YAI authority.
- declared endpoint: Endpoint material recorded as a static seed before observation or probing.
- endpoint registry seed: Static fixture material for future registry work. It is not a live runtime registry.

Required endpoint kinds:
local_process, localhost_http, local_ipc, lan_http, remote_http, future_transport.

Required endpoint scopes:
process, machine, lan, remote, external.

Required endpoint states:
unknown, declared, available, degraded, unavailable, retired.

Required endpoint security values:
unknown, local_only, trusted_local, declared_remote, external_untrusted.

## Health, Readiness and Liveness

A health report is an observation or declaration, not authority. Liveness means
a subject appears alive. Readiness means a subject appears ready for a future
operation. Capability readiness is about declared capability availability, not
permission. Endpoint reachability is about communication surface status, not
operation approval. NET may transport or represent health material. YAI remains
responsible for authority and policy. This wave does not perform live probes.
Fixture health reports are canonical examples, not real observations.

- health report: Versioned health material about a subject. It is not an authorization decision.
- health state: One of unknown, alive, ready, degraded, unavailable or failed.
- liveness: Observation or declaration that a subject appears alive. It is not readiness.
- readiness: Observation or declaration that a subject appears ready for future work. It is not permission.
- capability readiness: Availability posture for a declared capability. It does not approve use.
- endpoint reachability: Status of a communication surface. It does not approve an operation.
- health subject: Node, endpoint, capability, transport or external material referenced by a health report.
- observed health: Health material reported by an observing node. It is not proof of trust.
- declared health: Health material declared as seed or fixture material before live observation.
- degraded state: Subject may have limited availability. Recovery policy is outside this contract.
- unavailable state: Subject is represented as unavailable. This does not decide fallback policy.

Required health states:
unknown, alive, ready, degraded, unavailable, failed.

Required health check kinds:
health, liveness, readiness, capability_readiness, endpoint_reachability.

Required health subject kinds:
node, endpoint, capability, transport, external.

## Local Service Lifecycle

A lifecycle request describes intent; it does not execute anything by itself. A
lifecycle report describes declared or observed state; it is not authority. NET
may represent lifecycle material for local services later. YAI remains
responsible for deciding whether a lifecycle action is allowed. This wave does
not start, stop, restart, supervise, probe or route services. CLORI may later
be controlled through lifecycle-compatible surfaces, but no CLORI behavior is
implemented here.

- lifecycle request: Versioned intent material for a lifecycle action. It does not execute work.
- lifecycle report: Versioned state material for a lifecycle subject. It is not authority.
- lifecycle action: One of declare, prepare, start, stop, restart, status or retire.
- lifecycle state: One of unknown, declared, preparing, starting, running, stopping, stopped, failed or retired.
- lifecycle subject: Node, endpoint, capability, service or external material referenced by lifecycle material.
- declared service: Service represented as known seed material. It is not running proof.
- prepared service: Service represented as prepared for a future operation. It is not permission to start.
- running service: Service represented as running by declared or observed state. It is not authority.
- stopped service: Service represented as stopped. It does not decide recovery policy.
- failed service: Service represented as failed. It does not decide fallback policy.
- retired service: Service represented as no longer active. It is not deletion.
- service lifecycle boundary: NET may represent local lifecycle state later, but YAI decides whether action is allowed.

Required lifecycle actions:
declare, prepare, start, stop, restart, status, retire.

Required lifecycle states:
unknown, declared, preparing, starting, running, stopping, stopped, failed, retired.

Required lifecycle subject kinds:
node, endpoint, capability, service, external.

## Transport Adapter

A transport adapter describes how NET may later move stream envelopes. A
transport adapter does not approve an invocation, perform policy or make
receipts authoritative. Localhost transport is a declared local-machine
communication surface. This wave does not open sockets, make HTTP calls, probe
endpoints or move data. Endpoint descriptors and transport adapter descriptors
are linked but distinct.

- transport adapter: Descriptor for a future stream movement surface. It is not live IO.
- localhost transport: Local-machine transport posture for a localhost endpoint. It does not perform HTTP here.
- local IPC transport: Local IPC posture for future platform-native channels. It does not open sockets here.
- LAN HTTP transport: LAN transport posture. It is not LAN discovery and not live HTTP.
- remote HTTP transport: Remote transport posture. It is not trusted by placement alone.
- future transport: Reserved adapter posture for later transport forms.
- transport scope: One of process, machine, lan, remote or external. It does not imply reachability.
- transport state: One of unknown, declared, available, degraded, unavailable or retired.
- transport security: One of unknown, local_only, trusted_local, declared_remote or external_untrusted.
- stream profile: Declared stream envelope profile a future adapter may carry.
- adapter descriptor: Versioned transport adapter declaration linked to an endpoint id.
- declared transport: Static transport material represented before live implementation exists.
- transport boundary: NET may later move streams through adapters; YAI still controls authority.

Required transport kinds:
localhost_http, local_ipc, lan_http, remote_http, future.

Required transport scopes:
process, machine, lan, remote, external.

Required transport states:
unknown, declared, available, degraded, unavailable, retired.

Required transport security values:
unknown, local_only, trusted_local, declared_remote, external_untrusted.

## Local IPC Channel

A local IPC channel descriptor describes a possible local communication
channel. It does not create a channel, open a socket, expose a filesystem path,
approve an invocation, perform policy or move stream envelopes in this wave.
IPC channel descriptors may reference transport adapter descriptors and
endpoint descriptors. Canonical fixtures must use abstract channel references,
not real filesystem paths.

- local IPC channel: Declared local communication channel surface. It is not live IPC.
- IPC channel descriptor: Versioned local IPC channel declaration linked to transport and endpoint ids.
- abstract local channel: Abstract local channel posture that avoids platform and path details.
- Unix socket future channel: Reserved future Unix socket posture. It does not expose a socket path.
- named pipe future channel: Reserved future named pipe posture. It does not create a pipe.
- platform IPC future channel: Reserved future platform IPC posture. It does not call platform APIs.
- memory channel future: Reserved future memory channel posture. It does not allocate shared memory.
- channel reference: Abstract channel identifier such as ipc-abstract://demo. It is not a filesystem path.
- channel scope: One of process, machine, session or future.
- channel state: One of unknown, declared, available, degraded, unavailable or retired.
- channel security: One of unknown, local_only, same_user_future, declared_local or untrusted.
- transport adapter relation: IPC descriptors reference a transport adapter id. The relation is declarative.
- endpoint relation: IPC descriptors reference an endpoint id. The relation is declarative.

Required IPC channel kinds:
abstract_local, unix_socket_future, named_pipe_future, platform_ipc_future, memory_channel_future.

Required IPC channel scopes:
process, machine, session, future.

Required IPC channel states:
unknown, declared, available, degraded, unavailable, retired.

Required IPC channel security values:
unknown, local_only, same_user_future, declared_local, untrusted.
