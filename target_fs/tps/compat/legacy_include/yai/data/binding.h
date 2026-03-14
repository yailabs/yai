/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#include <yai/data/store.h>

int yai_data_store_binding_init(char *err, size_t err_cap);
int yai_data_store_binding_attach_scope(const char *data_scope_id,
                                        char *err,
                                        size_t err_cap);
int yai_data_store_binding_is_ready(void);
const char *yai_data_store_binding_root(void);
