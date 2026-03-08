#include "../internal.h"

#include <string.h>
#include <time.h>

typedef struct yai_stack_accumulator {
  int has_baseline_authority;
  int has_review_required;
  int has_explicit_approval;
  int has_escalation;
  int has_dual_control;
  int has_human_oversight;

  int has_resolution_trace;
  int has_decision_record;
  int has_review_trace;
  int has_retention;
  int has_provenance;
  int has_approval_chain;
  int has_dependency_chain;
  int has_lawful_basis;
  int has_oversight_trace;
  int has_parameter_lock;
  int has_fraud_trace;
} yai_stack_accumulator_t;

static void add_rule(yai_law_effective_stack_t *stack, const char *id) {
  if (!stack || !id) return;
  if (stack->applied_rule_count >= YAI_LAW_RULE_MAX) return;
  (void)yai_law_safe_snprintf(stack->applied_rules[stack->applied_rule_count],
                              sizeof(stack->applied_rules[stack->applied_rule_count]),
                              "%s", id);
  stack->applied_rule_count++;
}

static void add_compliance(yai_law_effective_stack_t *stack, const char *id) {
  if (!stack || !id) return;
  if (stack->compliance_count >= YAI_LAW_COMPLIANCE_MAX) return;
  (void)yai_law_safe_snprintf(stack->compliance_layers[stack->compliance_count],
                              sizeof(stack->compliance_layers[stack->compliance_count]),
                              "%s", id);
  stack->compliance_count++;
}

static int stack_has_token(char arr[][64], int count, const char *id) {
  int i;
  if (!id) return 0;
  for (i = 0; i < count; ++i) {
    if (strcmp(arr[i], id) == 0) return 1;
  }
  return 0;
}

static void add_overlay(yai_law_effective_stack_t *stack, const char *id) {
  if (!stack || !id) return;
  if (stack_has_token(stack->overlay_layers, stack->overlay_count, id)) return;
  if (stack->overlay_count >= YAI_LAW_COMPLIANCE_MAX) return;
  (void)yai_law_safe_snprintf(stack->overlay_layers[stack->overlay_count],
                              sizeof(stack->overlay_layers[stack->overlay_count]),
                              "%s", id);
  stack->overlay_count++;
}

static void add_regulatory_overlay(yai_law_effective_stack_t *stack, const char *overlay_id) {
  char buf[64];
  if (!stack || !overlay_id) return;
  if (stack_has_token(stack->regulatory_overlays, stack->regulatory_overlay_count, overlay_id)) return;
  if (stack->regulatory_overlay_count >= YAI_LAW_COMPLIANCE_MAX) return;
  (void)yai_law_safe_snprintf(stack->regulatory_overlays[stack->regulatory_overlay_count],
                              sizeof(stack->regulatory_overlays[stack->regulatory_overlay_count]),
                              "%s",
                              overlay_id);
  stack->regulatory_overlay_count++;
  (void)yai_law_safe_snprintf(buf, sizeof(buf), "regulatory.%s", overlay_id);
  add_compliance(stack, buf);
  add_overlay(stack, buf);
}

static void add_sector_overlay(yai_law_effective_stack_t *stack, const char *overlay_id) {
  if (!stack || !overlay_id) return;
  if (stack_has_token(stack->sector_overlays, stack->sector_overlay_count, overlay_id)) return;
  if (stack->sector_overlay_count >= YAI_LAW_COMPLIANCE_MAX) return;
  (void)yai_law_safe_snprintf(stack->sector_overlays[stack->sector_overlay_count],
                              sizeof(stack->sector_overlays[stack->sector_overlay_count]),
                              "%s",
                              overlay_id);
  stack->sector_overlay_count++;
  add_compliance(stack, overlay_id);
  add_overlay(stack, overlay_id);
}

static void add_contextual_overlay(yai_law_effective_stack_t *stack, const char *overlay_id) {
  char full[64];
  if (!stack || !overlay_id) return;
  if (stack_has_token(stack->contextual_overlays, stack->contextual_overlay_count, overlay_id)) return;
  if (stack->contextual_overlay_count >= YAI_LAW_COMPLIANCE_MAX) return;
  (void)yai_law_safe_snprintf(stack->contextual_overlays[stack->contextual_overlay_count],
                              sizeof(stack->contextual_overlays[stack->contextual_overlay_count]),
                              "%s",
                              overlay_id);
  stack->contextual_overlay_count++;
  (void)yai_law_safe_snprintf(full, sizeof(full), "contextual.%s", overlay_id);
  add_overlay(stack, full);
}

