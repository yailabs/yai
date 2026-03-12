#include <yai/governance/discovery.h>

#include "../internal.h"

#include <string.h>

int yai_governance_protocol_match_bonus(const yai_governance_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_governance_provider_match_bonus(const yai_governance_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_governance_resource_match_bonus(const yai_governance_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_governance_command_match_bonus(const yai_governance_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_governance_confidence_label(double score, int *ambiguous);

typedef struct yai_family_candidate {
  char family_id[64];
  char domain_id[64];
  double score;
  char rationale[192];
} yai_family_candidate_t;

static void map_family_to_domain_id(const char *family_id, char *out, size_t out_cap) {
  char domain_id[64];
  if (!family_id || !out || out_cap == 0) return;
  domain_id[0] = '\0';
  if (yai_governance_domain_model_family_resolution(family_id,
                                             domain_id,
                                             sizeof(domain_id),
                                             NULL,
                                             0,
                                             NULL,
                                             0,
                                             NULL) == 0 &&
      domain_id[0] != '\0') {
    (void)yai_governance_safe_snprintf(out, out_cap, "%s", domain_id);
    return;
  }
  if (strcmp(family_id, "economic") == 0) (void)yai_governance_safe_snprintf(out, out_cap, "%s", "D5-economic");
  else if (strcmp(family_id, "scientific") == 0) (void)yai_governance_safe_snprintf(out, out_cap, "%s", "D8-scientific");
  else (void)yai_governance_safe_snprintf(out, out_cap, "%s", "D1-digital");
}

static int read_classification_map(char *json, size_t cap) {
  if (!json || cap == 0) return -1;
  if (yai_governance_read_text_file("specs/classification/maps/classification-map.v1.json", json, cap) == 0) return 0;
  if (yai_governance_read_governance_surface_file(NULL, "classification/maps/classification-map.v1.json", json, cap) == 0) return 0;
  return -1;
}

static void apply_classification_map_boost(const yai_governance_classification_ctx_t *ctx,
                                           const char *family_id,
                                           double *score) {
  char map_json[12288];
  char family_needle[96];
  char action_needle[96];
  char resource_needle[96];
  char provider_needle[96];
  if (!ctx || !family_id || !score) return;
  if (read_classification_map(map_json, sizeof(map_json)) != 0) return;
  if (yai_governance_safe_snprintf(family_needle, sizeof(family_needle), "\"family\": \"%s\"", family_id) != 0) return;
  if (!strstr(map_json, family_needle)) return;

  if (ctx->action[0] != '\0') {
    (void)yai_governance_safe_snprintf(action_needle, sizeof(action_needle), "\"%s\"", ctx->action);
    if (strstr(map_json, action_needle)) *score += 0.04;
  }
  if (ctx->resource[0] != '\0') {
    (void)yai_governance_safe_snprintf(resource_needle, sizeof(resource_needle), "\"%s\"", ctx->resource);
    if (strstr(map_json, resource_needle)) *score += 0.03;
  }
  if (ctx->provider[0] != '\0') {
    (void)yai_governance_safe_snprintf(provider_needle, sizeof(provider_needle), "\"%s\"", ctx->provider);
    if (strstr(map_json, provider_needle)) *score += 0.02;
  }
}

static int specialization_exists(const char *family_id, const char *specialization_id) {
  if (!family_id || !specialization_id) return 0;
  return yai_governance_domain_model_specialization_exists(family_id, specialization_id);
}

static void push_specialization_candidate(yai_governance_discovery_result_t *out, const char *spec) {
  if (!out || !spec) return;
  if (out->specialization_candidate_count >= 6) return;
  (void)yai_governance_safe_snprintf(
      out->specialization_candidates[out->specialization_candidate_count],
      sizeof(out->specialization_candidates[out->specialization_candidate_count]),
      "%s",
      spec);
  out->specialization_candidate_count++;
}

static void choose_specialization(const yai_governance_classification_ctx_t *ctx,
                                  const char *family_id,
                                  yai_governance_discovery_result_t *out) {
  char candidates[16][96];
  int candidate_count = 0;
  char default_specialization[96];
  int i;

  if (!ctx || !family_id || !out) return;
  default_specialization[0] = '\0';
  (void)yai_governance_domain_model_family_resolution(family_id,
                                               NULL,
                                               0,
                                               default_specialization,
                                               sizeof(default_specialization),
                                               candidates,
                                               16,
                                               &candidate_count);

  for (i = 0; i < candidate_count && i < 6; ++i) {
    push_specialization_candidate(out, candidates[i]);
  }

  if (strcmp(family_id, "economic") == 0) {
    if (strstr(ctx->command, "settlement")) (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "settlements");
    else if (strstr(ctx->command, "transfer")) (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "transfers");
    else if (strstr(ctx->command, "fraud") || strstr(ctx->resource, "anomaly")) (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "fraud-risk-controls");
    else if (default_specialization[0]) (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", default_specialization);
    else (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "payments");
  } else if (strcmp(family_id, "scientific") == 0) {
    if (ctx->black_box_mode || strstr(ctx->command, "black-box") || strstr(ctx->command, "black_box")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "black-box-evaluation");
    } else if (ctx->has_publication_intent || strstr(ctx->command, "publish") || strstr(ctx->command, "export")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "result-publication-control");
    } else if (ctx->has_repro_context || strstr(ctx->command, "repro") || strstr(ctx->command, "lineage")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "reproducibility-control");
    } else if ((ctx->has_dataset_ref && strcmp(ctx->action, "publish") == 0) ||
               strstr(ctx->command, "dataset.verify") ||
               strstr(ctx->command, "dataset.audit") ||
               strstr(ctx->command, "dataset.integrity")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "dataset-integrity");
    } else if (strstr(ctx->command, "configure") || strstr(ctx->command, "config") || strstr(ctx->command, "setup")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "experiment-configuration");
    } else if (ctx->has_locked_parameters || ctx->has_params_hash || strstr(ctx->command, "parameter") || strstr(ctx->command, "params")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "parameter-governance");
    } else if (strstr(ctx->command, "result")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "result-publication-control");
    } else if (default_specialization[0]) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", default_specialization);
    } else {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "parameter-governance");
    }
  } else {
    if (ctx->has_commentary_intent || strstr(ctx->command, "comment")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "external-commentary");
    } else if (ctx->has_distribution_intent || strstr(ctx->command, "distribution") || strstr(ctx->command, "deliver")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "artifact-distribution");
    } else if (ctx->has_retrieve_intent || strstr(ctx->command, "retrieve")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "remote-retrieval");
    } else if (ctx->has_sink_ref || strstr(ctx->command, "sink") || strstr(ctx->resource, "sink")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "digital-sink-control");
    } else if (ctx->has_publication_intent || strstr(ctx->command, "publish")) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "remote-publication");
    } else if (default_specialization[0]) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", default_specialization);
    } else {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "network-egress");
    }
  }

  if (!specialization_exists(family_id, out->specialization_id)) {
    if (default_specialization[0]) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", default_specialization);
    } else if (candidate_count > 0) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", candidates[0]);
    } else if (strcmp(family_id, "economic") == 0) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "payments");
    } else if (strcmp(family_id, "scientific") == 0) {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "parameter-governance");
    } else {
      (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "network-egress");
    }
  }
}

