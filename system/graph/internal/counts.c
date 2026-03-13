/* SPDX-License-Identifier: Apache-2.0 */

#include "counts.h"

#include <yai/data/query.h>

size_t yai_graph_internal_query_count_or_zero(const char *workspace_id,
                                              const char *record_class)
{
  size_t out = 0;
  char err[64];
  if (yai_data_query_count(workspace_id, record_class, &out, err, sizeof(err)) != 0) {
    return 0;
  }
  return out;
}
