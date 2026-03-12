-------------------- MODULE yai_protocol_control ---------------------
EXTENDS Naturals

EnvelopeErrors == {
  "ok", "bad_arg", "bad_version", "missing_ws", "ws_mismatch", "missing_type",
  "type_not_allowed", "priv_required", "role_required", "handshake_required"
}

ControlEnvelopeInvariant(version, wsId, reqType, result) ==
  /\ version \in Nat
  /\ wsId \in STRING
  /\ reqType \in STRING
  /\ result \in EnvelopeErrors
  /\ version = 1 => result # "bad_version"

=============================================================================
