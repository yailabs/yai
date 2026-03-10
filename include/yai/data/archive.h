/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/data/retention.h>

int yai_data_archive_rotate_class(const char *workspace_id,
                                  const char *record_class,
                                  char *out_archive_path,
                                  size_t out_archive_path_cap,
                                  char *err,
                                  size_t err_cap);
