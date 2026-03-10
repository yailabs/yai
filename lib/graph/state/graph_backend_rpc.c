/* SPDX-License-Identifier: Apache-2.0 */

#include "graph_backend.h"

#include <stdio.h>
#include <string.h>

static char g_rpc_endpoint[256];

int yai_mind_graph_backend_select_rpc(const char *endpoint)
{
  if (!endpoint || !endpoint[0]) return YAI_MIND_ERR_INVALID_ARG;
  snprintf(g_rpc_endpoint, sizeof(g_rpc_endpoint), "%s", endpoint);
  /*
   * RPC backend is not available yet in this runtime build.
   * Keep the endpoint for diagnostics, then fall back to in-memory backend
   * so graph flows remain operational instead of returning not implemented.
   */
  return yai_mind_graph_backend_select_inmemory();
}
