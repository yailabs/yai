/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/network/topology/topology.h>

int yai_network_topology_ready(yai_network_topology_kind_t kind)
{
  return kind > 0 ? 1 : 0;
}
