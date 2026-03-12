/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/reply/reply.h>

#ifdef __cplusplus
extern "C" {
#endif

char *yai_reply_to_json(const yai_reply_t *r);

#ifdef __cplusplus
}
#endif
