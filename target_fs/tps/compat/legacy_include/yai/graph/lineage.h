/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/graph/ids.h>

int yai_graph_lineage_summary(const char *graph_scope_id,
                              const char *anchor_ref,
                              char *out_json,
                              size_t out_cap,
                              char *err,
                              size_t err_cap);
