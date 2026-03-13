/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/client.h>
#include <yai/sdk/rpc.h>

struct yai_sdk_client {
    yai_rpc_client_t rpc;
    char scope_id[128];
    char uds_path[512];
    char role[32];
    char correlation_id[64];
    int arming;
    int auto_handshake;
    int is_open;
    int handshaken;
    yai_sdk_runtime_locator_state_t locator_state;
    int connect_timeout_ms;
    int connect_attempts;
};
