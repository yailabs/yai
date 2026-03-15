#include <yai/proc/lifecycle.h>

const char *yai_process_state_name(yai_process_state_t value) {
  switch (value) {
    case YAI_PROCESS_STATE_RUNNING: return "running";
    case YAI_PROCESS_STATE_STOPPED: return "stopped";
    case YAI_PROCESS_STATE_EXITED: return "exited";
    case YAI_PROCESS_STATE_CREATED:
    default: return "created";
  }
}
