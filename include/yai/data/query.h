/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/data/records.h>

int yai_data_query_count(const char *workspace_id,
                         const char *record_class,
                         size_t *out_count,
                         char *err,
                         size_t err_cap);
int yai_data_query_summary_json(const char *workspace_id,
                                char *out_json,
                                size_t out_cap,
                                char *err,
                                size_t err_cap);
int yai_data_query_tail_json(const char *workspace_id,
                             const char *record_class,
                             size_t limit,
                             char *out_json,
                             size_t out_cap,
                             char *err,
                             size_t err_cap);
