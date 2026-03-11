#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <string.h>
#include <time.h>

#include <yai/orchestration/peer_registry.h>

#define YAI_OWNER_PEER_REGISTRY_MAX 512

typedef struct yai_owner_peer_registry_slot {
  int used;
  yai_owner_peer_registry_entry_t e;
} yai_owner_peer_registry_slot_t;

static yai_owner_peer_registry_slot_t g_slots[YAI_OWNER_PEER_REGISTRY_MAX];

static int set_err(char *err, size_t err_cap, const char *msg)
{
  if (!err || err_cap == 0)
  {
    return -1;
  }
  if (!msg)
  {
    err[0] = '\0';
    return 0;
  }
  if (snprintf(err, err_cap, "%s", msg) >= (int)err_cap)
  {
    return -1;
  }
  return 0;
}

static int64_t now_epoch(void)
{
  return (int64_t)time(NULL);
}

static int same_key(const yai_owner_peer_registry_entry_t *a,
                    const yai_owner_peer_registry_entry_t *b)
{
  return a && b &&
         strcmp(a->workspace_id, b->workspace_id) == 0 &&
         strcmp(a->source_node_id, b->source_node_id) == 0 &&
         strcmp(a->source_binding_id, b->source_binding_id) == 0;
}

static int find_slot_for(const yai_owner_peer_registry_entry_t *entry)
{
  size_t i = 0;
  int first_free = -1;
  for (i = 0; i < YAI_OWNER_PEER_REGISTRY_MAX; ++i)
  {
    if (!g_slots[i].used)
    {
      if (first_free < 0)
      {
        first_free = (int)i;
      }
      continue;
    }
    if (same_key(&g_slots[i].e, entry))
    {
      return (int)i;
    }
  }
  return first_free;
}

static int find_slot_by_key(const char *workspace_id,
                            const char *source_node_id,
                            const char *source_binding_id)
{
  size_t i = 0;
  if (!workspace_id || !workspace_id[0] ||
      !source_node_id || !source_node_id[0] ||
      !source_binding_id || !source_binding_id[0])
  {
    return -1;
  }
  for (i = 0; i < YAI_OWNER_PEER_REGISTRY_MAX; ++i)
  {
    if (!g_slots[i].used) continue;
    if (strcmp(g_slots[i].e.workspace_id, workspace_id) != 0) continue;
    if (strcmp(g_slots[i].e.source_node_id, source_node_id) != 0) continue;
    if (strcmp(g_slots[i].e.source_binding_id, source_binding_id) != 0) continue;
    return (int)i;
  }
  return -1;
}

static const char *freshness_state(const yai_owner_peer_registry_entry_t *e,
                                   int64_t now)
{
  int64_t delta = 0;
  if (!e || e->last_seen_epoch <= 0)
  {
    return "unknown";
  }
  delta = now - e->last_seen_epoch;
  if (delta <= 30)
  {
    return "fresh";
  }
  if (delta <= 120)
  {
    return "stale";
  }
  return "disconnected";
}

static const char *effective_state(const yai_owner_peer_registry_entry_t *e,
                                   int64_t now)
{
  const char *fresh = freshness_state(e, now);
  if (!e)
  {
    return "unknown";
  }
  if (strcmp(fresh, "disconnected") == 0)
  {
    return "disconnected";
  }
  if (e->backlog_failed > 0)
  {
    return "degraded";
  }
  if (e->backlog_retry_due > 0)
  {
    return "draining";
  }
  if (e->peer_state[0])
  {
    return e->peer_state;
  }
  return "active";
}