static void add_authority_contributor(yai_law_effective_stack_t *stack, const char *id) {
  if (!stack || !id) return;
  if (stack_has_token(stack->authority_contributors, stack->authority_contributor_count, id)) return;
  if (stack->authority_contributor_count >= YAI_LAW_CONTRIBUTOR_MAX) return;
  (void)yai_law_safe_snprintf(stack->authority_contributors[stack->authority_contributor_count],
                              sizeof(stack->authority_contributors[stack->authority_contributor_count]),
                              "%s",
                              id);
  stack->authority_contributor_count++;
}

static void add_evidence_contributor(yai_law_effective_stack_t *stack, const char *id) {
  if (!stack || !id) return;
  if (stack_has_token(stack->evidence_contributors, stack->evidence_contributor_count, id)) return;
  if (stack->evidence_contributor_count >= YAI_LAW_CONTRIBUTOR_MAX) return;
  (void)yai_law_safe_snprintf(stack->evidence_contributors[stack->evidence_contributor_count],
                              sizeof(stack->evidence_contributors[stack->evidence_contributor_count]),
                              "%s",
                              id);
  stack->evidence_contributor_count++;
}

static int read_json_from_surface(const char *embedded_rel, char *out, size_t out_cap) {
  if (!embedded_rel || !out || out_cap == 0) return -1;
  return yai_law_read_text_file(embedded_rel, out, out_cap);
}

static void attach_overlays_from_specialization_manifest(const yai_law_discovery_result_t *discovery,
                                                         yai_law_effective_stack_t *stack) {
  char path_embedded[256];
  char json[8192];
  if (!discovery || !stack) return;
  if (yai_law_safe_snprintf(path_embedded, sizeof(path_embedded), "embedded/law/domain-specializations/%s/manifest.json",
                            discovery->specialization_id) != 0) return;
  if (read_json_from_surface(path_embedded, json, sizeof(json)) != 0) return;

  if (yai_law_json_contains(json, "\"gdpr-eu\"")) add_regulatory_overlay(stack, "gdpr-eu");
  if (yai_law_json_contains(json, "\"ai-act\"")) add_regulatory_overlay(stack, "ai-act");
  if (yai_law_json_contains(json, "\"retention-governance\"")) add_regulatory_overlay(stack, "retention-governance");
  if (yai_law_json_contains(json, "\"security-supply-chain\"")) add_regulatory_overlay(stack, "security-supply-chain");

  if (yai_law_json_contains(json, "\"sector.finance\"")) add_sector_overlay(stack, "sector.finance");
  if (yai_law_json_contains(json, "\"sector.healthcare\"")) add_sector_overlay(stack, "sector.healthcare");
  if (yai_law_json_contains(json, "\"sector.public-sector\"")) add_sector_overlay(stack, "sector.public-sector");

  if (yai_law_json_contains(json, "\"organization\"")) add_contextual_overlay(stack, "organization");
  if (yai_law_json_contains(json, "\"workspace\"")) add_contextual_overlay(stack, "workspace");
  if (yai_law_json_contains(json, "\"session\"")) add_contextual_overlay(stack, "session");
  if (yai_law_json_contains(json, "\"experimental\"")) add_contextual_overlay(stack, "experimental");
}

