/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_sdk_endpoint_kind {
    YAI_SDK_ENDPOINT_KIND_LOCAL_DEFAULT = 0,
    YAI_SDK_ENDPOINT_KIND_LOCAL_UDS = 1,
    YAI_SDK_ENDPOINT_KIND_OWNER_REF = 2,
} yai_sdk_endpoint_kind_t;

typedef enum yai_sdk_transport_kind {
    YAI_SDK_TRANSPORT_KIND_UNSPECIFIED = 0,
    YAI_SDK_TRANSPORT_KIND_UDS = 1,
} yai_sdk_transport_kind_t;

typedef struct yai_sdk_runtime_endpoint {
    yai_sdk_endpoint_kind_t endpoint_kind;
    yai_sdk_transport_kind_t transport_kind;
    char locator[512];
    char owner_ref[512];
} yai_sdk_runtime_endpoint_t;

typedef struct yai_sdk_runtime_locator_state {
    yai_sdk_endpoint_kind_t endpoint_kind;
    yai_sdk_transport_kind_t transport_kind;
    char resolved_ingress[512];
    int explicit_target;
} yai_sdk_runtime_locator_state_t;

void yai_sdk_runtime_endpoint_init(yai_sdk_runtime_endpoint_t *out);
void yai_sdk_runtime_locator_state_init(yai_sdk_runtime_locator_state_t *out);

int yai_sdk_runtime_endpoint_local_default(yai_sdk_runtime_endpoint_t *out);
int yai_sdk_runtime_endpoint_local_uds(const char *uds_path, yai_sdk_runtime_endpoint_t *out);
int yai_sdk_runtime_endpoint_owner_ref(const char *owner_ref, yai_sdk_runtime_endpoint_t *out);

/*
 * Resolve runtime endpoint into an ingress locator.
 * v1 support:
 * - local default -> canonical runtime ingress path
 * - local uds -> explicit UDS path
 * - owner_ref with uds:// scheme -> explicit UDS path
 */
int yai_sdk_runtime_endpoint_resolve(
    const yai_sdk_runtime_endpoint_t *endpoint,
    yai_sdk_runtime_locator_state_t *out_state,
    char *err,
    size_t err_cap);

#ifdef __cplusplus
}
#endif
