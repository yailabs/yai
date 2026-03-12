/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#include <yai/cognition/errors.h>
#include <yai/cognition/types.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_protocol_kind {
  YAI_MIND_PROTOCOL_UNKNOWN = 0,
  YAI_MIND_PROTOCOL_PING,
  YAI_MIND_PROTOCOL_COMPLETE,
  YAI_MIND_PROTOCOL_EMBED,
  YAI_MIND_PROTOCOL_QUERY,
  YAI_MIND_PROTOCOL_COGNITION
} yai_protocol_kind_t;

typedef struct yai_protocol_request {
  yai_protocol_kind_t kind;
  char provider[32];
  char payload[512];
} yai_protocol_request_t;

typedef struct yai_protocol_response {
  int status;
  int code;
  char payload[512];
} yai_protocol_response_t;

int yai_transport_init(void);
int yai_transport_shutdown(void);
int yai_transport_is_initialized(void);
/* YD-1 transport lock:
 * transport is exec-owned mediation plumbing and does not carry owner truth semantics.
 */
int yai_exec_transport_start(void);
int yai_exec_transport_stop(void);
int yai_exec_transport_is_ready(void);

int yai_protocol_parse(const char *raw,
                            yai_protocol_request_t *request_out);
int yai_protocol_dispatch(const yai_protocol_request_t *request,
                               yai_protocol_response_t *response_out);
int yai_protocol_format_response(const yai_protocol_response_t *response,
                                      char *out,
                                      size_t out_cap);

int yai_transport_handle_raw(const char *raw,
                                  char *response_out,
                                  size_t response_cap);

const char *yai_uds_default_path(void);
int yai_uds_server_run_once(const char *socket_path);

#ifdef __cplusplus
}
#endif
