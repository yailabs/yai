/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include <yai/data/binding.h>

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

static int mkdir_if_needed(const char *path)
{
  if (!path || !path[0]) return -1;
  if (mkdir(path, 0775) == 0 || errno == EEXIST) return 0;
  return -1;
}

int yai_data_store_binding_attach_scope(const char *data_scope_id,
                                        char *err,
                                        size_t err_cap)
{
  char ws_root[768];
  char data_dir[768];
  char graph_dir[768];
  char knowledge_dir[768];
  char transient_dir[768];
  const char *run_root;

  if (err && err_cap > 0) err[0] = '\0';
  if (!data_scope_id || !data_scope_id[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "data_scope_id_missing");
    return -1;
  }

  run_root = yai_data_store_binding_root();
  if (!run_root) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "store_binding_not_initialized");
    return -1;
  }

  if (snprintf(ws_root, sizeof(ws_root), "%s/%s", run_root, data_scope_id) <= 0 ||
      snprintf(data_dir, sizeof(data_dir), "%s/data", ws_root) <= 0 ||
      snprintf(graph_dir, sizeof(graph_dir), "%s/graph", ws_root) <= 0 ||
      snprintf(knowledge_dir, sizeof(knowledge_dir), "%s/knowledge", ws_root) <= 0 ||
      snprintf(transient_dir, sizeof(transient_dir), "%s/transient", ws_root) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "data_scope_binding_path_format_failed");
    return -1;
  }

  if (mkdir_if_needed(ws_root) != 0 ||
      mkdir_if_needed(data_dir) != 0 ||
      mkdir_if_needed(graph_dir) != 0 ||
      mkdir_if_needed(knowledge_dir) != 0 ||
      mkdir_if_needed(transient_dir) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "data_scope_binding_mkdir_failed");
    return -1;
  }

  return 0;
}
