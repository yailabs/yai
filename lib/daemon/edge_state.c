#include <stdio.h>
#include <string.h>
#include <time.h>

#include <yai/daemon/edge_state.h>

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

int yai_daemon_edge_state_init(yai_daemon_edge_state_t *state,
                               const yai_daemon_config_t *cfg,
                               const yai_daemon_paths_t *paths,
                               const char *instance_id)
{
  (void)paths;
  if (!state || !cfg)
  {
    return -1;
  }
  memset(state, 0, sizeof(*state));

  (void)copy_string(state->phase, sizeof(state->phase), YAI_DAEMON_EDGE_PHASE_BOOTSTRAP);
  (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "initializing");
  (void)copy_string(state->source_label, sizeof(state->source_label), cfg->source_label);
  (void)copy_string(state->node_identity_state, sizeof(state->node_identity_state), "pending");
  (void)copy_string(state->daemon_instance_id, sizeof(state->daemon_instance_id), instance_id ? instance_id : "");
  (void)copy_string(state->owner_ref, sizeof(state->owner_ref), cfg->owner_ref);
  (void)copy_string(state->owner_association_state, sizeof(state->owner_association_state), "unbound");
  (void)copy_string(state->owner_scope_state, sizeof(state->owner_scope_state), "owner_scoped_subordinate");
  (void)copy_string(state->delegated_policy_state, sizeof(state->delegated_policy_state), "pending");
  (void)copy_string(state->grants_state, sizeof(state->grants_state), "pending");
  (void)copy_string(state->observation_state, sizeof(state->observation_state), "placeholder_ready");
  (void)copy_string(state->mediation_state, sizeof(state->mediation_state), "placeholder_ready");
  (void)copy_string(state->spool_retry_state, sizeof(state->spool_retry_state), "placeholder_ready");
  (void)copy_string(state->health_state, sizeof(state->health_state), "starting");

  state->started_at_epoch = now_epoch();
  state->last_tick_epoch = state->started_at_epoch;
  return 0;
}

int yai_daemon_edge_state_set_phase(yai_daemon_edge_state_t *state, const char *phase)
{
  if (!state || !phase || !phase[0])
  {
    return -1;
  }
  return copy_string(state->phase, sizeof(state->phase), phase);
}

int yai_daemon_edge_state_set_runtime_status(yai_daemon_edge_state_t *state, const char *status)
{
  if (!state || !status || !status[0])
  {
    return -1;
  }
  return copy_string(state->runtime_status, sizeof(state->runtime_status), status);
}

int yai_daemon_edge_state_refresh_from_local(yai_daemon_edge_state_t *state,
                                             const yai_daemon_local_runtime_t *local,
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
  if (local->owner_trust_artifact_token[0] &&
      strcmp(local->owner_trust_artifact_token, "pending") != 0)
  {
    (void)copy_string(state->delegated_policy_state,
                      sizeof(state->delegated_policy_state),
                      "owner_grant_token_present");
  }

  state->owner_connected = local->owner_connected;
  state->last_owner_contact_epoch = local->last_owner_contact_epoch;
  state->spool_queued = local->spool_queued;
  state->spool_retry_due = local->spool_retry_due;
  state->spool_failed = local->spool_failed;
  (void)copy_string(state->health_state, sizeof(state->health_state), local->health_state);

  if (!local->owner_connected)
  {
    (void)copy_string(state->phase, sizeof(state->phase), YAI_DAEMON_EDGE_PHASE_DISCONNECTED);
    (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "degraded");
  }
  else if (strcmp(local->health_state, YAI_DAEMON_HEALTH_DEGRADED) == 0)
  {
    (void)copy_string(state->phase, sizeof(state->phase), YAI_DAEMON_EDGE_PHASE_DEGRADED);
    (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "degraded");
  }
  else
  {
    (void)copy_string(state->phase, sizeof(state->phase), YAI_DAEMON_EDGE_PHASE_OBSERVATION_LOOP);
    (void)copy_string(state->runtime_status, sizeof(state->runtime_status), "running");
  }

  return 0;
}

int yai_daemon_edge_state_json(const yai_daemon_edge_state_t *state,
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
               "  \"observation_state\": \"%s\",\n"
               "  \"mediation_state\": \"%s\",\n"
               "  \"spool_retry_state\": \"%s\",\n"
               "  \"health_state\": \"%s\",\n"
               "  \"owner_connected\": %s,\n"
               "  \"tick_count\": %u,\n"
               "  \"started_at_epoch\": %lld,\n"
               "  \"last_tick_epoch\": %lld,\n"
               "  \"last_owner_contact_epoch\": %lld,\n"
               "  \"spool_queued\": %u,\n"
               "  \"spool_retry_due\": %u,\n"
               "  \"spool_failed\": %u\n"
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
               state->observation_state,
               state->mediation_state,
               state->spool_retry_state,
               state->health_state,
               state->owner_connected ? "true" : "false",
               state->tick_count,
               (long long)state->started_at_epoch,
               (long long)state->last_tick_epoch,
               (long long)state->last_owner_contact_epoch,
               state->spool_queued,
               state->spool_retry_due,
               state->spool_failed) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}

