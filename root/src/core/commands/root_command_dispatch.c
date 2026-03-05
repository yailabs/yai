/* SPDX-License-Identifier: Apache-2.0 */
#define _POSIX_C_SOURCE 200809L

#include "root_command_dispatch.h"
#include "control_transport.h"
#include "ws_id.h"

#include <errors.h>
#include <protocol.h>
#include <roles.h>
#include <yai_protocol_ids.h>

#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

static int send_response(int fd,
                         const yai_rpc_envelope_t *req,
                         uint32_t cmd,
                         const void *payload,
                         uint32_t payload_len)
{
    yai_rpc_envelope_t resp;
    memset(&resp, 0, sizeof(resp));

    resp.magic = YAI_FRAME_MAGIC;
    resp.version = YAI_PROTOCOL_IDS_VERSION;
    resp.command_id = cmd;
    resp.payload_len = payload_len;
    snprintf(resp.ws_id, sizeof(resp.ws_id), "%s", req->ws_id);
    snprintf(resp.trace_id, sizeof(resp.trace_id), "%s", req->trace_id);
    resp.role = req->role;
    resp.arming = req->arming;
    resp.checksum = 0;

    return yai_control_write_frame(fd, &resp, payload);
}

static int send_error_response(int fd,
                               const yai_rpc_envelope_t *req,
                               uint32_t code,
                               const char *reason)
{
    char payload[256];
    int n = snprintf(payload,
                     sizeof(payload),
                     "{\"status\":\"error\",\"code\":%u,\"reason\":\"%s\"}",
                     code,
                     reason ? reason : "unknown");
    if (n <= 0 || (size_t)n >= sizeof(payload))
        return -1;

    yai_rpc_envelope_t safe_req;
    memset(&safe_req, 0, sizeof(safe_req));
    if (req)
        safe_req = *req;

    return send_response(fd,
                         &safe_req,
                         safe_req.command_id ? safe_req.command_id : YAI_CMD_CONTROL,
                         payload,
                         (uint32_t)n);
}

static int is_valid_role(uint16_t role)
{
    return role == YAI_ROLE_NONE ||
           role == YAI_ROLE_USER ||
           role == YAI_ROLE_OPERATOR ||
           role == YAI_ROLE_SYSTEM;
}

static int connect_kernel_socket(void)
{
    const char *home = getenv("HOME");
    if (!home || !home[0])
        home = "/tmp";

    char path[PATH_MAX];
    snprintf(path, sizeof(path), "%s/.yai/run/kernel/control.sock", home);

    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0)
        return -1;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s", path);

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0)
    {
        close(fd);
        return -1;
    }

    return fd;
}

