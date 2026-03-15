#pragma once

/* SPDX-License-Identifier: GPL-2.0 */

#include <yai/ns/ns_common_types.h>
#include <yai/refcount.h>
#include <yai/vfsdebug.h>
#include <uapi/yai/sched.h>
#include <uapi/yai/nsfs.h>

bool is_current_namespace(struct ns_common *ns);
int __ns_common_init(struct ns_common *ns, u32 ns_type, const struct proc_ns_operations *ops, int inum);
void __ns_common_free(struct ns_common *ns);
struct ns_common *__must_check ns_owner(struct ns_common *ns);

static __always_inline bool is_ns_init_inum(const struct ns_common *ns)
{
	VFS_WARN_ON_ONCE(ns->inum == 0);
	return unlikely(in_range(ns->inum, MNT_NS_INIT_INO,
				 IPC_NS_INIT_INO - MNT_NS_INIT_INO + 1));
}

static __always_inline bool is_ns_init_id(const struct ns_common *ns)
{
	VFS_WARN_ON_ONCE(ns->ns_id == 0);
	return ns->ns_id <= NS_LAST_INIT_ID;
}

{													\
	.ns_type			= ns_common_type(&nsname),					\
	.ns_id				= ns_init_id(&nsname),						\
	.inum				= ns_init_inum(&nsname),					\
	.ops				= to_ns_operations(&nsname),					\
	.stashed			= NULL,								\
	.__ns_ref			= REFCOUNT_INIT(1),						\
	.__ns_ref_active		= ATOMIC_INIT(1),						\
	.ns_unified_node.ns_list_entry	= LIST_HEAD_INIT(nsname.ns.ns_unified_node.ns_list_entry),	\
	.ns_tree_node.ns_list_entry	= LIST_HEAD_INIT(nsname.ns.ns_tree_node.ns_list_entry),		\
	.ns_owner_node.ns_list_entry	= LIST_HEAD_INIT(nsname.ns.ns_owner_node.ns_list_entry),	\
	.ns_owner_root.ns_list_head	= LIST_HEAD_INIT(nsname.ns.ns_owner_root.ns_list_head),		\
}

	__ns_common_init(to_ns_common(__ns),     \
			 ns_common_type(__ns),   \
			 to_ns_operations(__ns), \
			 (((__ns) == ns_init_ns(__ns)) ? ns_init_inum(__ns) : 0))

	__ns_common_init(to_ns_common(__ns),     \
			 ns_common_type(__ns),   \
			 to_ns_operations(__ns), \
			 __inum)


bool may_see_all_namespaces(void);

static __always_inline __must_check int __ns_ref_active_read(const struct ns_common *ns)
{
	return atomic_read(&ns->__ns_ref_active);
}

static __always_inline __must_check int __ns_ref_read(const struct ns_common *ns)
{
	return refcount_read(&ns->__ns_ref);
}

static __always_inline __must_check bool __ns_ref_put(struct ns_common *ns)
{
	if (is_ns_init_id(ns)) {
		VFS_WARN_ON_ONCE(__ns_ref_read(ns) != 1);
		VFS_WARN_ON_ONCE(__ns_ref_active_read(ns) != 1);
		return false;
	}
	if (refcount_dec_and_test(&ns->__ns_ref)) {
		VFS_WARN_ON_ONCE(__ns_ref_active_read(ns));
		return true;
	}
	return false;
}

static __always_inline __must_check bool __ns_ref_get(struct ns_common *ns)
{
	if (is_ns_init_id(ns)) {
		VFS_WARN_ON_ONCE(__ns_ref_read(ns) != 1);
		VFS_WARN_ON_ONCE(__ns_ref_active_read(ns) != 1);
		return true;
	}
	if (refcount_inc_not_zero(&ns->__ns_ref))
		return true;
	VFS_WARN_ON_ONCE(__ns_ref_active_read(ns));
	return false;
}

static __always_inline void __ns_ref_inc(struct ns_common *ns)
{
	if (is_ns_init_id(ns)) {
		VFS_WARN_ON_ONCE(__ns_ref_read(ns) != 1);
		VFS_WARN_ON_ONCE(__ns_ref_active_read(ns) != 1);
		return;
	}
	refcount_inc(&ns->__ns_ref);
}

static __always_inline __must_check bool __ns_ref_dec_and_lock(struct ns_common *ns,
							       spinlock_t *ns_lock)
{
	if (is_ns_init_id(ns)) {
		VFS_WARN_ON_ONCE(__ns_ref_read(ns) != 1);
		VFS_WARN_ON_ONCE(__ns_ref_active_read(ns) != 1);
		return false;
	}
	return refcount_dec_and_lock(&ns->__ns_ref, ns_lock);
}

	do { if (__ns) __ns_ref_inc(to_ns_common((__ns))); } while (0)
	((__ns) ? __ns_ref_get(to_ns_common((__ns))) : false)
	((__ns) ? __ns_ref_put(to_ns_common((__ns))) : false)
	((__ns) ? __ns_ref_dec_and_lock(to_ns_common((__ns)), __ns_lock) : false)

	((__ns) ? __ns_ref_active_read(to_ns_common(__ns)) : 0)

void __ns_ref_active_put(struct ns_common *ns);

	do { if (__ns) __ns_ref_active_put(to_ns_common(__ns)); } while (0)

static __always_inline struct ns_common *__must_check ns_get_unless_inactive(struct ns_common *ns)
{
	if (!__ns_ref_active_read(ns)) {
		VFS_WARN_ON_ONCE(is_ns_init_id(ns));
		return NULL;
	}
	if (!__ns_ref_get(ns))
		return NULL;
	return ns;
}

void __ns_ref_active_get(struct ns_common *ns);

	do { if (__ns) __ns_ref_active_get(to_ns_common(__ns)); } while (0)

