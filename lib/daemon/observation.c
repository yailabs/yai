#include <yai/daemon/observation.h>

int yai_daemon_observation_tick(yai_edge_runtime_t *rt)
{
  return rt ? 0 : -1;
}

