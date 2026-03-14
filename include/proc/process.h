#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t yai_process_handle_t;

typedef enum {
  YAI_PROCESS_NONE = 0,
  YAI_PROCESS_KERNEL,
  YAI_PROCESS_DAEMON,
  YAI_PROCESS_SESSION,
  YAI_PROCESS_CONTAINER,
  YAI_PROCESS_WORKER,
} yai_process_class_t;

typedef enum {
  YAI_PROCESS_STATE_NONE = 0,
  YAI_PROCESS_STATE_CREATED,
  YAI_PROCESS_STATE_READY,
  YAI_PROCESS_STATE_RUNNING,
  YAI_PROCESS_STATE_BLOCKED,
  YAI_PROCESS_STATE_DEGRADED,
  YAI_PROCESS_STATE_TERMINATED,
} yai_process_state_t;

typedef struct {
  yai_process_handle_t handle;
  yai_process_class_t process_class;
  yai_process_state_t state;
  uint64_t owner_handle;
  uint64_t address_space_handle;
  uint64_t flags;
  int64_t created_at;
  int64_t updated_at;
  char name[64];
} yai_process_t;

void yai_process_defaults(yai_process_t *process);
int yai_process_create(yai_process_class_t process_class,
                       const char *name,
                       uint64_t owner_handle,
                       yai_process_t *out_process);
int yai_process_set_state(yai_process_t *process, yai_process_state_t state, int64_t updated_at);

#ifdef __cplusplus
}
#endif
