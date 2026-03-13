#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t yai_trace_event_handle_t;

typedef enum {
  YAI_TRACE_DOMAIN_NONE = 0,
  YAI_TRACE_DOMAIN_KERNEL,
  YAI_TRACE_DOMAIN_CONTAINER,
  YAI_TRACE_DOMAIN_SESSION,
  YAI_TRACE_DOMAIN_POLICY,
  YAI_TRACE_DOMAIN_IPC,
  YAI_TRACE_DOMAIN_NET,
  YAI_TRACE_DOMAIN_FS,
  YAI_TRACE_DOMAIN_MM,
} yai_trace_domain_t;

typedef enum {
  YAI_TRACE_LEVEL_NONE = 0,
  YAI_TRACE_LEVEL_DEBUG,
  YAI_TRACE_LEVEL_INFO,
  YAI_TRACE_LEVEL_WARN,
  YAI_TRACE_LEVEL_ERROR,
  YAI_TRACE_LEVEL_AUDIT,
} yai_trace_level_t;

typedef struct {
  yai_trace_event_handle_t handle;
  yai_trace_domain_t domain;
  yai_trace_level_t level;
  uint64_t subject_handle;
  uint64_t code;
  int64_t timestamp;
  char message[160];
} yai_trace_event_t;

void yai_trace_event_defaults(yai_trace_event_t *event);
int yai_trace_event_init(yai_trace_domain_t domain,
                         yai_trace_level_t level,
                         uint64_t subject_handle,
                         uint64_t code,
                         const char *message,
                         yai_trace_event_t *out_event);

#ifdef __cplusplus
}
#endif
