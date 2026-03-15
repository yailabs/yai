/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX__INIT_TASK_H
#define _LINUX__INIT_TASK_H

#include <yai/rcupdate.h>
#include <yai/irqflags.h>
#include <yai/utsname.h>
#include <yai/lockdep.h>
#include <yai/ftrace.h>
#include <yai/ipc.h>
#include <yai/pid_namespace.h>
#include <yai/user_namespace.h>
#include <yai/securebits.h>
#include <yai/seqlock.h>
#include <yai/rbtree.h>
#include <yai/refcount.h>
#include <yai/sched/autogroup.h>
#include <net/net_namespace.h>
#include <yai/sched/rt.h>
#include <yai/livepatch.h>
#include <yai/mm_types.h>

#include <asm/thread_info.h>

extern struct files_struct init_files;
extern struct fs_struct init_fs;
extern struct nsproxy init_nsproxy;

#ifndef CONFIG_VIRT_CPU_ACCOUNTING_NATIVE
#define INIT_PREV_CPUTIME(x)	.prev_cputime = {			\
	.lock = __RAW_SPIN_LOCK_UNLOCKED(x.prev_cputime.lock),		\
},
#else
#define INIT_PREV_CPUTIME(x)
#endif

#define INIT_TASK_COMM "swapper"

/* Attach to the thread_info data structure for proper alignment */
#define __init_thread_info __section(".data..init_thread_info")

#endif