static int build_workspace_peer_rows_json(const char *workspace_id,
                                          char *out_json,
                                          size_t out_cap,
                                          int *peer_count_out,
                                          int *ready_count_out,
                                          int *degraded_count_out,
                                          int *disconnected_count_out,
                                          int *stale_count_out,
                                          int64_t *backlog_queued_out,
                                          int64_t *backlog_retry_due_out,
                                          int64_t *backlog_failed_out,
                                          int64_t *replay_detected_out,
                                          int64_t *overlap_detected_out,
                                          int64_t *conflict_detected_out,
                                          int64_t *ordering_late_out,
                                          int64_t *review_required_out,
                                          int *coverage_scope_count_out,
                                          int *coverage_overlap_count_out,
                                          int *coverage_gap_count_out,
                                          int *coverage_distinct_count_out,
                                          char *err,
                                          size_t err_cap)
{
  size_t i = 0;
  int64_t now = now_epoch();
  char peers_json[8192];
  size_t used = 0;
  int first = 1;
  char seen_coverage[64][256];
  int seen_coverage_count = 0;
  int coverage_overlap = 0;
  int coverage_gap = 0;
  int coverage_distinct = 0;
  int peer_count = 0;
  int ready_count = 0;
  int degraded_count = 0;
  int disconnected_count = 0;
  int stale_count = 0;
  int64_t backlog_queued = 0;
  int64_t backlog_retry_due = 0;
  int64_t backlog_failed = 0;
  int64_t replay_detected = 0;
  int64_t overlap_detected = 0;
  int64_t conflict_detected = 0;
  int64_t ordering_late = 0;
  int64_t review_required = 0;

  peers_json[0] = '[';
  peers_json[1] = '\0';
  used = 1;

  for (i = 0; i < YAI_OWNER_PEER_REGISTRY_MAX; ++i)
  {
    const yai_owner_peer_registry_entry_t *e = NULL;
    const char *fresh = NULL;
    const char *eff = NULL;
    int n = 0;
    int seen = 0;
    int j = 0;
    if (!g_slots[i].used) continue;
    if (strcmp(g_slots[i].e.workspace_id, workspace_id) != 0) continue;
    e = &g_slots[i].e;
    fresh = freshness_state(e, now);
    eff = effective_state(e, now);
    peer_count += 1;
    backlog_queued += e->backlog_queued;
    backlog_retry_due += e->backlog_retry_due;
    backlog_failed += e->backlog_failed;
    replay_detected += e->replay_detected_count;
    overlap_detected += e->overlap_detected_count;
    conflict_detected += e->conflict_detected_count;
    ordering_late += e->ordering_late_count;
    review_required += e->review_required_count;
    if (strcmp(eff, "disconnected") == 0) disconnected_count += 1;
    else if (strcmp(eff, "degraded") == 0 || strcmp(eff, "draining") == 0) degraded_count += 1;
    else ready_count += 1;
    if (strcmp(fresh, "stale") == 0) stale_count += 1;

    if (e->coverage_ref[0] && seen_coverage_count < (int)(sizeof(seen_coverage) / sizeof(seen_coverage[0])))
    {
      for (j = 0; j < seen_coverage_count; ++j)
      {
        if (strcmp(seen_coverage[j], e->coverage_ref) == 0)
        {
          seen = 1;
          break;
        }
      }
      if (!seen)
      {
        snprintf(seen_coverage[seen_coverage_count], sizeof(seen_coverage[0]), "%s", e->coverage_ref);
        seen_coverage_count += 1;
      }
    }
    if (strcmp(e->overlap_state, "overlap_confirmed") == 0 || strcmp(e->overlap_state, "overlap_possible") == 0)
    {
      coverage_overlap += 1;
    }
    else if (strcmp(e->overlap_state, "gap") == 0 || strcmp(e->overlap_state, "uncovered") == 0)
    {
      coverage_gap += 1;
    }
    else
    {
      coverage_distinct += 1;
    }

    n = snprintf(peers_json + used,
                 sizeof(peers_json) - used,
                 "%s{\"source_node_id\":\"%s\",\"source_binding_id\":\"%s\",\"daemon_instance_id\":\"%s\",\"peer_role\":\"%s\",\"peer_scope\":\"%s\",\"effective_state\":\"%s\",\"freshness\":\"%s\",\"backlog\":{\"queued\":%lld,\"retry_due\":%lld,\"failed\":%lld},\"integrity\":{\"replay_detected\":%lld,\"overlap_detected\":%lld,\"conflict_detected\":%lld,\"ordering_late\":%lld,\"review_required\":%lld,\"last_classification\":\"%s\",\"last_handling_action\":\"%s\"},\"coverage_ref\":\"%s\",\"overlap_state\":\"%s\",\"last_seen_epoch\":%lld,\"last_activity_epoch\":%lld}",
                 first ? "" : ",",
                 e->source_node_id,
                 e->source_binding_id,
                 e->daemon_instance_id,
                 e->peer_role,
                 e->peer_scope,
                 eff,
                 fresh,
                 (long long)e->backlog_queued,
                 (long long)e->backlog_retry_due,
                 (long long)e->backlog_failed,
                 (long long)e->replay_detected_count,
                 (long long)e->overlap_detected_count,
                 (long long)e->conflict_detected_count,
                 (long long)e->ordering_late_count,
                 (long long)e->review_required_count,
                 e->last_integrity_classification[0] ? e->last_integrity_classification : "unknown",
                 e->last_handling_action[0] ? e->last_handling_action : "unknown",
                 e->coverage_ref,
                 e->overlap_state,
                 (long long)e->last_seen_epoch,
                 (long long)e->last_activity_epoch);
    if (n <= 0 || (size_t)n >= (sizeof(peers_json) - used))
    {
      (void)set_err(err, err_cap, "peer_registry_summary_encode_failed");
      return -1;
    }
    used += (size_t)n;
    first = 0;
  }

  if (used + 2 >= sizeof(peers_json))
  {
    (void)set_err(err, err_cap, "peer_registry_summary_overflow");
    return -1;
  }
  peers_json[used++] = ']';
  peers_json[used] = '\0';

  if (out_json && out_cap > 0)
  {
    if (snprintf(out_json, out_cap, "%s", peers_json) <= 0)
    {
      (void)set_err(err, err_cap, "peer_registry_rows_encode_failed");
      return -1;
    }
  }
  if (peer_count_out) *peer_count_out = peer_count;
  if (ready_count_out) *ready_count_out = ready_count;
  if (degraded_count_out) *degraded_count_out = degraded_count;
  if (disconnected_count_out) *disconnected_count_out = disconnected_count;
  if (stale_count_out) *stale_count_out = stale_count;
  if (backlog_queued_out) *backlog_queued_out = backlog_queued;
  if (backlog_retry_due_out) *backlog_retry_due_out = backlog_retry_due;
  if (backlog_failed_out) *backlog_failed_out = backlog_failed;
  if (replay_detected_out) *replay_detected_out = replay_detected;
  if (overlap_detected_out) *overlap_detected_out = overlap_detected;
  if (conflict_detected_out) *conflict_detected_out = conflict_detected;
  if (ordering_late_out) *ordering_late_out = ordering_late;
  if (review_required_out) *review_required_out = review_required;
  if (coverage_scope_count_out) *coverage_scope_count_out = seen_coverage_count;
  if (coverage_overlap_count_out) *coverage_overlap_count_out = coverage_overlap;
  if (coverage_gap_count_out) *coverage_gap_count_out = coverage_gap;
  if (coverage_distinct_count_out) *coverage_distinct_count_out = coverage_distinct;
  return 0;
}

