/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/memory.h>

int yai_graph_backend_use_inmemory(void);
int yai_graph_backend_use_rpc(const char *endpoint);
const char *yai_graph_backend_name(void);
