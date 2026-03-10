/* SPDX-License-Identifier: Apache-2.0 */
#define _POSIX_C_SOURCE 200809L

#include <yai/core/dispatch.h>
#include <yai/core/authority.h>
#include <yai/core/session.h>
#include <yai/core/workspace.h>

#include <errors.h>
#include <protocol.h>
#include <roles.h>
#include <yai_protocol_ids.h>

#include <errno.h>
#include <stdio.h>
#include <string.h>

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

int yai_dispatch_frame(
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

    {
        yai_authority_evaluation_t auth_eval;
        char auth_err[96];
        if (yai_authority_command_gate(NULL,
                                       env->command_id,
                                       env->role,
                                       env->arming,
                                       &auth_eval,
                                       auth_err,
                                       sizeof(auth_err)) != 0) {
            return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "authority_gate_failed");
        }
        if (auth_eval.decision == YAI_AUTHORITY_DENY) {
            if (strcmp(auth_eval.reason, "operator_arming_required") == 0) {
                if (env->role != YAI_ROLE_OPERATOR)
                    return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "role_required");
                return send_error_response(client_fd, env, YAI_E_ARMING_REQUIRED, "arming_required");
            }
            return send_error_response(client_fd, env, YAI_E_ROLE_REQUIRED, "authority_denied");
        }
    }

    if (payload_len < 0)
        return send_error_response(client_fd, env, YAI_E_PAYLOAD_TOO_BIG, "bad_payload");

    yai_session_dispatch(client_fd, env, payload_len > 0 ? payload : "");
    return 0;
}