int yai_owner_peer_registry_upsert(const yai_owner_peer_registry_entry_t *entry,
                                   char *err,
                                   size_t err_cap)
{
  int idx = -1;
  if (err && err_cap > 0)
  {
    err[0] = '\0';
  }
  if (!entry || !entry->workspace_id[0] || !entry->source_node_id[0])
  {
    (void)set_err(err, err_cap, "peer_registry_bad_args");
    return -1;
  }

  idx = find_slot_for(entry);
  if (idx < 0)
  {
    (void)set_err(err, err_cap, "peer_registry_capacity_exceeded");
    return -1;
  }

  if (!g_slots[idx].used)
  {
    memset(&g_slots[idx], 0, sizeof(g_slots[idx]));
    g_slots[idx].used = 1;
    g_slots[idx].e = *entry;
  }
  else
  {
    yai_owner_peer_registry_entry_t *dst = &g_slots[idx].e;
    if (entry->daemon_instance_id[0]) snprintf(dst->daemon_instance_id, sizeof(dst->daemon_instance_id), "%s", entry->daemon_instance_id);
    if (entry->peer_role[0]) snprintf(dst->peer_role, sizeof(dst->peer_role), "%s", entry->peer_role);
    if (entry->peer_scope[0]) snprintf(dst->peer_scope, sizeof(dst->peer_scope), "%s", entry->peer_scope);
    if (entry->peer_state[0]) snprintf(dst->peer_state, sizeof(dst->peer_state), "%s", entry->peer_state);
    if (entry->coverage_ref[0]) snprintf(dst->coverage_ref, sizeof(dst->coverage_ref), "%s", entry->coverage_ref);
    if (entry->overlap_state[0]) snprintf(dst->overlap_state, sizeof(dst->overlap_state), "%s", entry->overlap_state);
    if (entry->last_integrity_classification[0]) snprintf(dst->last_integrity_classification, sizeof(dst->last_integrity_classification), "%s", entry->last_integrity_classification);
    if (entry->last_handling_action[0]) snprintf(dst->last_handling_action, sizeof(dst->last_handling_action), "%s", entry->last_handling_action);
    dst->backlog_queued = entry->backlog_queued;
    dst->backlog_retry_due = entry->backlog_retry_due;
    dst->backlog_failed = entry->backlog_failed;
    if (entry->replay_detected_count > 0) dst->replay_detected_count = entry->replay_detected_count;
    if (entry->overlap_detected_count > 0) dst->overlap_detected_count = entry->overlap_detected_count;
    if (entry->conflict_detected_count > 0) dst->conflict_detected_count = entry->conflict_detected_count;
    if (entry->ordering_late_count > 0) dst->ordering_late_count = entry->ordering_late_count;
    if (entry->review_required_count > 0) dst->review_required_count = entry->review_required_count;
    if (entry->last_seen_epoch > 0) dst->last_seen_epoch = entry->last_seen_epoch;
    if (entry->last_activity_epoch > 0) dst->last_activity_epoch = entry->last_activity_epoch;
    dst->updated_at_epoch = entry->updated_at_epoch > 0 ? entry->updated_at_epoch : now_epoch();
  }

  if (g_slots[idx].e.last_seen_epoch <= 0)
  {
    g_slots[idx].e.last_seen_epoch = now_epoch();
  }
  if (g_slots[idx].e.updated_at_epoch <= 0)
  {
    g_slots[idx].e.updated_at_epoch = now_epoch();
  }
  return 0;
}

