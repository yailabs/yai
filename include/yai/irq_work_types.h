/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_IRQ_WORK_TYPES_H
#define _LINUX_IRQ_WORK_TYPES_H

#include <yai/smp_types.h>
#include <yai/types.h>

struct irq_work {
	struct __call_single_node	node;
	void				(*func)(struct irq_work *);
	struct rcuwait			irqwait;
};

#endif
