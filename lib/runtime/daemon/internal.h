#pragma once

#include <stdarg.h>

int yai_edge_write_file(const char *path, const char *payload);
int yai_edge_mkdir_recursive(const char *path);
void yai_edge_vlog(const char *instance_id,
                     const char *level,
                     const char *fmt,
                     va_list ap);
