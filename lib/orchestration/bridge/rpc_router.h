#pragma once

#include <yai/protocol/contracts/rpc.h>
#include <yai/orchestration/transport.h>

typedef char *(*yai_rpc_handler_t)(const char *ws_id, const yai_rpc_envelope_t *env, const char *payload);
char *yai_rpc_router_dispatch(const char *ws_id, const yai_rpc_envelope_t *env, const char *payload);
