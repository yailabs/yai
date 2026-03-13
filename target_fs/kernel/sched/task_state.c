#include <yai/scheduler/task_state.h>

const char *yai_task_state_name(yai_task_state_t value) {
  switch (value) {
    case YAI_TASK_STATE_RUNNABLE: return "runnable";
    case YAI_TASK_STATE_RUNNING: return "running";
    case YAI_TASK_STATE_BLOCKED: return "blocked";
    case YAI_TASK_STATE_EXITED: return "exited";
    case YAI_TASK_STATE_CREATED:
    default: return "created";
  }
}
