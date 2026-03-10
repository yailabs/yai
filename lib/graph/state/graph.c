/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/memory.h>
#include "graph_state_internal.h"

void yai_mind_graph_state_reset(void)
{
  yai_mind_graph_activation_reset();
  yai_mind_graph_authority_reset();
  yai_mind_graph_episodic_reset();
  yai_mind_graph_semantic_reset();
  yai_mind_graph_vector_reset();
}