static void attach_overlays_from_matrix(const yai_law_classification_ctx_t *ctx,
                                        const yai_law_discovery_result_t *discovery,
                                        yai_law_effective_stack_t *stack) {
  char json[12288];
  char fam[96];
  char spec[128];
  char action[96];
  if (!ctx || !discovery || !stack) return;
  if (yai_law_safe_snprintf(fam, sizeof(fam), "\"family\":\"%s\"", discovery->family_id) != 0) return;
  if (yai_law_safe_snprintf(spec, sizeof(spec), "\"specialization\":\"%s\"", discovery->specialization_id) != 0) return;
  if (yai_law_safe_snprintf(action, sizeof(action), "\"action\":\"%s\"", ctx->action) != 0) return;

  if (read_json_from_surface("embedded/law/overlays/regulatory/index/overlay-attachment-matrix.json",
                             json, sizeof(json)) == 0 &&
      yai_law_json_contains(json, fam) && yai_law_json_contains(json, spec) && yai_law_json_contains(json, action)) {
    if (yai_law_json_contains(json, "\"overlay\":\"gdpr-eu\"")) add_regulatory_overlay(stack, "gdpr-eu");
    if (yai_law_json_contains(json, "\"overlay\":\"ai-act\"")) add_regulatory_overlay(stack, "ai-act");
    if (yai_law_json_contains(json, "\"overlay\":\"retention-governance\"")) add_regulatory_overlay(stack, "retention-governance");
    if (yai_law_json_contains(json, "\"overlay\":\"security-supply-chain\"")) add_regulatory_overlay(stack, "security-supply-chain");
    if (yai_law_json_contains(json, "\"overlay\":\"sector.finance\"")) add_sector_overlay(stack, "sector.finance");
  }

  if (read_json_from_surface("embedded/law/overlays/sector/index/overlay-attachment-matrix.json",
                             json, sizeof(json)) == 0 &&
      yai_law_json_contains(json, fam) && yai_law_json_contains(json, spec)) {
    if (yai_law_json_contains(json, "\"overlay\":\"sector.finance\"")) add_sector_overlay(stack, "sector.finance");
    if (yai_law_json_contains(json, "\"overlay\":\"sector.healthcare\"")) add_sector_overlay(stack, "sector.healthcare");
    if (yai_law_json_contains(json, "\"overlay\":\"sector.public-sector\"")) add_sector_overlay(stack, "sector.public-sector");
  }
}

static void build_profile_from_contributors(char arr[][64], int n, char *out, size_t out_cap) {
  int i;
  size_t used = 0;
  if (!out || out_cap == 0) return;
  out[0] = '\0';
  for (i = 0; i < n; ++i) {
    int rc;
    if (arr[i][0] == '\0') continue;
    rc = yai_law_safe_snprintf(out + used, out_cap - used, "%s%s", used == 0 ? "" : "+", arr[i]);
    if (rc != 0) break;
    used = strlen(out);
    if (used >= out_cap - 1) break;
  }
}

