#include <stdio.h>
#include <string.h>
#include <time.h>

#include <yai/proc/process.h>

static yai_process_handle_t g_next_process_handle = 1u;

void yai_process_defaults(yai_process_t *process) {
  if (!process) {
    return;
  }
  memset(process, 0, sizeof(*process));
}

int yai_process_create(yai_process_class_t process_class,
                       const char *name,
                       uint64_t owner_handle,
                       yai_process_t *out_process) {
  int64_t now = (int64_t)time(NULL);

  if (!out_process || !name || name[0] == '\0' || process_class == YAI_PROCESS_NONE) {
    return -1;
  }

  yai_process_defaults(out_process);
  out_process->handle = g_next_process_handle++;
  out_process->process_class = process_class;
  out_process->state = YAI_PROCESS_STATE_CREATED;
  out_process->owner_handle = owner_handle;
  out_process->created_at = now;
  out_process->updated_at = now;

  if (snprintf(out_process->name, sizeof(out_process->name), "%s", name) >= (int)sizeof(out_process->name)) {
    return -1;
  }

  return 0;
}

int yai_process_set_state(yai_process_t *process, yai_process_state_t state, int64_t updated_at) {
  if (!process || process->handle == 0 || state == YAI_PROCESS_STATE_NONE) {
    return -1;
  }

  process->state = state;
  process->updated_at = updated_at;
  return 0;
}
