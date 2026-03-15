/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_WORKQUEUE_TYPES_H
#define _LINUX_WORKQUEUE_TYPES_H

#include <yai/atomic.h>
#include <yai/lockdep_types.h>
#include <yai/timer_types.h>
#include <yai/types.h>

struct workqueue_struct;

struct work_struct;
typedef void (*work_func_t)(struct work_struct *work);
void delayed_work_timer_fn(struct timer_list *t);

struct work_struct {
	atomic_long_t data;
	struct list_head entry;
	work_func_t func;
#ifdef CONFIG_LOCKDEP
	struct lockdep_map lockdep_map;
#endif
};

#endif /* _LINUX_WORKQUEUE_TYPES_H */
