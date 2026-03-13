/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

typedef struct yai_rpc_client {
    int fd;
    char scope_id[128];
    uint8_t role;
    uint8_t arming;
    char correlation_id[64];
    uint32_t trace_seq;
} yai_rpc_client_t;

int yai_rpc_connect(yai_rpc_client_t *c, const char *scope_id);
int yai_rpc_connect_at(yai_rpc_client_t *c, const char *scope_id, const char *sock_path);
void yai_rpc_close(yai_rpc_client_t *c);
void yai_rpc_set_authority(yai_rpc_client_t *c, int arming, const char *role_str);
void yai_rpc_set_correlation_id(yai_rpc_client_t *c, const char *correlation_id);

int yai_rpc_call_raw(
    yai_rpc_client_t *c,
    uint32_t command_id,
    const void *payload,
    uint32_t payload_len,
    void *out_buf,
    size_t out_cap,
    uint32_t *out_len);

int yai_rpc_handshake(yai_rpc_client_t *c);

#ifdef __cplusplus
}
#endif
