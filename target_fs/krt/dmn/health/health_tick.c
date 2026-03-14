#include <yai/dmn/health/health_tick.h>

int yai_daemon_observation_tick(yai_edge_runtime_t *rt)
{
  return rt ? 0 : -1;
}

