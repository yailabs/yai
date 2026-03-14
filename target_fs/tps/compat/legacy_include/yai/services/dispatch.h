#pragma once

#include <stddef.h>
#include <sys/types.h>

#include <yai/ipc/runtime.h>
#include <yai/ipc/transport.h>

#define YAI_CONTROL_BACKLOG 16

#define YAI_CTL_OK            0
#define YAI_CTL_ERR_SOCKET   -1
#define YAI_CTL_ERR_BIND     -2
#define YAI_CTL_ERR_LISTEN   -3
#define YAI_CTL_ERR_READ     -4
#define YAI_CTL_ERR_WRITE    -5
#define YAI_CTL_ERR_OVERFLOW -6

typedef enum yai_runtime_ingress_kind {
    YAI_RUNTIME_INGRESS_CONTROL = 0,
    YAI_RUNTIME_INGRESS_PEER = 1
} yai_runtime_ingress_kind_t;

int yai_control_listen_at(const char *path);
ssize_t yai_control_read_frame(int fd, yai_rpc_envelope_t *env, void *payload_buf, size_t payload_cap);
int yai_control_write_frame(int fd, const yai_rpc_envelope_t *env, const void *payload);

int yai_dispatch_frame(int client_fd,
                       const yai_rpc_envelope_t *env,
                       const char *payload,
                       ssize_t payload_len,
                       int *handshake_done,
                       yai_runtime_ingress_kind_t ingress_kind);

int yai_core_attach_flow_enabled(void);
