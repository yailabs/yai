#include <string.h>

#include <yai/trace/buffer.h>

void yai_trace_buffer_bootstrap(yai_trace_buffer_t *buffer) {
  if (!buffer) {
    return;
  }
  memset(buffer, 0, sizeof(*buffer));
}

int yai_trace_buffer_push(yai_trace_buffer_t *buffer, const yai_trace_event_t *event) {
  if (!buffer || !event || event->handle == 0) {
    return -1;
  }

  buffer->entries[buffer->head] = *event;
  buffer->head = (buffer->head + 1u) % YAI_TRACE_BUFFER_MAX;

  if (buffer->len < YAI_TRACE_BUFFER_MAX) {
    buffer->len++;
  }

  return 0;
}

int yai_trace_buffer_snapshot(const yai_trace_buffer_t *buffer,
                              yai_trace_event_t *entries,
                              size_t cap,
                              size_t *out_len) {
  size_t i;
  size_t start;
  size_t n;

  if (!buffer || !entries || cap == 0) {
    return -1;
  }

  n = buffer->len < cap ? buffer->len : cap;
  start = (buffer->len == YAI_TRACE_BUFFER_MAX) ? buffer->head : 0u;

  for (i = 0; i < n; ++i) {
    entries[i] = buffer->entries[(start + i) % YAI_TRACE_BUFFER_MAX];
  }

  if (out_len) {
    *out_len = n;
  }

  return 0;
}
