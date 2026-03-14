#include "../internal.h"

#include <string.h>

static const char *normalize_family(const char *id) {
  if (!id) return "";
  if (strcmp(id, "D1-digital") == 0 || strcmp(id, "D1") == 0) return "digital";
  if (strcmp(id, "D5-economic") == 0 || strcmp(id, "D5") == 0) return "economic";
  if (strcmp(id, "D8-scientific") == 0 || strcmp(id, "D8") == 0) return "scientific";
  return id;
}

int yai_governance_match_signal_score(const yai_governance_classification_ctx_t *ctx,
                               const char *domain_id,
                               double *score,
                               char *rationale,
                               size_t rationale_cap) {
  double s = 0.0;
  const char *family;
  if (!ctx || !domain_id || !score) return -1;
  family = normalize_family(domain_id);

  if (strcmp(family, "digital") == 0) {
    if (strcmp(ctx->action, "egress") == 0 || strcmp(ctx->action, "publish") == 0 || ctx->has_egress_intent) s += 0.34;
    if (strcmp(ctx->action, "retrieve") == 0 || ctx->has_retrieve_intent) s += 0.18;
    if (ctx->has_commentary_intent) s += 0.16;
    if (ctx->has_distribution_intent) s += 0.16;
    if (strstr(ctx->provider, "curl") || strstr(ctx->provider, "otel") || strstr(ctx->provider, "s3") ||
        strstr(ctx->provider, "github") || strstr(ctx->provider, "cdn")) s += 0.28;
    if (strstr(ctx->resource, "external") || strstr(ctx->resource, "bucket") || strstr(ctx->resource, "sink") ||
        strstr(ctx->resource, "endpoint")) s += 0.22;
    if (ctx->has_sink_ref) s += 0.10;
    if (ctx->sink_external) s += 0.06;
    if (rationale) (void)yai_governance_safe_snprintf(rationale, rationale_cap, "%s", "digital egress/retrieve/sink signal match");
  } else if (strcmp(family, "scientific") == 0) {
    if (strstr(ctx->action, "experiment") || strstr(ctx->action, "parameter") || strcmp(ctx->action, "infer") == 0) s += 0.40;
    if (strstr(ctx->provider, "experiment") || strstr(ctx->resource, "dataset") || strstr(ctx->resource, "experiment")) s += 0.35;
    if (ctx->has_params_hash || ctx->black_box_mode) s += 0.25;
    if (ctx->has_repro_context) s += 0.12;
    if (ctx->has_dataset_ref) s += 0.10;
    if (ctx->has_publication_intent) s += 0.08;
    if (rationale) (void)yai_governance_safe_snprintf(rationale, rationale_cap, "%s", "scientific reproducibility signal match");
  } else if (strcmp(family, "economic") == 0) {
    if (strcmp(ctx->action, "authorize") == 0 || strstr(ctx->action, "settle")) s += 0.40;
    if (strstr(ctx->provider, "payment") || strstr(ctx->provider, "ledger") || strstr(ctx->provider, "gateway")) s += 0.35;
    if (strstr(ctx->resource, "ledger") || strstr(ctx->resource, "counterparty") || strstr(ctx->resource, "instrument")) s += 0.25;
    if (rationale) (void)yai_governance_safe_snprintf(rationale, rationale_cap, "%s", "economic authorization/settlement signal match");
  } else {
    s = 0.10;
    if (rationale) (void)yai_governance_safe_snprintf(rationale, rationale_cap, "%s", "fallback low-confidence match");
  }

  *score = s;
  return 0;
}