int yai_law_stack_build(const yai_law_runtime_t *rt,
                        const yai_law_discovery_result_t *discovery,
                        const yai_law_classification_ctx_t *ctx,
                        yai_law_effective_stack_t *stack,
                        yai_law_effect_t *effect,
                        char *rationale,
                        size_t rationale_cap) {
  yai_stack_accumulator_t agg;
  (void)rt;
  if (!discovery || !ctx || !stack || !effect) return -1;

  memset(stack, 0, sizeof(*stack));
  memset(&agg, 0, sizeof(agg));
  (void)yai_law_safe_snprintf(stack->stack_id, sizeof(stack->stack_id), "stack-%ld", (long)time(NULL));
  (void)yai_law_safe_snprintf(stack->domain_id, sizeof(stack->domain_id), "%s", discovery->domain_id);
  (void)yai_law_safe_snprintf(stack->family_id, sizeof(stack->family_id), "%s", discovery->family_id);
  (void)yai_law_safe_snprintf(stack->specialization_id, sizeof(stack->specialization_id), "%s", discovery->specialization_id);

  attach_overlays_from_specialization_manifest(discovery, stack);
  attach_overlays_from_matrix(ctx, discovery, stack);

  add_authority_contributor(stack, "baseline_authority");
  add_evidence_contributor(stack, "resolution_trace");
  add_evidence_contributor(stack, "decision_record");
  agg.has_baseline_authority = 1;
  agg.has_resolution_trace = 1;
  agg.has_decision_record = 1;

  if (strcmp(discovery->family_id, "economic") == 0) {
    if (stack->sector_overlay_count == 0) add_sector_overlay(stack, "sector.finance");
    add_contextual_overlay(stack, "organization");
    add_contextual_overlay(stack, "workspace");
    *effect = YAI_LAW_EFFECT_REVIEW_REQUIRED;
    add_rule(stack, "economic.baseline.review");
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "economic specialization baseline review");
    agg.has_review_required = 1;
    agg.has_retention = 1;
  } else if (strcmp(discovery->family_id, "scientific") == 0) {
    if (stack->regulatory_overlay_count == 0) add_regulatory_overlay(stack, "ai-act");
    add_contextual_overlay(stack, "experimental");
    *effect = YAI_LAW_EFFECT_ALLOW;
    add_rule(stack, "scientific.baseline.allow");
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "scientific specialization baseline");
  } else {
    if (stack->regulatory_overlay_count == 0) {
      add_regulatory_overlay(stack, "gdpr-eu");
      add_regulatory_overlay(stack, "retention-governance");
    }
    add_contextual_overlay(stack, "workspace");
    *effect = YAI_LAW_EFFECT_REVIEW_REQUIRED;
    add_rule(stack, "digital.baseline.review");
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "digital specialization baseline review");
    agg.has_review_required = 1;
    agg.has_review_trace = 1;
  }

  if (strcmp(discovery->specialization_id, "settlements") == 0 || strcmp(discovery->specialization_id, "transfers") == 0) {
    add_rule(stack, "specialization.high-impact.dual-control");
    agg.has_dual_control = 1;
    agg.has_approval_chain = 1;
    agg.has_review_required = 1;
    *effect = YAI_LAW_EFFECT_REVIEW_REQUIRED;
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "high-impact specialization requires dual-control review");
  }
  if (strcmp(discovery->specialization_id, "fraud-risk-controls") == 0 || strstr(ctx->command, "fraud") || strstr(ctx->resource, "anomaly")) {
    add_rule(stack, "specialization.fraud.quarantine");
    agg.has_fraud_trace = 1;
    agg.has_provenance = 1;
    agg.has_escalation = 1;
    *effect = YAI_LAW_EFFECT_QUARANTINE;
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "fraud-risk controls triggered");
  }
  if (strcmp(discovery->specialization_id, "parameter-governance") == 0 && !ctx->has_params_hash) {
    add_rule(stack, "specialization.parameter-lock.required");
    agg.has_parameter_lock = 1;
    agg.has_review_trace = 1;
    agg.has_review_required = 1;
    *effect = YAI_LAW_EFFECT_DENY;
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "missing parameter lock");
  }
  if (strcmp(discovery->specialization_id, "black-box-evaluation") == 0 || ctx->black_box_mode) {
    add_rule(stack, "specialization.black-box.review");
    agg.has_review_required = 1;
    agg.has_review_trace = 1;
    *effect = YAI_LAW_EFFECT_REVIEW_REQUIRED;
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "black-box evaluation requires review");
  }

  if (stack_has_token(stack->regulatory_overlays, stack->regulatory_overlay_count, "security-supply-chain") &&
      (strstr(ctx->provider, "untrusted") || strstr(ctx->provider, "unknown"))) {
    add_rule(stack, "overlay.security-supply-chain.quarantine");
    agg.has_escalation = 1;
    agg.has_provenance = 1;
    agg.has_dependency_chain = 1;
    agg.has_review_trace = 1;
    *effect = YAI_LAW_EFFECT_QUARANTINE;
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "security supply-chain quarantine for untrusted provider");
  }

  if (stack_has_token(stack->regulatory_overlays, stack->regulatory_overlay_count, "gdpr-eu") &&
      (strstr(ctx->command, "personal") || strstr(ctx->resource, "personal") || strstr(ctx->resource, "sensitive"))) {
    add_rule(stack, "overlay.gdpr.personal-data.review");
    if (*effect != YAI_LAW_EFFECT_DENY && *effect != YAI_LAW_EFFECT_QUARANTINE) *effect = YAI_LAW_EFFECT_REVIEW_REQUIRED;
    agg.has_review_required = 1;
    agg.has_lawful_basis = 1;
    agg.has_retention = 1;
    agg.has_provenance = 1;
    (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "gdpr-eu overlay requires personal-data review");
  }

  if (stack_has_token(stack->regulatory_overlays, stack->regulatory_overlay_count, "ai-act")) {
    if (strstr(ctx->command, "prohibited")) {
      add_rule(stack, "overlay.ai-act.prohibited-use.deny");
      agg.has_explicit_approval = 1;
      agg.has_oversight_trace = 1;
      agg.has_provenance = 1;
      *effect = YAI_LAW_EFFECT_DENY;
      (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "ai-act prohibited use");
    } else if (strstr(ctx->command, "high-risk")) {
      add_rule(stack, "overlay.ai-act.high-risk.review");
      if (*effect != YAI_LAW_EFFECT_DENY && *effect != YAI_LAW_EFFECT_QUARANTINE) *effect = YAI_LAW_EFFECT_REVIEW_REQUIRED;
      agg.has_human_oversight = 1;
      agg.has_oversight_trace = 1;
      agg.has_review_required = 1;
      agg.has_retention = 1;
      (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "ai-act high-risk review");
    }
  }

  if (stack_has_token(stack->sector_overlays, stack->sector_overlay_count, "sector.finance") &&
      strcmp(discovery->family_id, "economic") == 0) {
    add_rule(stack, "overlay.sector.finance.hardening");
    agg.has_dual_control = 1;
    agg.has_approval_chain = 1;
    agg.has_retention = 1;
    agg.has_provenance = 1;
    if (!ctx->has_authority_contract) {
      agg.has_explicit_approval = 1;
      *effect = YAI_LAW_EFFECT_DENY;
      (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "finance overlay requires authority contract");
    }
  }

  if (stack_has_token(stack->contextual_overlays, stack->contextual_overlay_count, "experimental")) {
    agg.has_review_trace = 1;
  }
  if (!ctx->has_authority_contract && (strcmp(ctx->action, "publish") == 0 || strcmp(ctx->action, "egress") == 0 ||
                                       strcmp(ctx->action, "authorize") == 0 || strcmp(ctx->action, "settle") == 0)) {
    add_rule(stack, "authority.contract.missing");
    agg.has_explicit_approval = 1;
    if (*effect != YAI_LAW_EFFECT_DENY && *effect != YAI_LAW_EFFECT_QUARANTINE) {
      *effect = YAI_LAW_EFFECT_DENY;
      (void)yai_law_safe_snprintf(rationale, rationale_cap, "%s", "authority contract missing for sensitive action");
    }
  }

  if (agg.has_review_required) add_authority_contributor(stack, "review_required");
  if (agg.has_explicit_approval) add_authority_contributor(stack, "explicit_approval_required");
  if (agg.has_escalation) add_authority_contributor(stack, "escalation_required");
  if (agg.has_dual_control) add_authority_contributor(stack, "dual_control_required");
  if (agg.has_human_oversight) add_authority_contributor(stack, "human_oversight_required");

  if (agg.has_resolution_trace) add_evidence_contributor(stack, "resolution_trace");
  if (agg.has_decision_record) add_evidence_contributor(stack, "decision_record");
  if (agg.has_review_trace) add_evidence_contributor(stack, "review_trace_required");
  if (agg.has_retention) add_evidence_contributor(stack, "retention_required");
  if (agg.has_provenance) add_evidence_contributor(stack, "provenance_required");
  if (agg.has_approval_chain) add_evidence_contributor(stack, "approval_chain_required");
  if (agg.has_dependency_chain) add_evidence_contributor(stack, "dependency_chain_ref");
  if (agg.has_lawful_basis) add_evidence_contributor(stack, "lawful_basis_trace");
  if (agg.has_oversight_trace) add_evidence_contributor(stack, "oversight_trace_required");
  if (agg.has_parameter_lock) add_evidence_contributor(stack, "parameter_lock_required");
  if (agg.has_fraud_trace) add_evidence_contributor(stack, "fraud_trace_required");

  build_profile_from_contributors(stack->authority_contributors,
                                  stack->authority_contributor_count,
                                  stack->authority_profile,
                                  sizeof(stack->authority_profile));
  build_profile_from_contributors(stack->evidence_contributors,
                                  stack->evidence_contributor_count,
                                  stack->evidence_profile,
                                  sizeof(stack->evidence_profile));

  if (stack->authority_profile[0] == '\0') {
    (void)yai_law_safe_snprintf(stack->authority_profile, sizeof(stack->authority_profile), "%s", "baseline_authority");
  }
  if (stack->evidence_profile[0] == '\0') {
    (void)yai_law_safe_snprintf(stack->evidence_profile, sizeof(stack->evidence_profile), "%s", "resolution_trace");
  }

  (void)yai_law_safe_snprintf(stack->precedence_trace,
                              sizeof(stack->precedence_trace),
                              "%s",
                              "specialization_baseline + overlays => deny > quarantine > review_required > allow");
  return 0;
}
