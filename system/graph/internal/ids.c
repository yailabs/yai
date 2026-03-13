/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/memory.h>

int yai_node_id_is_valid(yai_node_id_t node_id)
{
  return node_id != YAI_MIND_NODE_ID_INVALID;
}

int yai_edge_id_is_valid(yai_edge_id_t edge_id)
{
  return edge_id != YAI_MIND_EDGE_ID_INVALID;
}
