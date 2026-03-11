#include "../internal.h"
#include <string.h>

int yai_law_domain_merge_apply(yai_law_discovery_result_t *discovery,
                               char *err,
                               size_t err_cap) {
  char family[64];
  char domain_id[64];
  char default_specialization[96];
  if (err && err_cap > 0) err[0] = '\0';
  if (!discovery) {
    if (err && err_cap > 0) (void)yai_law_safe_snprintf(err, err_cap, "%s", "domain_merge_missing_discovery");
    return -1;
  }

  family[0] = '\0';
  domain_id[0] = '\0';
  default_specialization[0] = '\0';

  if (!discovery->family_id[0] && discovery->domain_id[0]) {
    if (yai_law_domain_model_lookup(discovery->domain_id,
                                    NULL,
                                    0,
                                    family,
                                    sizeof(family),
                                    NULL,
                                    0,
                                    NULL,
                                    0,
                                    default_specialization,
                                    sizeof(default_specialization)) == 0 &&
        family[0]) {
      (void)yai_law_safe_snprintf(discovery->family_id, sizeof(discovery->family_id), "%s", family);
    }
  }

  if (!discovery->domain_id[0] && discovery->family_id[0]) {
    if (yai_law_domain_model_family_resolution(discovery->family_id,
                                               domain_id,
                                               sizeof(domain_id),
                                               default_specialization,
                                               sizeof(default_specialization),
                                               NULL,
                                               0,
                                               NULL) == 0 &&
        domain_id[0]) {
      (void)yai_law_safe_snprintf(discovery->domain_id, sizeof(discovery->domain_id), "%s", domain_id);
    }
  }

  if (!discovery->specialization_id[0]) {
    if (default_specialization[0] == '\0' && discovery->family_id[0]) {
      (void)yai_law_domain_model_family_resolution(discovery->family_id,
                                                   NULL,
                                                   0,
                                                   default_specialization,
                                                   sizeof(default_specialization),
                                                   NULL,
                                                   0,
                                                   NULL);
    }
    (void)yai_law_safe_snprintf(discovery->specialization_id,
                                sizeof(discovery->specialization_id),
                                "%s",
                                default_specialization[0] ? default_specialization : "general");
  }

  if (!discovery->domain_id[0] || !discovery->family_id[0]) {
    if (err && err_cap > 0) (void)yai_law_safe_snprintf(err, err_cap, "%s", "domain_merge_incomplete");
    return -1;
  }
  return 0;
}
