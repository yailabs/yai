#pragma once

#ifndef YAI_SCHEDULER_RUN_QUEUE_H
#define YAI_SCHEDULER_RUN_QUEUE_H

#include <stdint.h>

typedef struct {
  uint64_t queued_tasks;
  uint64_t runnable_tasks;
  uint64_t sleeping_tasks;
  uint64_t last_update_tick;
} yai_scheduler_run_queue_state_t;

void yai_scheduler_run_queue_init(void);
void yai_scheduler_run_queue_set_counts(uint64_t queued_tasks,
                                        uint64_t runnable_tasks,
                                        uint64_t sleeping_tasks,
                                        uint64_t tick);
const yai_scheduler_run_queue_state_t *yai_scheduler_run_queue_get(void);

#endif /* YAI_SCHEDULER_RUN_QUEUE_H */
