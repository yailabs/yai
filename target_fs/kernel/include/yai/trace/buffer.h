#pragma once

#include <stddef.h>

#include <yai/trace/event.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_TRACE_BUFFER_MAX 512u

typedef struct {
  yai_trace_event_t entries[YAI_TRACE_BUFFER_MAX];
  size_t head;
  size_t len;
} yai_trace_buffer_t;

void yai_trace_buffer_bootstrap(yai_trace_buffer_t *buffer);
int yai_trace_buffer_push(yai_trace_buffer_t *buffer, const yai_trace_event_t *event);
int yai_trace_buffer_snapshot(const yai_trace_buffer_t *buffer,
                              yai_trace_event_t *entries,
                              size_t cap,
                              size_t *out_len);

#ifdef __cplusplus
}
#endif
