#pragma once

typedef enum {
  YAI_PROCESS_STATE_CREATED = 0,
  YAI_PROCESS_STATE_RUNNING,
  YAI_PROCESS_STATE_STOPPED,
  YAI_PROCESS_STATE_EXITED,
} yai_process_state_t;

const char *yai_process_state_name(yai_process_state_t value);