int yai_owner_peer_registry_note_emit(const char *workspace_id,
                                      const char *source_node_id,
                                      const char *source_binding_id,
                                      int success,
                                      char *err,
                                      size_t err_cap)
{
  size_t i = 0;
  int64_t now = now_epoch();
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !source_node_id || !source_node_id[0] || !source_binding_id || !source_binding_id[0])
  {
    (void)set_err(err, err_cap, "peer_registry_emit_bad_args");
    return -1;
  }

  for (i = 0; i < YAI_OWNER_PEER_REGISTRY_MAX; ++i)
  {
    if (!g_slots[i].used) continue;
    if (strcmp(g_slots[i].e.workspace_id, workspace_id) != 0) continue;
    if (strcmp(g_slots[i].e.source_node_id, source_node_id) != 0) continue;
    if (strcmp(g_slots[i].e.source_binding_id, source_binding_id) != 0) continue;
    g_slots[i].e.last_seen_epoch = now;
    g_slots[i].e.last_activity_epoch = now;
    g_slots[i].e.updated_at_epoch = now;
    if (success)
    {
      if (g_slots[i].e.backlog_retry_due > 0) g_slots[i].e.backlog_retry_due -= 1;
      if (g_slots[i].e.backlog_queued > 0) g_slots[i].e.backlog_queued -= 1;
      snprintf(g_slots[i].e.peer_state, sizeof(g_slots[i].e.peer_state), "%s", "active");
    }
    else
    {
      g_slots[i].e.backlog_retry_due += 1;
      snprintf(g_slots[i].e.peer_state, sizeof(g_slots[i].e.peer_state), "%s", "degraded");
    }
    return 0;
  }
  (void)set_err(err, err_cap, "peer_registry_entry_not_found");
  return -1;
}

