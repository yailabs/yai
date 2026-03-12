/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/memory.h>

int yai_graph_query_summary(const char *graph_scope_id,
                            char *out_json,
                            size_t out_cap,
                            char *err,
                            size_t err_cap);
int yai_graph_query_unified_summary(const char *graph_scope_id,
                                    char *out_json,
                                    size_t out_cap,
                                    char *err,
                                    size_t err_cap);
