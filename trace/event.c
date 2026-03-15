#include <stdio.h>
#include <string.h>
#include <time.h>

#include <yai/trace/event.h>

static yai_trace_event_handle_t g_next_trace_event_handle = 1u;

void yai_trace_event_defaults(yai_trace_event_t *event) {
  if (!event) {
    return;
  }
  memset(event, 0, sizeof(*event));
}

int yai_trace_event_init(yai_trace_domain_t domain,
                         yai_trace_level_t level,
                         uint64_t subject_handle,
                         uint64_t code,
                         const char *message,
                         yai_trace_event_t *out_event) {
  if (!out_event || domain == YAI_TRACE_DOMAIN_NONE || level == YAI_TRACE_LEVEL_NONE) {
    return -1;
  }

  yai_trace_event_defaults(out_event);
  out_event->handle = g_next_trace_event_handle++;
  out_event->domain = domain;
  out_event->level = level;
  out_event->subject_handle = subject_handle;
  out_event->code = code;
  out_event->timestamp = (int64_t)time(NULL);

  if (message && message[0] != '\0') {
    if (snprintf(out_event->message, sizeof(out_event->message), "%s", message) >= (int)sizeof(out_event->message)) {
      return -1;
    }
  }

  return 0;
}
