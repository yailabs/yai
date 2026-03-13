#include <string.h>
#include <time.h>

#include <yai/proc/process.h>
#include <yai/proc/task.h>

static yai_task_handle_t g_next_task_handle = 1u;

void yai_task_defaults(yai_task_t *task) {
  if (!task) {
    return;
  }
  memset(task, 0, sizeof(*task));
}

int yai_task_create(yai_process_handle_t process_handle,
                    yai_task_class_t task_class,
                    yai_task_t *out_task) {
  int64_t now = (int64_t)time(NULL);

  if (!out_task || process_handle == 0 || task_class == YAI_TASK_NONE) {
    return -1;
  }

  yai_task_defaults(out_task);
  out_task->handle = g_next_task_handle++;
  out_task->process_handle = process_handle;
  out_task->task_class = task_class;
  out_task->state = YAI_TASK_STATE_READY;
  out_task->created_at = now;
  out_task->updated_at = now;
  return 0;
}

int yai_task_set_state(yai_task_t *task, yai_task_state_t state, int64_t updated_at) {
  if (!task || task->handle == 0 || state == YAI_TASK_STATE_NONE) {
    return -1;
  }

  task->state = state;
  task->updated_at = updated_at;
  return 0;
}
