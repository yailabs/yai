#include "../internal.h"

#include <string.h>

static const char *normalize_family(const char *id) {
  if (!id) return "";
  if (strcmp(id, "D1-digital") == 0 || strcmp(id, "D1") == 0) return "digital";
  if (strcmp(id, "D5-economic") == 0 || strcmp(id, "D5") == 0) return "economic";
  if (strcmp(id, "D8-scientific") == 0 || strcmp(id, "D8") == 0) return "scientific";
  return id;
}

int yai_governance_protocol_match_bonus(const yai_governance_classification_ctx_t *ctx, const char *domain_id, double *bonus) {
  const char *family;
  if (!ctx || !domain_id || !bonus) return -1;
  *bonus = 0.0;
  family = normalize_family(domain_id);
  if (strcmp(family, "digital") == 0 && (ctx->protocol[0] == 'h' || ctx->protocol[0] == 'g')) *bonus = 0.05;
  if (strcmp(family, "economic") == 0 && (ctx->protocol[0] == 'h' || ctx->protocol[0] == 'q')) *bonus = 0.05;
  if (strcmp(family, "scientific") == 0 && ctx->protocol[0] == 'c') *bonus = 0.05;
  return 0;
}
