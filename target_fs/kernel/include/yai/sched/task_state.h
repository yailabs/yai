#pragma once

typedef enum {
  YAI_TASK_STATE_CREATED = 0,
  YAI_TASK_STATE_RUNNABLE,
  YAI_TASK_STATE_RUNNING,
  YAI_TASK_STATE_BLOCKED,
  YAI_TASK_STATE_EXITED,
} yai_task_state_t;

const char *yai_task_state_name(yai_task_state_t value);
