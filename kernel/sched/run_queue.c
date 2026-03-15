#include <yai/sched/run_queue.h>

static yai_scheduler_run_queue_state_t g_run_queue;

void yai_scheduler_run_queue_init(void) {
  g_run_queue.queued_tasks = 0;
  g_run_queue.runnable_tasks = 0;
  g_run_queue.sleeping_tasks = 0;
  g_run_queue.last_update_tick = 0;
}

void yai_scheduler_run_queue_set_counts(uint64_t queued_tasks,
                                        uint64_t runnable_tasks,
                                        uint64_t sleeping_tasks,
                                        uint64_t tick) {
  g_run_queue.queued_tasks = queued_tasks;
  g_run_queue.runnable_tasks = runnable_tasks;
  g_run_queue.sleeping_tasks = sleeping_tasks;
  g_run_queue.last_update_tick = tick;
}

const yai_scheduler_run_queue_state_t *yai_scheduler_run_queue_get(void) {
  return &g_run_queue;
}
