/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/cognition/memory.h>
#include <yai/graph/internal/state_reset.h>

void yai_graph_state_reset(void)
{
  yai_graph_activation_reset();
  yai_graph_authority_reset();
  yai_graph_episodic_reset();
  yai_graph_semantic_reset();
  yai_graph_vector_reset();
}
