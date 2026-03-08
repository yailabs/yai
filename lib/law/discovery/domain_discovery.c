#include <yai/law/discovery.h>

#include "../internal.h"

#include <string.h>

int yai_law_protocol_match_bonus(const yai_law_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_law_provider_match_bonus(const yai_law_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_law_resource_match_bonus(const yai_law_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_law_command_match_bonus(const yai_law_classification_ctx_t *ctx, const char *domain_id, double *bonus);
int yai_law_confidence_label(double score, int *ambiguous);

typedef struct yai_family_candidate {
  char family_id[64];
  char domain_id[64];
  double score;
  char rationale[192];
} yai_family_candidate_t;

static void map_family_to_domain_id(const char *family_id, char *out, size_t out_cap) {
  if (strcmp(family_id, "economic") == 0) (void)yai_law_safe_snprintf(out, out_cap, "%s", "D5-economic");
  else if (strcmp(family_id, "scientific") == 0) (void)yai_law_safe_snprintf(out, out_cap, "%s", "D8-scientific");
  else (void)yai_law_safe_snprintf(out, out_cap, "%s", "D1-digital");
}

static int read_classification_map(char *json, size_t cap) {
  if (!json || cap == 0) return -1;
  return yai_law_read_text_file("embedded/law/classification/classification-map.json", json, cap);
}

static int read_specializations_index(char *json, size_t cap) {
  if (!json || cap == 0) return -1;
  return yai_law_read_text_file("embedded/law/domain-specializations/index/specializations.index.json", json, cap);
}

static void apply_classification_map_boost(const yai_law_classification_ctx_t *ctx,
                                           const char *family_id,
                                           double *score) {
  char map_json[12288];
  char family_needle[96];
  char action_needle[96];
  char resource_needle[96];
  char provider_needle[96];
  if (!ctx || !family_id || !score) return;
  if (read_classification_map(map_json, sizeof(map_json)) != 0) return;
  if (yai_law_safe_snprintf(family_needle, sizeof(family_needle), "\"family\": \"%s\"", family_id) != 0) return;
  if (!strstr(map_json, family_needle)) return;

  if (ctx->action[0] != '\0') {
    (void)yai_law_safe_snprintf(action_needle, sizeof(action_needle), "\"%s\"", ctx->action);
    if (strstr(map_json, action_needle)) *score += 0.04;
  }
  if (ctx->resource[0] != '\0') {
    (void)yai_law_safe_snprintf(resource_needle, sizeof(resource_needle), "\"%s\"", ctx->resource);
    if (strstr(map_json, resource_needle)) *score += 0.03;
  }
  if (ctx->provider[0] != '\0') {
    (void)yai_law_safe_snprintf(provider_needle, sizeof(provider_needle), "\"%s\"", ctx->provider);
    if (strstr(map_json, provider_needle)) *score += 0.02;
  }
}

static int specialization_exists(const char *family_id, const char *specialization_id) {
  char idx_json[24576];
  char spec_needle[160];
  char fam_needle[96];
  const char *spec_pos;
  if (!family_id || !specialization_id) return 0;
  if (read_specializations_index(idx_json, sizeof(idx_json)) != 0) return 0;
  if (yai_law_safe_snprintf(spec_needle, sizeof(spec_needle), "\"specialization_id\": \"%s\"", specialization_id) != 0) return 0;
  if (yai_law_safe_snprintf(fam_needle, sizeof(fam_needle), "\"family\": \"%s\"", family_id) != 0) return 0;
  spec_pos = strstr(idx_json, spec_needle);
  if (!spec_pos) return 0;
  return strstr(spec_pos, fam_needle) != NULL;
}

static void push_specialization_candidate(yai_law_discovery_result_t *out, const char *spec) {
  if (!out || !spec) return;
  if (out->specialization_candidate_count >= 6) return;
  (void)yai_law_safe_snprintf(
      out->specialization_candidates[out->specialization_candidate_count],
      sizeof(out->specialization_candidates[out->specialization_candidate_count]),
      "%s",
      spec);
  out->specialization_candidate_count++;
}

static void choose_specialization(const yai_law_classification_ctx_t *ctx,
                                  const char *family_id,
                                  yai_law_discovery_result_t *out) {
  if (!ctx || !family_id || !out) return;

  if (strcmp(family_id, "economic") == 0) {
    push_specialization_candidate(out, "payments");
    push_specialization_candidate(out, "transfers");
    push_specialization_candidate(out, "settlements");
    push_specialization_candidate(out, "fraud-risk-controls");
    if (strstr(ctx->command, "settlement")) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "settlements");
    else if (strstr(ctx->command, "transfer")) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "transfers");
    else if (strstr(ctx->command, "fraud") || strstr(ctx->resource, "anomaly")) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "fraud-risk-controls");
    else (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "payments");
  } else if (strcmp(family_id, "scientific") == 0) {
    push_specialization_candidate(out, "parameter-governance");
    push_specialization_candidate(out, "black-box-evaluation");
    push_specialization_candidate(out, "result-publication-control");
    push_specialization_candidate(out, "reproducibility-control");
    if (ctx->black_box_mode || strstr(ctx->command, "black-box") || strstr(ctx->command, "black_box")) {
      (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "black-box-evaluation");
    } else if (strstr(ctx->command, "result")) {
      (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "result-publication-control");
    } else if (strstr(ctx->command, "repro")) {
      (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "reproducibility-control");
    } else {
      (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "parameter-governance");
    }
  } else {
    push_specialization_candidate(out, "network-egress");
    push_specialization_candidate(out, "remote-publication");
    push_specialization_candidate(out, "external-commentary");
    push_specialization_candidate(out, "remote-retrieval");
    if (strstr(ctx->command, "comment")) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "external-commentary");
    else if (strstr(ctx->command, "retrieve")) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "remote-retrieval");
    else if (strstr(ctx->command, "publish")) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "remote-publication");
    else (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "network-egress");
  }

  if (!specialization_exists(family_id, out->specialization_id)) {
    /* Guardrail: keep a known valid fallback per family if selected candidate is not in index. */
    if (strcmp(family_id, "economic") == 0) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "payments");
    else if (strcmp(family_id, "scientific") == 0) (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "parameter-governance");
    else (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "network-egress");
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

