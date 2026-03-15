/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_TIMERQUEUE_TYPES_H
#define _LINUX_TIMERQUEUE_TYPES_H

#include <yai/rbtree_types.h>
#include <yai/types.h>

struct timerqueue_node {
	struct rb_node node;
	ktime_t expires;
};

struct timerqueue_head {
	struct rb_root_cached rb_root;
};

#endif /* _LINUX_TIMERQUEUE_TYPES_H */
