/* SPDX-License-Identifier: Apache-2.0 */

#include "reply_map.h"
#include <yai/sdk/errors.h>

#include <string.h>

int yai_reply_map_rc(const char *status, const char *code)
{
    if (!status || !status[0] || !code || !code[0]) {
        return YAI_SDK_PROTOCOL;
    }

    if (strcmp(status, "ok") == 0 && strcmp(code, "OK") == 0) {
        return YAI_SDK_OK;
    }
    if (strcmp(status, "nyi") == 0 && strcmp(code, "NOT_IMPLEMENTED") == 0) {
        return YAI_SDK_NYI;
    }
    if (strcmp(status, "error") != 0) {
        return YAI_SDK_PROTOCOL;
    }
    if (strcmp(code, "BAD_ARGS") == 0) {
        return YAI_SDK_BAD_ARGS;
    }
    if (strcmp(code, "UNAUTHORIZED") == 0) {
        return YAI_SDK_UNAUTHORIZED;
    }
    if (strcmp(code, "RUNTIME_NOT_READY") == 0) {
        return YAI_SDK_RUNTIME_NOT_READY;
    }
    if (strcmp(code, "SERVER_UNAVAILABLE") == 0) {
        return YAI_SDK_SERVER_OFF;
    }
    if (strcmp(code, "PROTOCOL_ERROR") == 0 ||
        strcmp(code, "INTERNAL_ERROR") == 0 ||
        strcmp(code, "INVALID_TARGET") == 0) {
        return YAI_SDK_PROTOCOL;
    }
    return YAI_SDK_PROTOCOL;
}
