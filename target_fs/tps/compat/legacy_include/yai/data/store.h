/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

int yai_data_store_init_scope(const char *data_scope_id, char *err, size_t err_cap);
int yai_data_store_append(const char *data_scope_id,
                          const char *record_class,
                          const char *record_json,
                          char *out_ref,
                          size_t out_ref_cap,
                          char *err,
                          size_t err_cap);
int yai_data_store_tail_json(const char *data_scope_id,
                             const char *record_class,
                             size_t limit,
                             char *out_json,
                             size_t out_cap,
                             char *err,
                             size_t err_cap);
