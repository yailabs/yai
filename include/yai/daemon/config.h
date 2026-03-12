#pragma once

#include <yai/runtime/local/config.h>

typedef yai_edge_config_t yai_daemon_config_t;

int yai_daemon_config_validate(const yai_daemon_config_t *cfg);

