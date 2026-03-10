/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/graph/query.h>

int yai_graph_query_recent_summary(const char *workspace_id,
                                   size_t limit,
                                   char *out_json,
                                   size_t out_cap,
                                   char *err,
                                   size_t err_cap);
