/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Print a response payload (text or json), used by shell porcelain.
void yai_print_response(const char *payload, int json_mode);

#ifdef __cplusplus
} // extern "C"
#endif
