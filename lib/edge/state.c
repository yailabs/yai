#include <stdio.h>
#include <string.h>
#include <time.h>

#include <yai/edge/state.h>

static int copy_string(char *dst, size_t dst_cap, const char *src)
{
  if (!dst || dst_cap == 0 || !src)
  {
    return -1;
  }
  if (snprintf(dst, dst_cap, "%s", src) >= (int)dst_cap)
  {
    return -1;
  }
  return 0;
}

static int64_t now_epoch(void)
{
  return (int64_t)time(NULL);
}

int yai_edge_edge_state_init(yai_edge_edge_state_t *state,
                               const yai_edge_config_t *cfg,
                               const yai_edge_paths_t *paths,
                               const char *instance_id)
{
  (void)paths;
  if (!state || !cfg)
  {
    return -1;
  }
  memset(state, 0, sizeof(*state));

  (void)copy_string(state->phase, sizeof(state->phase), YAI_EDGE_EDGE_PHASE_BOOTSTRAP);
  (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "initializing");
  (void)copy_string(state->source_label, sizeof(state->source_label), cfg->source_label);
  (void)copy_string(state->node_identity_state, sizeof(state->node_identity_state), "pending");
  (void)copy_string(state->daemon_instance_id, sizeof(state->daemon_instance_id), instance_id ? instance_id : "");
  (void)copy_string(state->owner_ref, sizeof(state->owner_ref), cfg->owner_ref);
  (void)copy_string(state->owner_association_state, sizeof(state->owner_association_state), "unbound");
  (void)copy_string(state->owner_scope_state, sizeof(state->owner_scope_state), "owner_scoped_subordinate");
  (void)copy_string(state->delegated_policy_state, sizeof(state->delegated_policy_state), "pending");
  (void)copy_string(state->grants_state, sizeof(state->grants_state), "pending");
  (void)copy_string(state->policy_snapshot_version, sizeof(state->policy_snapshot_version), "ws-policy-snapshot-v1");
  (void)copy_string(state->delegated_observation_scope, sizeof(state->delegated_observation_scope), "workspace/default");
  (void)copy_string(state->delegated_mediation_scope, sizeof(state->delegated_mediation_scope), "none");
  (void)copy_string(state->delegated_enforcement_scope, sizeof(state->delegated_enforcement_scope), "none");
  (void)copy_string(state->observation_state, sizeof(state->observation_state), "placeholder_ready");
  (void)copy_string(state->mediation_state, sizeof(state->mediation_state), "placeholder_ready");
  (void)copy_string(state->spool_retry_state, sizeof(state->spool_retry_state), "placeholder_ready");
  (void)copy_string(state->health_state, sizeof(state->health_state), "starting");
  (void)copy_string(state->connectivity_state, sizeof(state->connectivity_state), "unknown");
  (void)copy_string(state->freshness_state, sizeof(state->freshness_state), "unknown");
  (void)copy_string(state->spool_pressure_state, sizeof(state->spool_pressure_state), "low");
  (void)copy_string(state->retry_pressure_state, sizeof(state->retry_pressure_state), "low");
  (void)copy_string(state->policy_staleness_state, sizeof(state->policy_staleness_state), "pending");
  (void)copy_string(state->grant_validity_state, sizeof(state->grant_validity_state), "missing_or_pending");
  (void)copy_string(state->delegated_validity_state, sizeof(state->delegated_validity_state), "missing_or_pending");
  (void)copy_string(state->delegated_refresh_state, sizeof(state->delegated_refresh_state), "not_required");
  (void)copy_string(state->delegated_revoke_state, sizeof(state->delegated_revoke_state), "active");
  (void)copy_string(state->delegated_fallback_mode, sizeof(state->delegated_fallback_mode), "restricted_hold_escalate");
  (void)copy_string(state->delegated_stale_reason, sizeof(state->delegated_stale_reason), "none");
  (void)copy_string(state->degradation_state, sizeof(state->degradation_state), "initializing");

  state->started_at_epoch = now_epoch();
  state->last_tick_epoch = state->started_at_epoch;
  return 0;
}