int yai_law_discover_domain(const yai_law_classification_ctx_t *ctx,
                            yai_law_discovery_result_t *out) {
  double b = 0.0;
  const char *families[] = {"digital", "economic", "scientific"};
  yai_family_candidate_t candidates[3];
  int i;

  if (!ctx || !out) return -1;
  memset(out, 0, sizeof(*out));
  memset(candidates, 0, sizeof(candidates));

  for (i = 0; i < 3; ++i) {
    (void)yai_law_safe_snprintf(candidates[i].family_id, sizeof(candidates[i].family_id), "%s", families[i]);
    map_family_to_domain_id(families[i], candidates[i].domain_id, sizeof(candidates[i].domain_id));

    (void)yai_law_match_signal_score(ctx, families[i], &candidates[i].score, candidates[i].rationale, sizeof(candidates[i].rationale));
    (void)yai_law_protocol_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    (void)yai_law_provider_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    (void)yai_law_resource_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    (void)yai_law_command_match_bonus(ctx, families[i], &b); candidates[i].score += b;
    apply_classification_map_boost(ctx, families[i], &candidates[i].score);
  }

  sort_family_candidates(candidates, 3);
  out->family_candidate_count = 3;
  for (i = 0; i < 3; ++i) {
    (void)yai_law_safe_snprintf(out->family_candidates[i], sizeof(out->family_candidates[i]), "%s", candidates[i].family_id);
    out->family_candidate_scores[i] = candidates[i].score;
  }

  (void)yai_law_safe_snprintf(out->family_id, sizeof(out->family_id), "%s", candidates[0].family_id);
  (void)yai_law_safe_snprintf(out->domain_id, sizeof(out->domain_id), "%s", candidates[0].domain_id);
  out->confidence = candidates[0].score;
  (void)yai_law_safe_snprintf(out->rationale, sizeof(out->rationale), "%s", candidates[0].rationale);
  choose_specialization(ctx, out->family_id, out);

  (void)yai_law_confidence_label(out->confidence, &out->ambiguous);
  if (out->confidence < 0.45) {
    (void)yai_law_safe_snprintf(out->family_id, sizeof(out->family_id), "%s", "digital");
    (void)yai_law_safe_snprintf(out->domain_id, sizeof(out->domain_id), "%s", "D1-digital");
    (void)yai_law_safe_snprintf(out->specialization_id, sizeof(out->specialization_id), "%s", "network-egress");
    out->ambiguous = 1;
    (void)yai_law_safe_snprintf(out->rationale, sizeof(out->rationale), "%s", "low confidence fallback to digital/network-egress");
  }

  return 0;
}