static int forward_to_kernel_and_relay(int client_fd,
                                       const yai_rpc_envelope_t *env,
                                       const void *payload,
                                       uint32_t payload_len)
{
    int kfd = connect_kernel_socket();
    if (kfd < 0)
    {
        (void)send_error_response(client_fd, env, YAI_E_INTERNAL_ERROR, "kernel_connect_failed");
        return -1;
    }

    yai_rpc_envelope_t kreq;
    memset(&kreq, 0, sizeof(kreq));
    kreq.magic = YAI_FRAME_MAGIC;
    kreq.version = YAI_PROTOCOL_IDS_VERSION;
    kreq.command_id = YAI_CMD_HANDSHAKE;
    kreq.payload_len = (uint32_t)sizeof(yai_handshake_req_t);
    snprintf(kreq.ws_id, sizeof(kreq.ws_id), "%s", env->ws_id);
    snprintf(kreq.trace_id, sizeof(kreq.trace_id), "%s", env->trace_id);
    kreq.role = env->role;
    kreq.arming = env->arming;
    kreq.checksum = 0;

    yai_handshake_req_t hs;
    memset(&hs, 0, sizeof(hs));
    hs.client_version = YAI_PROTOCOL_IDS_VERSION;
    hs.capabilities_requested = 0;
    snprintf(hs.client_name, sizeof(hs.client_name), "yai-root");

    if (yai_control_write_frame(kfd, &kreq, &hs) != 0)
    {
        close(kfd);
        (void)send_error_response(client_fd, env, YAI_E_INTERNAL_ERROR, "kernel_handshake_write_failed");
        return -1;
    }

    yai_rpc_envelope_t kresp;
    char kpayload[YAI_MAX_PAYLOAD];
    ssize_t hr = yai_control_read_frame(kfd, &kresp, kpayload, sizeof(kpayload));
    if (hr < 0 || kresp.command_id != YAI_CMD_HANDSHAKE)
    {
        close(kfd);
        (void)send_error_response(client_fd, env, YAI_E_INTERNAL_ERROR, "kernel_handshake_read_failed");
        return -1;
    }

    if (yai_control_write_frame(kfd, env, payload_len ? payload : NULL) != 0)
    {
        close(kfd);
        (void)send_error_response(client_fd, env, YAI_E_INTERNAL_ERROR, "kernel_write_failed");
        return -1;
    }

    yai_rpc_envelope_t resp;
    char resp_payload[YAI_MAX_PAYLOAD];
    ssize_t r = yai_control_read_frame(kfd, &resp, resp_payload, sizeof(resp_payload));
    close(kfd);

    if (r < 0)
    {
        (void)send_error_response(client_fd, env, YAI_E_INTERNAL_ERROR, "kernel_read_failed");
        return -1;
    }

    return yai_control_write_frame(client_fd, &resp, resp_payload);
}

int root_dispatch_frame(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload,
    ssize_t payload_len,
    int *handshake_done)
{
    if (!env || !handshake_done)
        return -1;

    if (env->magic != YAI_FRAME_MAGIC)
        return send_error_response(client_fd, env, YAI_E_BAD_MAGIC, "bad_magic");
    if (env->version != YAI_PROTOCOL_IDS_VERSION)
        return send_error_response(client_fd, env, YAI_E_BAD_VERSION, "bad_version");
    if (env->payload_len > YAI_MAX_PAYLOAD)
        return send_error_response(client_fd, env, YAI_E_PAYLOAD_TOO_BIG, "payload_too_big");
    if (env->checksum != 0)
        return send_error_response(client_fd, env, YAI_E_BAD_CHECKSUM, "bad_checksum");
    if (env->arming > 1)
        return send_error_response(client_fd, env, YAI_E_ARMING_REQUIRED, "arming_flag_invalid");
    if (!is_valid_role(env->role))
        return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "role_invalid");
    if (!yai_ws_id_is_valid(env->ws_id))
        return send_error_response(client_fd, env, YAI_E_BAD_WS_ID, "bad_ws_id");

    if (env->command_id == YAI_CMD_HANDSHAKE)
    {
        if ((size_t)payload_len != sizeof(yai_handshake_req_t))
            return send_error_response(client_fd, env, YAI_E_PAYLOAD_TOO_BIG, "bad_handshake_payload_size");

        yai_handshake_ack_t ack;
        memset(&ack, 0, sizeof(ack));
        ack.server_version = YAI_PROTOCOL_IDS_VERSION;
        ack.capabilities_granted = 0;
        ack.session_id = 1;
        ack.status = (uint8_t)YAI_PROTO_STATE_READY;
        ack._pad = 0;

        if (send_response(client_fd, env, YAI_CMD_HANDSHAKE, &ack, (uint32_t)sizeof(ack)) != 0)
            return -1;
        *handshake_done = 1;
        return 0;
    }

    if (!*handshake_done)
        return send_error_response(client_fd, env, YAI_E_NEED_HANDSHAKE, "need_handshake");

    if (env->role != YAI_ROLE_OPERATOR || !env->arming)
    {
        if (env->role != YAI_ROLE_OPERATOR)
            return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "role_required");
        return send_error_response(client_fd, env, YAI_E_ARMING_REQUIRED, "arming_required");
    }

    return forward_to_kernel_and_relay(client_fd, env, payload, (uint32_t)payload_len);
}