int yai_edge_edge_state_set_phase(yai_edge_edge_state_t *state, const char *phase)
{
  if (!state || !phase || !phase[0])
  {
    return -1;
  }
  return copy_string(state->phase, sizeof(state->phase), phase);
}

int yai_edge_edge_state_set_runtime_status(yai_edge_edge_state_t *state, const char *status)
{
  if (!state || !status || !status[0])
  {
    return -1;
  }
  return copy_string(state->runtime_status, sizeof(state->runtime_status), status);
}

int yai_edge_edge_state_refresh_from_local(yai_edge_edge_state_t *state,
                                             const yai_edge_local_runtime_t *local,
                                             uint32_t tick_count)
{
  if (!state)
  {
    return -1;
  }

  state->tick_count = tick_count;
  state->last_tick_epoch = now_epoch();

  if (!local)
  {
    return 0;
  }

  if (local->daemon_instance_id[0])
  {
    (void)copy_string(state->daemon_instance_id, sizeof(state->daemon_instance_id), local->daemon_instance_id);
  }
  if (local->source_node_id[0])
  {
    (void)copy_string(state->node_identity_state, sizeof(state->node_identity_state), "bound");
  }
  if (local->owner_registered)
  {
    (void)copy_string(state->owner_association_state, sizeof(state->owner_association_state), "registered");
  }
  if (local->source_enrollment_grant_id[0])
  {
    (void)copy_string(state->grants_state, sizeof(state->grants_state), "present");
  }
  if (local->source_policy_snapshot_id[0])
  {
    (void)copy_string(state->source_policy_snapshot_id,
                      sizeof(state->source_policy_snapshot_id),
                      local->source_policy_snapshot_id);
  }
  if (local->source_capability_envelope_id[0])
  {
    (void)copy_string(state->source_capability_envelope_id,
                      sizeof(state->source_capability_envelope_id),
                      local->source_capability_envelope_id);
  }
  if (local->policy_snapshot_version[0])
  {
    (void)copy_string(state->policy_snapshot_version,
                      sizeof(state->policy_snapshot_version),
                      local->policy_snapshot_version);
  }
  if (local->distribution_target_ref[0])
  {
    (void)copy_string(state->distribution_target_ref,
                      sizeof(state->distribution_target_ref),
                      local->distribution_target_ref);
  }
  if (local->delegated_observation_scope[0])
  {
    (void)copy_string(state->delegated_observation_scope,
                      sizeof(state->delegated_observation_scope),
                      local->delegated_observation_scope);
  }
  if (local->delegated_mediation_scope[0])
  {
    (void)copy_string(state->delegated_mediation_scope,
                      sizeof(state->delegated_mediation_scope),
                      local->delegated_mediation_scope);
  }
  if (local->delegated_enforcement_scope[0])
  {
    (void)copy_string(state->delegated_enforcement_scope,
                      sizeof(state->delegated_enforcement_scope),
                      local->delegated_enforcement_scope);
  }
  if (local->owner_trust_artifact_token[0] &&
      strcmp(local->owner_trust_artifact_token, "pending") != 0)
  {
    (void)copy_string(state->delegated_policy_state,
                      sizeof(state->delegated_policy_state),
                      "owner_grant_token_present");
  }

  state->owner_connected = local->owner_connected;
  state->last_owner_contact_epoch = local->last_owner_contact_epoch;
  state->grant_issued_at_epoch = local->grant_issued_at_epoch;
  state->grant_refresh_after_epoch = local->grant_refresh_after_epoch;
  state->grant_expires_at_epoch = local->grant_expires_at_epoch;
  state->snapshot_issued_at_epoch = local->snapshot_issued_at_epoch;
  state->snapshot_refresh_after_epoch = local->snapshot_refresh_after_epoch;
  state->snapshot_expires_at_epoch = local->snapshot_expires_at_epoch;
  state->capability_issued_at_epoch = local->capability_issued_at_epoch;
  state->capability_refresh_after_epoch = local->capability_refresh_after_epoch;
  state->capability_expires_at_epoch = local->capability_expires_at_epoch;
  state->grant_revoked = local->grant_revoked;
  state->snapshot_revoked = local->snapshot_revoked;
  state->capability_revoked = local->capability_revoked;
  state->spool_queued = local->spool_queued;
  state->spool_retry_due = local->spool_retry_due;
  state->spool_failed = local->spool_failed;
  state->retry_consecutive_failures = local->retry_consecutive_failures;
  state->last_observation_epoch = local->last_observation_epoch;
  state->last_successful_emit_epoch = local->last_successful_emit_epoch;
  (void)copy_string(state->health_state, sizeof(state->health_state), local->health_state);
  (void)copy_string(state->connectivity_state, sizeof(state->connectivity_state), local->connectivity_state);
  (void)copy_string(state->freshness_state, sizeof(state->freshness_state), local->freshness_state);
  (void)copy_string(state->spool_pressure_state, sizeof(state->spool_pressure_state), local->spool_pressure_state);
  (void)copy_string(state->retry_pressure_state, sizeof(state->retry_pressure_state), local->retry_pressure_state);
  (void)copy_string(state->policy_staleness_state, sizeof(state->policy_staleness_state), local->policy_staleness_state);
  (void)copy_string(state->grant_validity_state, sizeof(state->grant_validity_state), local->grant_validity_state);
  (void)copy_string(state->delegated_validity_state, sizeof(state->delegated_validity_state), local->delegated_validity_state);
  (void)copy_string(state->delegated_refresh_state, sizeof(state->delegated_refresh_state), local->delegated_refresh_state);
  (void)copy_string(state->delegated_revoke_state, sizeof(state->delegated_revoke_state), local->delegated_revoke_state);
  (void)copy_string(state->delegated_fallback_mode, sizeof(state->delegated_fallback_mode), local->delegated_fallback_mode);
  (void)copy_string(state->delegated_stale_reason, sizeof(state->delegated_stale_reason), local->delegated_stale_reason);
  (void)copy_string(state->degradation_state, sizeof(state->degradation_state), local->degradation_state);

  if (!local->owner_connected)
  {
    (void)copy_string(state->phase, sizeof(state->phase), YAI_EDGE_EDGE_PHASE_DISCONNECTED);
    (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "degraded");
  }
  else if (strcmp(local->health_state, YAI_EDGE_HEALTH_DEGRADED) == 0)
  {
    (void)copy_string(state->phase, sizeof(state->phase), YAI_EDGE_EDGE_PHASE_DEGRADED);
    (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "degraded");
  }
  else
  {
    (void)copy_string(state->phase, sizeof(state->phase), YAI_EDGE_EDGE_PHASE_OBSERVATION_LOOP);
    (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "running");
  }

  return 0;
}

