/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_SDK_ABI_VERSION 1

int yai_sdk_abi_version(void);
const char *yai_sdk_version(void);

/* Canonical unified-runtime public taxonomy. */
#include <yai/sdk/core.h>
#include <yai/sdk/runtime.h>
#include <yai/sdk/models.h>
#include <yai/sdk/targets.h>
#include <yai/sdk/transport.h>
#include <yai/sdk/container.h>
#include <yai/sdk/exec.h>
#include <yai/sdk/db.h>
#include <yai/sdk/data.h>
#include <yai/sdk/graph.h>
#include <yai/sdk/cognition.h>
#include <yai/sdk/source.h>
#include <yai/sdk/policy.h>
#include <yai/sdk/recovery.h>
#include <yai/sdk/debug.h>
#include <yai/sdk/governance.h>

/* Stable legacy module surface retained for compatibility. */
#include <yai/sdk/errors.h>
#include <yai/sdk/paths.h>
#include <yai/sdk/context.h>
#include <yai/sdk/client.h>
#include <yai/sdk/catalog.h>
#include <yai/sdk/protocol.h>
#include <yai/sdk/rpc.h>
#include <yai/sdk/log.h>
#include <yai/sdk/reply/reply.h>
#include <yai/sdk/reply/reply_builder.h>
#include <yai/sdk/reply/reply_json.h>

#ifdef __cplusplus
}
#endif