int yai_owner_peer_registry_note_integrity(const char *workspace_id,
                                           const char *source_node_id,
                                           const char *source_binding_id,
                                           const char *classification,
                                           const char *handling_action,
                                           int ordering_late,
                                           int replay_detected,
                                           int overlap_detected,
                                           int conflict_detected,
                                           int review_required,
                                           char *err,
                                           size_t err_cap)
{
  int idx = -1;
  if (err && err_cap > 0) err[0] = '\0';
  idx = find_slot_by_key(workspace_id, source_node_id, source_binding_id);
  if (idx < 0)
  {
    (void)set_err(err, err_cap, "peer_registry_entry_not_found");
    return -1;
  }
  if (classification && classification[0])
  {
    snprintf(g_slots[idx].e.last_integrity_classification,
             sizeof(g_slots[idx].e.last_integrity_classification),
             "%s",
             classification);
  }
  if (handling_action && handling_action[0])
  {
    snprintf(g_slots[idx].e.last_handling_action,
             sizeof(g_slots[idx].e.last_handling_action),
             "%s",
             handling_action);
  }
  if (ordering_late) g_slots[idx].e.ordering_late_count += 1;
  if (replay_detected) g_slots[idx].e.replay_detected_count += 1;
  if (overlap_detected) g_slots[idx].e.overlap_detected_count += 1;
  if (conflict_detected) g_slots[idx].e.conflict_detected_count += 1;
  if (review_required) g_slots[idx].e.review_required_count += 1;
  g_slots[idx].e.updated_at_epoch = now_epoch();
  return 0;
}