int yai_edge_edge_state_json(const yai_edge_edge_state_t *state,
                               char *out,
                               size_t out_cap)
{
  if (!state || !out || out_cap == 0)
  {
    return -1;
  }
  if (snprintf(out,
               out_cap,
               "{\n"
               "  \"type\": \"yai.daemon.edge.runtime.state.v1\",\n"
               "  \"phase\": \"%s\",\n"
               "  \"runtime_status\": \"%s\",\n"
               "  \"source_label\": \"%s\",\n"
               "  \"node_identity_state\": \"%s\",\n"
               "  \"daemon_instance_id\": \"%s\",\n"
               "  \"owner_ref\": \"%s\",\n"
               "  \"owner_association_state\": \"%s\",\n"
               "  \"owner_scope_state\": \"%s\",\n"
               "  \"delegated_policy_state\": \"%s\",\n"
               "  \"grants_state\": \"%s\",\n"
               "  \"source_policy_snapshot_id\": \"%s\",\n"
               "  \"source_capability_envelope_id\": \"%s\",\n"
               "  \"policy_snapshot_version\": \"%s\",\n"
               "  \"distribution_target_ref\": \"%s\",\n"
               "  \"delegated_observation_scope\": \"%s\",\n"
               "  \"delegated_mediation_scope\": \"%s\",\n"
               "  \"delegated_enforcement_scope\": \"%s\",\n"
               "  \"observation_state\": \"%s\",\n"
               "  \"mediation_state\": \"%s\",\n"
               "  \"spool_retry_state\": \"%s\",\n"
               "  \"health_state\": \"%s\",\n"
               "  \"connectivity_state\": \"%s\",\n"
               "  \"freshness_state\": \"%s\",\n"
               "  \"spool_pressure_state\": \"%s\",\n"
               "  \"retry_pressure_state\": \"%s\",\n"
               "  \"policy_staleness_state\": \"%s\",\n"
               "  \"grant_validity_state\": \"%s\",\n"
               "  \"delegated_validity_state\": \"%s\",\n"
               "  \"delegated_refresh_state\": \"%s\",\n"
               "  \"delegated_revoke_state\": \"%s\",\n"
               "  \"delegated_fallback_mode\": \"%s\",\n"
               "  \"delegated_stale_reason\": \"%s\",\n"
               "  \"grant_issued_at_epoch\": %lld,\n"
               "  \"grant_refresh_after_epoch\": %lld,\n"
               "  \"grant_expires_at_epoch\": %lld,\n"
               "  \"snapshot_issued_at_epoch\": %lld,\n"
               "  \"snapshot_refresh_after_epoch\": %lld,\n"
               "  \"snapshot_expires_at_epoch\": %lld,\n"
               "  \"capability_issued_at_epoch\": %lld,\n"
               "  \"capability_refresh_after_epoch\": %lld,\n"
               "  \"capability_expires_at_epoch\": %lld,\n"
               "  \"grant_revoked\": %s,\n"
               "  \"snapshot_revoked\": %s,\n"
               "  \"capability_revoked\": %s,\n"
               "  \"degradation_state\": \"%s\",\n"
               "  \"owner_connected\": %s,\n"
               "  \"tick_count\": %u,\n"
               "  \"started_at_epoch\": %lld,\n"
               "  \"last_tick_epoch\": %lld,\n"
               "  \"last_owner_contact_epoch\": %lld,\n"
               "  \"spool_queued\": %u,\n"
               "  \"spool_retry_due\": %u,\n"
               "  \"spool_failed\": %u,\n"
               "  \"retry_consecutive_failures\": %u,\n"
               "  \"last_observation_epoch\": %lld,\n"
               "  \"last_successful_emit_epoch\": %lld\n"
               "}\n",
               state->phase,
               state->runtime_status,
               state->source_label,
               state->node_identity_state,
               state->daemon_instance_id,
               state->owner_ref,
               state->owner_association_state,
               state->owner_scope_state,
               state->delegated_policy_state,
               state->grants_state,
               state->source_policy_snapshot_id,
               state->source_capability_envelope_id,
               state->policy_snapshot_version,
               state->distribution_target_ref,
               state->delegated_observation_scope,
               state->delegated_mediation_scope,
               state->delegated_enforcement_scope,
               state->observation_state,
               state->mediation_state,
               state->spool_retry_state,
               state->health_state,
               state->connectivity_state,
               state->freshness_state,
               state->spool_pressure_state,
               state->retry_pressure_state,
               state->policy_staleness_state,
               state->grant_validity_state,
               state->delegated_validity_state,
               state->delegated_refresh_state,
               state->delegated_revoke_state,
               state->delegated_fallback_mode,
               state->delegated_stale_reason,
               (long long)state->grant_issued_at_epoch,
               (long long)state->grant_refresh_after_epoch,
               (long long)state->grant_expires_at_epoch,
               (long long)state->snapshot_issued_at_epoch,
               (long long)state->snapshot_refresh_after_epoch,
               (long long)state->snapshot_expires_at_epoch,
               (long long)state->capability_issued_at_epoch,
               (long long)state->capability_refresh_after_epoch,
               (long long)state->capability_expires_at_epoch,
               state->grant_revoked ? "true" : "false",
               state->snapshot_revoked ? "true" : "false",
               state->capability_revoked ? "true" : "false",
               state->degradation_state,
               state->owner_connected ? "true" : "false",
               state->tick_count,
               (long long)state->started_at_epoch,
               (long long)state->last_tick_epoch,
               (long long)state->last_owner_contact_epoch,
               state->spool_queued,
               state->spool_retry_due,
               state->spool_failed,
               state->retry_consecutive_failures,
               (long long)state->last_observation_epoch,
               (long long)state->last_successful_emit_epoch) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}
