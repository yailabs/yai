/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/knowledge/memory.h>

int yai_graph_query_workspace_summary(const char *workspace_id,
                                      char *out_json,
                                      size_t out_cap,
                                      char *err,
                                      size_t err_cap);
