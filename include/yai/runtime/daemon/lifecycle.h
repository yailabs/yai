#pragma once

#include <yai/runtime/daemon/runtime.h>

int yai_edge_lifecycle_install_signals(void);
int yai_edge_lifecycle_should_stop(void);
int yai_edge_lifecycle_run_foreground(yai_edge_runtime_t *rt);