int yai_owner_peer_registry_workspace_summary_json(const char *workspace_id,
                                                   char *out_json,
                                                   size_t out_cap,
                                                   char *err,
                                                   size_t err_cap)
{
  int peer_count = 0;
  int ready_count = 0;
  int degraded_count = 0;
  int disconnected_count = 0;
  int stale_count = 0;
  int64_t backlog_queued = 0;
  int64_t backlog_retry_due = 0;
  int64_t backlog_failed = 0;
  int64_t replay_detected = 0;
  int64_t overlap_detected = 0;
  int64_t conflict_detected = 0;
  int64_t ordering_late = 0;
  int64_t review_required = 0;
  int coverage_scope_count = 0;
  int coverage_overlap_count = 0;
  int coverage_gap_count = 0;
  int coverage_distinct_count = 0;
  char peers_json[8192];
  const char *sched = "nominal";

  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0)
  {
    (void)set_err(err, err_cap, "peer_registry_summary_bad_args");
    return -1;
  }
  if (build_workspace_peer_rows_json(workspace_id,
                                     peers_json,
                                     sizeof(peers_json),
                                     &peer_count,
                                     &ready_count,
                                     &degraded_count,
                                     &disconnected_count,
                                     &stale_count,
                                     &backlog_queued,
                                     &backlog_retry_due,
                                     &backlog_failed,
                                     &replay_detected,
                                     &overlap_detected,
                                     &conflict_detected,
                                     &ordering_late,
                                     &review_required,
                                     &coverage_scope_count,
                                     &coverage_overlap_count,
                                     &coverage_gap_count,
                                     &coverage_distinct_count,
                                     err,
                                     err_cap) != 0)
  {
    return -1;
  }

  if (disconnected_count > 0 || backlog_failed > 0)
  {
    sched = "attention_required";
  }
  else if (backlog_retry_due > 0 || backlog_queued > 20 || stale_count > 0)
  {
    sched = "backlog_pressure";
  }

  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"peer_count\":%d,\"states\":{\"ready\":%d,\"degraded\":%d,\"disconnected\":%d,\"stale\":%d},\"backlog\":{\"queued\":%lld,\"retry_due\":%lld,\"failed\":%lld},\"integrity\":{\"replay_detected\":%lld,\"overlap_detected\":%lld,\"conflict_detected\":%lld,\"ordering_late\":%lld,\"review_required\":%lld},\"coverage\":{\"scope_count\":%d,\"distinct\":%d,\"overlap\":%d,\"gap\":%d},\"scheduling_state\":\"%s\",\"peers\":%s}",
               workspace_id,
               peer_count,
               ready_count,
               degraded_count,
               disconnected_count,
               stale_count,
               (long long)backlog_queued,
               (long long)backlog_retry_due,
               (long long)backlog_failed,
               (long long)replay_detected,
               (long long)overlap_detected,
               (long long)conflict_detected,
               (long long)ordering_late,
               (long long)review_required,
               coverage_scope_count,
               coverage_distinct_count,
               coverage_overlap_count,
               coverage_gap_count,
               sched,
               peers_json) <= 0)
  {
    (void)set_err(err, err_cap, "peer_registry_summary_encode_failed");
    return -1;
  }
  return 0;
}

int yai_owner_peer_registry_workspace_peer_rows_json(const char *workspace_id,
                                                     char *out_json,
                                                     size_t out_cap,
                                                     char *err,
                                                     size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0)
  {
    (void)set_err(err, err_cap, "peer_registry_rows_bad_args");
    return -1;
  }
  return build_workspace_peer_rows_json(workspace_id,
                                        out_json,
                                        out_cap,
                                        NULL, NULL, NULL, NULL, NULL,
                                        NULL, NULL, NULL,
                                        NULL, NULL, NULL, NULL, NULL,
                                        NULL, NULL, NULL, NULL,
                                        err,
                                        err_cap);
}

int yai_owner_peer_registry_workspace_coverage_summary_json(const char *workspace_id,
                                                            char *out_json,
                                                            size_t out_cap,
                                                            char *err,
                                                            size_t err_cap)
{
  int peer_count = 0;
  int coverage_scope_count = 0;
  int coverage_overlap_count = 0;
  int coverage_gap_count = 0;
  int coverage_distinct_count = 0;
  char peers_json[8192];
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !out_json || out_cap == 0)
  {
    (void)set_err(err, err_cap, "peer_registry_coverage_bad_args");
    return -1;
  }
  if (build_workspace_peer_rows_json(workspace_id,
                                     peers_json,
                                     sizeof(peers_json),
                                     &peer_count,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     NULL,
                                     &coverage_scope_count,
                                     &coverage_overlap_count,
                                     &coverage_gap_count,
                                     &coverage_distinct_count,
                                     err,
                                     err_cap) != 0)
  {
    return -1;
  }

  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"peer_count\":%d,\"coverage_scope_count\":%d,\"coverage_distinct_count\":%d,\"overlap_count\":%d,\"gap_count\":%d}",
               workspace_id,
               peer_count,
               coverage_scope_count,
               coverage_distinct_count,
               coverage_overlap_count,
               coverage_gap_count) <= 0)
  {
    (void)set_err(err, err_cap, "peer_registry_coverage_encode_failed");
    return -1;
  }
  return 0;
}
