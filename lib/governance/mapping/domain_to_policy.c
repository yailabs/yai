#include "../internal.h"

#include <string.h>

int yai_law_domain_to_policy_pack(const char *domain_id, const char *action, char *out, size_t out_cap) {
  char family[64];
  if (!domain_id || !action || !out || out_cap == 0) return -1;
  family[0] = '\0';
  (void)yai_law_domain_model_lookup(domain_id,
                                    NULL,
                                    0,
                                    family,
                                    sizeof(family),
                                    NULL,
                                    0,
                                    NULL,
                                    0,
                                    NULL,
                                    0);
  if (family[0] == '\0') {
    if (strncmp(domain_id, "D1-", 3) == 0) (void)yai_law_safe_snprintf(family, sizeof(family), "%s", "digital");
    else if (strncmp(domain_id, "D5-", 3) == 0) (void)yai_law_safe_snprintf(family, sizeof(family), "%s", "economic");
    else if (strncmp(domain_id, "D8-", 3) == 0) (void)yai_law_safe_snprintf(family, sizeof(family), "%s", "scientific");
  }

  if (strcmp(family, "digital") == 0) {
    if (strcmp(action, "egress") == 0 || strcmp(action, "publish") == 0)
      return yai_law_safe_snprintf(out, out_cap, "%s", "D1-digital.baseline");
    return yai_law_safe_snprintf(out, out_cap, "%s", "D1-digital.review");
  }
  if (strcmp(family, "economic") == 0) {
    if (strcmp(action, "authorize") == 0)
      return yai_law_safe_snprintf(out, out_cap, "%s", "D5-economic.payments.baseline");
    if (strcmp(action, "settle") == 0)
      return yai_law_safe_snprintf(out, out_cap, "%s", "D5-economic.settlements.review");
    return yai_law_safe_snprintf(out, out_cap, "%s", "D5-economic.review");
  }
  if (strcmp(family, "scientific") == 0) {
    return yai_law_safe_snprintf(out, out_cap, "%s", "D8-scientific.baseline");
  }
  return yai_law_safe_snprintf(out, out_cap, "%s", "unknown");
}
