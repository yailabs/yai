/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

/* Status/code to SDK rc mapping used by protocol-first client paths. */
int yai_reply_map_rc(const char *status, const char *code);

#ifdef __cplusplus
}
#endif
