/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _IF_TUNNEL_H_
#define _IF_TUNNEL_H_

#include <yai/ip.h>
#include <yai/in6.h>
#include <yai/uapi/if_tunnel.h>
#include <yai/u64_stats_sync.h>

/*
 * Locking : hash tables are protected by RCU and RTNL
 */

#define for_each_ip_tunnel_rcu(pos, start) \
	for (pos = rcu_dereference(start); pos; pos = rcu_dereference(pos->next))

#endif /* _IF_TUNNEL_H_ */
