#include "../internal.h"

#include <string.h>

static const char *normalize_family(const char *id) {
  if (!id) return "";
  if (strcmp(id, "D1-digital") == 0 || strcmp(id, "D1") == 0) return "digital";
  if (strcmp(id, "D5-economic") == 0 || strcmp(id, "D5") == 0) return "economic";
  if (strcmp(id, "D8-scientific") == 0 || strcmp(id, "D8") == 0) return "scientific";
  return id;
}

int yai_governance_resource_match_bonus(const yai_governance_classification_ctx_t *ctx, const char *domain_id, double *bonus) {
  const char *family;
  if (!ctx || !domain_id || !bonus) return -1;
  *bonus = 0.0;
  family = normalize_family(domain_id);
  if (strcmp(family, "digital") == 0 && strstr(ctx->resource, "external")) *bonus = 0.05;
  if (strcmp(family, "digital") == 0 &&
      (strstr(ctx->resource, "sink") || strstr(ctx->resource, "channel") ||
       strstr(ctx->resource, "distribution") || strstr(ctx->resource, "artifact"))) *bonus += 0.04;
  if (strcmp(family, "economic") == 0 &&
      (strstr(ctx->resource, "ledger") || strstr(ctx->resource, "counterparty") || strstr(ctx->resource, "instrument"))) *bonus = 0.05;
  if (strcmp(family, "scientific") == 0 &&
      (strstr(ctx->resource, "dataset") || strstr(ctx->resource, "experiment") || strstr(ctx->resource, "artifact"))) *bonus = 0.05;
  return 0;
}
