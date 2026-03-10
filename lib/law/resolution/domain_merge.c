#include "../internal.h"
#include <string.h>

int yai_law_domain_merge_apply(yai_law_discovery_result_t *discovery,
                               char *err,
                               size_t err_cap) {
  if (err && err_cap > 0) err[0] = '\0';
  if (!discovery) {
    if (err && err_cap > 0) (void)yai_law_safe_snprintf(err, err_cap, "%s", "domain_merge_missing_discovery");
    return -1;
  }

  if (!discovery->family_id[0] && discovery->domain_id[0]) {
    if (strncmp(discovery->domain_id, "D1-", 3) == 0) (void)yai_law_safe_snprintf(discovery->family_id, sizeof(discovery->family_id), "%s", "digital");
    else if (strncmp(discovery->domain_id, "D5-", 3) == 0) (void)yai_law_safe_snprintf(discovery->family_id, sizeof(discovery->family_id), "%s", "economic");
    else if (strncmp(discovery->domain_id, "D8-", 3) == 0) (void)yai_law_safe_snprintf(discovery->family_id, sizeof(discovery->family_id), "%s", "scientific");
  }

  if (!discovery->domain_id[0] && discovery->family_id[0]) {
    if (strcmp(discovery->family_id, "digital") == 0) (void)yai_law_safe_snprintf(discovery->domain_id, sizeof(discovery->domain_id), "%s", "D1-digital");
    else if (strcmp(discovery->family_id, "economic") == 0) (void)yai_law_safe_snprintf(discovery->domain_id, sizeof(discovery->domain_id), "%s", "D5-economic");
    else if (strcmp(discovery->family_id, "scientific") == 0) (void)yai_law_safe_snprintf(discovery->domain_id, sizeof(discovery->domain_id), "%s", "D8-scientific");
  }

  if (!discovery->specialization_id[0]) {
    (void)yai_law_safe_snprintf(discovery->specialization_id,
                                sizeof(discovery->specialization_id),
                                "%s",
                                "general");
  }

  if (!discovery->domain_id[0] || !discovery->family_id[0]) {
    if (err && err_cap > 0) (void)yai_law_safe_snprintf(err, err_cap, "%s", "domain_merge_incomplete");
    return -1;
  }
  return 0;
}
