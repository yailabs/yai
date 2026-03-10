/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

int yai_data_store_paths(const char *workspace_id,
                         const char *record_class,
                         char *dir_out,
                         size_t dir_cap,
                         char *file_out,
                         size_t file_cap,
                         char *err,
                         size_t err_cap);
int yai_data_store_count_lines(const char *path, size_t *out_count, char *err, size_t err_cap);

int yai_data_duckdb_append(const char *workspace_id,
                           const char *record_class,
                           const char *record_key,
                           const char *record_json,
                           char *err,
                           size_t err_cap);
int yai_data_duckdb_count(const char *workspace_id,
                          const char *record_class,
                          size_t *out_count,
                          char *err,
                          size_t err_cap);
