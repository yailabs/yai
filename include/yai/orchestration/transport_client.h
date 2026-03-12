#pragma once

#include <stdbool.h>
#include <stdint.h>

#include <yai/protocol/contracts/transport.h>

typedef struct yai_rpc_client {
  int fd;
  char ws_id[36];
  uint32_t session_id;
  bool connected;
} yai_rpc_client_t;

/* Runtime ingress resolution: transport concerns only, independent from governance/registry compatibility data. */
int yai_runtime_ingress_path(char *out, uint32_t out_cap);
int yai_runtime_peer_ingress_path(char *out, uint32_t out_cap);
int yai_rpc_connect_at_ingress(yai_rpc_client_t *c, const char *ws_id, const char *socket_path);
int yai_rpc_connect(yai_rpc_client_t *c, const char *ws_id);
void yai_rpc_close(yai_rpc_client_t *c);
int yai_rpc_handshake(yai_rpc_client_t *c, uint32_t capabilities);
int yai_rpc_call(yai_rpc_client_t *c,
                 uint32_t command_id,
                 const void *payload,
                 uint32_t payload_len,
                 void *out_buf,
                 uint32_t out_cap,
                 uint32_t *out_len);
void yai_make_trace_id(char out[36]);
