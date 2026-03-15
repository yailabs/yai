/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/ipc/runtime.h>
#include <yai/ipc/message_types.h>
#include <string.h>
#include <stdio.h>

bool yai_envelope_validate(
    const yai_rpc_envelope_t* env,
    const char* expected_ws_id)
{
    if (!env)
        return false;

    if (env->magic != YAI_FRAME_MAGIC)
        return false;

    if (env->version != YAI_PROTOCOL_IDS_VERSION)
        return false;

    if (!expected_ws_id || expected_ws_id[0] == '\0')
        return true;

    if (strncmp(env->ws_id,
                expected_ws_id,
                sizeof(env->ws_id)) != 0)
        return false;

    return true;
}

void yai_envelope_prepare_response(
    yai_rpc_envelope_t* out,
    const yai_rpc_envelope_t* request,
    uint32_t command_id,
    uint32_t payload_len)
{
    memset(out, 0, sizeof(*out));

    out->magic       = YAI_FRAME_MAGIC;
    out->version     = YAI_PROTOCOL_IDS_VERSION;
    out->command_id  = command_id;
    out->payload_len = payload_len;

    snprintf(out->ws_id,
             sizeof(out->ws_id),
             "%s",
             request->ws_id);

    snprintf(out->trace_id,
             sizeof(out->trace_id),
             "%s",
             request->trace_id);
}