static void sort_family_candidates(yai_family_candidate_t *cands, int n) {
  int i;
  int j;
  for (i = 0; i < n; ++i) {
    for (j = i + 1; j < n; ++j) {
      if (cands[j].score > cands[i].score) {
        yai_family_candidate_t tmp = cands[i];
        cands[i] = cands[j];
        cands[j] = tmp;
      }
    }
  }
}

static void apply_declared_workspace_hints(const yai_governance_classification_ctx_t *ctx,
                                           yai_family_candidate_t *cands,
                                           int n) {
  int i;
  if (!ctx || !cands || n <= 0) return;
  if (ctx->declared_family[0] == '\0' && ctx->declared_specialization[0] == '\0') return;

  for (i = 0; i < n; ++i) {
    if (ctx->declared_family[0] != '\0' && strcmp(cands[i].family_id, ctx->declared_family) == 0) {
      cands[i].score += 0.20; /* strong hint: prefer workspace-declared family when compatible */
      (void)yai_governance_safe_snprintf(cands[i].rationale, sizeof(cands[i].rationale),
                                  "workspace declared family hint + classification match");
    }
  }
}

int yai_governance_discover_domain(const yai_governance_classification_ctx_t *ctx,
                            yai_governance_discovery_result_t *out) {
  double b = 0.0;
  char families[4][64];
  int family_count = 0;
  yai_family_candidate_t candidates[4];
  int i;

  if (!ctx || !out) return -1;
  memset(out, 0, sizeof(*out));
  memset(candidates, 0, sizeof(candidates));

  if (yai_governance_domain_model_runtime_families(families, 4, &family_count) != 0 || family_count <= 0) {
    (void)yai_governance_safe_snprintf(families[0], sizeof(families[0]), "%s", "digital");
    (void)yai_governance_safe_snprintf(families[1], sizeof(families[1]), "%s", "economic");
    (void)yai_governance_safe_snprintf(families[2], sizeof(families[2]), "%s", "scientific");
    family_count = 3;
  }

  for (i = 0; i < family_count; ++i) {
    (void)yai_governance_safe_snprintf(candidates[i].family_id, sizeof(candidates[i].family_id), "%s", families[i]);
    map_family_to_domain_id(families[i], candidates[i].domain_id, sizeof(candidates[i].domain_id));

    (void)yai_governance_match_signal_score(ctx, families[i], &candidates[i].score, candidates[i].rationale, sizeof(candidates[i].rationale));
    (void)yai_governance_protocol_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    (void)yai_governance_provider_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    (void)yai_governance_resource_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    (void)yai_governance_command_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    apply_classification_map_boost(ctx, families[i], &candidates[i].score);
  }

  sort_family_candidates(candidates, family_count);
  apply_declared_workspace_hints(ctx, candidates, family_count);
  sort_family_candidates(candidates, family_count);
  if (family_count > 4) family_count = 4;
  out->family_candidate_count = family_count;
  for (i = 0; i < family_count; ++i) {
    (void)yai_governance_safe_snprintf(out->family_candidates[i], sizeof(out->family_candidates[i]), "%s", candidates[i].family_id);
    out->family_candidate_scores[i] = candidates[i].score;
  }

  (void)yai_governance_safe_snprintf(out->family_id, sizeof(out->family_id), "%s", candidates[0].family_id);
  (void)yai_governance_safe_snprintf(out->domain_id, sizeof(out->domain_id), "%s", candidates[0].domain_id);
  out->confidence = candidates[0].score;
  (void)yai_governance_safe_snprintf(out->rationale, sizeof(out->rationale), "%s", candidates[0].rationale);
  choose_specialization(ctx, out->family_id, out);

  if (ctx->declared_specialization[0] != '\0' &&
      specialization_exists(out->family_id, ctx->declared_specialization)) {
    (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", ctx->declared_specialization);
    out->confidence += 0.06;
    if (out->confidence > 1.0) out->confidence = 1.0;
    (void)yai_governance_safe_snprintf(out->rationale, sizeof(out->rationale),
                                "workspace declared specialization hint selected");
  }

  (void)yai_governance_confidence_label(out->confidence, &out->ambiguous);
  if (out->confidence < 0.45) {
    (void)yai_governance_safe_snprintf(out->family_id, sizeof(out->family_id), "%s", "digital");
    (void)yai_governance_safe_snprintf(out->domain_id, sizeof(out->domain_id), "%s", "D1-digital");
    (void)yai_governance_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "network-egress");
    out->ambiguous = 1;
    (void)yai_governance_safe_snprintf(out->rationale, sizeof(out->rationale), "%s", "low confidence fallback to digital/network-egress");
  }

  return 0;
}
