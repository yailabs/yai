#define _POSIX_C_SOURCE 200809L

#include "yai_session_internal.h"
#include "control_transport.h"

#include <stdio.h>
#include <string.h>
#include <yai_protocol_ids.h>

void yai_session_send_binary_response(
    int fd,
    const yai_rpc_envelope_t *req,
    uint32_t command_id,
    const char *json_payload)
{
    yai_rpc_envelope_t resp;
    memset(&resp, 0, sizeof(resp));

    resp.magic = YAI_FRAME_MAGIC;
    resp.version = YAI_PROTOCOL_IDS_VERSION;
    resp.command_id = command_id;

    if (req)
    {
        strncpy(resp.ws_id, req->ws_id, sizeof(resp.ws_id) - 1);
        strncpy(resp.trace_id, req->trace_id, sizeof(resp.trace_id) - 1);
    }

    resp.payload_len = (uint32_t)strlen(json_payload);
    yai_control_write_frame(fd, &resp, json_payload);
}

void yai_session_send_exec_reply(
    int fd,
    const yai_rpc_envelope_t *req,
    const char *status,
    const char *code,
    const char *reason,
    const char *command_id,
    const char *target_plane,
    const char *data_json)
{
    char out[2048];
    const char *trace = (req && req->trace_id[0]) ? req->trace_id : "";
    const char *cmd = (command_id && command_id[0]) ? command_id : "yai.kernel.unknown";
    const char *plane = (target_plane && target_plane[0]) ? target_plane : "kernel";
    const char *st = (status && status[0]) ? status : "error";
    const char *cd = (code && code[0]) ? code : "INTERNAL_ERROR";
    const char *rs = (reason && reason[0]) ? reason : "internal_error";
    const char *data = (data_json && data_json[0]) ? data_json : NULL;

    int n;
    if (data)
    {
        n = snprintf(
            out,
            sizeof(out),
            "{\"type\":\"yai.exec.reply.v1\",\"status\":\"%s\",\"code\":\"%s\",\"reason\":\"%s\","
            "\"command_id\":\"%s\",\"target_plane\":\"%s\",\"trace_id\":\"%s\",\"data\":%s}",
            st,
            cd,
            rs,
            cmd,
            plane,
            trace,
            data);
    }
    else
    {
        n = snprintf(
            out,
            sizeof(out),
            "{\"type\":\"yai.exec.reply.v1\",\"status\":\"%s\",\"code\":\"%s\",\"reason\":\"%s\","
            "\"command_id\":\"%s\",\"target_plane\":\"%s\",\"trace_id\":\"%s\"}",
            st,
            cd,
            rs,
            cmd,
            plane,
            trace);
    }

    if (n <= 0 || (size_t)n >= sizeof(out))
    {
        yai_session_send_binary_response(
            fd,
            req,
            req ? req->command_id : YAI_CMD_CONTROL_CALL,
            "{\"type\":\"yai.exec.reply.v1\",\"status\":\"error\",\"code\":\"INTERNAL_ERROR\","
            "\"reason\":\"response_encode_failed\",\"command_id\":\"yai.kernel.unknown\","
            "\"target_plane\":\"kernel\",\"trace_id\":\"\"}");
        return;
    }

    yai_session_send_binary_response(fd, req, req ? req->command_id : YAI_CMD_CONTROL_CALL, out);
}
