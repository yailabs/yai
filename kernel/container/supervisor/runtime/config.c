#include <yai/dmn/config.h>

int yai_daemon_config_validate(const yai_daemon_config_t *cfg)
{
  return yai_edge_config_validate(cfg);
}

