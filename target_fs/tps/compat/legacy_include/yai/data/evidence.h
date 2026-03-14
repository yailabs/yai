/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int yai_data_evidence_append(const char *data_scope_id,
                             const char *evidence_json,
                             char *out_ref,
                             size_t out_ref_cap,
                             char *err,
                             size_t err_cap);

int yai_data_evidence_summary_json(const char *data_scope_id,
                                   char *out_json,
                                   size_t out_cap,
                                   char *err,
                                   size_t err_cap);

#ifdef __cplusplus
}
#endif
