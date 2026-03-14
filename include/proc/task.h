#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t yai_task_handle_t;

typedef enum {
  YAI_TASK_NONE = 0,
  YAI_TASK_MAIN,
  YAI_TASK_WORKER,
  YAI_TASK_IO,
  YAI_TASK_BACKGROUND,
  YAI_TASK_WATCHER,
} yai_task_class_t;

typedef enum {
  YAI_TASK_STATE_NONE = 0,
  YAI_TASK_STATE_READY,
  YAI_TASK_STATE_RUNNING,
  YAI_TASK_STATE_WAITING,
  YAI_TASK_STATE_STOPPED,
} yai_task_state_t;

typedef struct {
  yai_task_handle_t handle;
  yai_process_handle_t process_handle;
  yai_task_class_t task_class;
  yai_task_state_t state;
  uint64_t flags;
  int64_t created_at;
  int64_t updated_at;
} yai_task_t;

void yai_task_defaults(yai_task_t *task);
int yai_task_create(yai_process_handle_t process_handle,
                    yai_task_class_t task_class,
                    yai_task_t *out_task);
int yai_task_set_state(yai_task_t *task, yai_task_state_t state, int64_t updated_at);

#ifdef __cplusplus
}
#endif
