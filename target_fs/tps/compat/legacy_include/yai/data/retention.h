/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/data/records.h>

int yai_data_retention_prune_tail(const char *data_scope_id,
                                  const char *record_class,
                                  size_t keep_last,
                                  size_t *pruned_out,
                                  char *err,
                                  size_t err_cap);
