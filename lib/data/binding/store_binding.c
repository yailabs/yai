/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include <yai/data/binding.h>

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static int g_store_binding_ready = 0;
static char g_store_root[512];

static int mkdir_if_needed(const char *path)
{
  struct stat st;
  if (!path || !path[0]) return -1;
  if (mkdir(path, 0775) == 0) return 0;
  if (errno == EEXIST) {
    if (stat(path, &st) == 0 && S_ISDIR(st.st_mode)) return 0;
    return -1;
  }
  return -1;
}

int yai_data_store_binding_init(char *err, size_t err_cap)
{
  const char *home = getenv("HOME");
  if (err && err_cap > 0) err[0] = '\0';

  if (!home || !home[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "missing_home");
    return -1;
  }

  if (snprintf(g_store_root, sizeof(g_store_root), "%s/.yai/run/data", home) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "store_root_format_failed");
    return -1;
  }

  {
    char run_root[sizeof(g_store_root)];
    if (snprintf(run_root, sizeof(run_root), "%s/.yai/run", home) <= 0) {
      if (err && err_cap > 0) snprintf(err, err_cap, "%s", "run_root_format_failed");
      return -1;
    }
    if (mkdir_if_needed(run_root) != 0) {
      if (err && err_cap > 0) snprintf(err, err_cap, "%s", "run_root_mkdir_failed");
      return -1;
    }
  }

  if (mkdir_if_needed(g_store_root) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "store_root_mkdir_failed");
    return -1;
  }

  g_store_binding_ready = 1;
  return 0;
}

int yai_data_store_binding_is_ready(void)
{
  return g_store_binding_ready;
}

const char *yai_data_store_binding_root(void)
{
  return g_store_binding_ready ? g_store_root : NULL;
}
