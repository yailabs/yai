#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include <yai/services/dispatch.h>
#include <yai/protocol/message_types.h>
#include <yai/protocol/rpc/codec.h>
#include <yai/protocol/transport/transport.h>

/*
 * Writes a protocol-level error frame with JSON payload.
 */
int yai_rpc_write_error_v1(int fd,
                           const char *ws_id,
                           const char *trace,
                           const char *code,
                           const char *msg,
                           const char *actor) {
  char payload[512];
  int n;
  yai_rpc_envelope_t env;

  if (fd < 0) {
    return -1;
  }

  n = snprintf(payload,
               sizeof(payload),
               "{\"type\":\"error\",\"error\":{\"code\":\"%s\",\"msg\":\"%s\",\"actor\":\"%s\"}}",
               code ? code : "UNKNOWN",
               msg ? msg : "unknown error",
               actor ? actor : "runtime");
  if (n <= 0 || (size_t)n >= sizeof(payload)) {
    return -1;
  }

  memset(&env, 0, sizeof(env));
  env.magic = YAI_FRAME_MAGIC;
  env.version = YAI_PROTOCOL_IDS_VERSION;
  env.command_id = YAI_CMD_PING;
  env.payload_len = (uint32_t)strlen(payload);

  if (ws_id) {
    strncpy(env.ws_id, ws_id, sizeof(env.ws_id) - 1);
  }
  if (trace) {
    strncpy(env.trace_id, trace, sizeof(env.trace_id) - 1);
  }

  if (yai_control_write_frame(fd, &env, payload) != 0) {
    return -1;
  }
  return 0;
}
