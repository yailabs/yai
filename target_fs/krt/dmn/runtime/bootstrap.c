#include <yai/dmn/bootstrap.h>

int yai_daemon_bootstrap_prepare(yai_edge_runtime_t *rt)
{
  return rt ? 0 : -1;
}

