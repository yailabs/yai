/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/pol/governance/publication.h>

#include <stdio.h>

int yai_governance_publication_status(const char *scope_id, char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  if (!scope_id || !scope_id[0]) scope_id = "user";
  return (snprintf(out, out_cap, "publication-ready:%s", scope_id) < (int)out_cap) ? 0 : -1;
}
