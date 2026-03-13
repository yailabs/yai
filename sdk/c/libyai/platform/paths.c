// SPDX-License-Identifier: Apache-2.0

#define _POSIX_C_SOURCE 200809L

#include <yai/sdk/paths.h>

#include <ctype.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static int path_exists_exec(const char *p)
{
  return (p && p[0] && access(p, X_OK) == 0);
}

static int path_exists_dir(const char *p)
{
  struct stat st;
  if (!p || !p[0]) return 0;
  if (stat(p, &st) != 0) return 0;
  return S_ISDIR(st.st_mode) ? 1 : 0;
}

static const char *env_non_empty(const char *key)
{
  const char *v = getenv(key);
  return (v && v[0]) ? v : NULL;
}

static int yai_home_dir(char *out, size_t cap)
{
  const char *home = env_non_empty("HOME");
  int n;
  if (!out || cap == 0) return -1;
  if (home) {
    n = snprintf(out, cap, "%s", home);
    return (n > 0 && (size_t)n < cap) ? 0 : -1;
  }
  n = snprintf(out, cap, "/tmp");
  return (n > 0 && (size_t)n < cap) ? 0 : -1;
}

static int current_exe_path(char *out, size_t cap)
{
#if defined(__linux__)
  ssize_t n;
  if (!out || cap == 0) return -1;
  n = readlink("/proc/self/exe", out, cap - 1);
  if (n <= 0 || (size_t)n >= cap) return -1;
  out[n] = '\0';
  return 0;
#else
  (void)out;
  (void)cap;
  return -1;
#endif
}

static int dirname_of(const char *path, char *out, size_t cap)
{
  const char *slash;
  size_t n;
  if (!path || !path[0] || !out || cap == 0) return -1;
  slash = strrchr(path, '/');
  if (!slash) return -1;
  n = (size_t)(slash - path);
  if (n == 0 || n >= cap) return -1;
  memcpy(out, path, n);
  out[n] = '\0';
  return 0;
}

static int derive_root_from_exe(char *out, size_t cap)
{
  char exe[PATH_MAX];
  char dir1[PATH_MAX];
  char dir2[PATH_MAX];
  char dir3[PATH_MAX];

  if (current_exe_path(exe, sizeof(exe)) != 0) return -1;
  if (dirname_of(exe, dir1, sizeof(dir1)) != 0) return -1;

  if (strstr(dir1, "/dist/bin") != NULL || strstr(dir1, "/build/bin") != NULL) {
    if (dirname_of(dir1, dir2, sizeof(dir2)) != 0) return -1;
    if (dirname_of(dir2, dir3, sizeof(dir3)) != 0) return -1;
    return snprintf(out, cap, "%s", dir3) > 0 ? 0 : -1;
  }

  if (strstr(dir1, "/bin") != NULL) {
    if (dirname_of(dir1, dir2, sizeof(dir2)) != 0) return -1;
    return snprintf(out, cap, "%s", dir2) > 0 ? 0 : -1;
  }

  return -1;
}

static int search_path_for_exec(const char *name, char *out, size_t cap)
{
  const char *path = env_non_empty("PATH");
  const char *seg_start;
  const char *seg_end;
  char candidate[PATH_MAX];
  if (!name || !name[0] || !out || cap == 0) return -1;
  if (strchr(name, '/')) {
    if (path_exists_exec(name)) {
      return snprintf(out, cap, "%s", name) > 0 ? 0 : -1;
    }
    return -1;
  }
  if (!path) return -1;
  seg_start = path;
  while (seg_start && *seg_start) {
    size_t seg_len;
    seg_end = strchr(seg_start, ':');
    seg_len = seg_end ? (size_t)(seg_end - seg_start) : strlen(seg_start);
    if (seg_len > 0 && seg_len < sizeof(candidate) - 2) {
      int n;
      n = snprintf(candidate, sizeof(candidate), "%.*s/%s", (int)seg_len, seg_start, name);
      if (n > 0 && (size_t)n < sizeof(candidate) && path_exists_exec(candidate)) {
        return snprintf(out, cap, "%s", candidate) > 0 ? 0 : -1;
      }
    }
    if (!seg_end) break;
    seg_start = seg_end + 1;
  }
  return -1;
}

static int resolve_runtime_binary(const char *env_key, const char *bin_name, char *out, size_t cap)
{
  char install_root[PATH_MAX];
  char candidate[PATH_MAX];
  const char *env_bin = env_non_empty(env_key);

  if (env_bin && path_exists_exec(env_bin)) {
    return snprintf(out, cap, "%s", env_bin) > 0 ? 0 : -1;
  }

  if (yai_path_install_root(install_root, sizeof(install_root)) == 0) {
    if (snprintf(candidate, sizeof(candidate), "%s/dist/bin/%s", install_root, bin_name) > 0 && path_exists_exec(candidate)) {
      return snprintf(out, cap, "%s", candidate) > 0 ? 0 : -1;
    }
    if (snprintf(candidate, sizeof(candidate), "%s/build/bin/%s", install_root, bin_name) > 0 && path_exists_exec(candidate)) {
      return snprintf(out, cap, "%s", candidate) > 0 ? 0 : -1;
    }
    if (snprintf(candidate, sizeof(candidate), "%s/bin/%s", install_root, bin_name) > 0 && path_exists_exec(candidate)) {
      return snprintf(out, cap, "%s", candidate) > 0 ? 0 : -1;
    }
    if (snprintf(candidate, sizeof(candidate), "%s/../yai/dist/bin/%s", install_root, bin_name) > 0 && path_exists_exec(candidate)) {
      return snprintf(out, cap, "%s", candidate) > 0 ? 0 : -1;
    }
    if (snprintf(candidate, sizeof(candidate), "%s/../yai/build/bin/%s", install_root, bin_name) > 0 && path_exists_exec(candidate)) {
      return snprintf(out, cap, "%s", candidate) > 0 ? 0 : -1;
    }
  }

  return search_path_for_exec(bin_name, out, cap);
}

static int is_valid_scope_id(const char *scope_id)
{
  const char *p;
  if (!scope_id || !scope_id[0]) return 0;
  if (strchr(scope_id, '/') || strstr(scope_id, "..") || scope_id[0] == '~') return 0;
  for (p = scope_id; *p; p++) {
    const char c = *p;
    const int ok =
      (c >= 'a' && c <= 'z') ||
      (c >= 'A' && c <= 'Z') ||
      (c >= '0' && c <= '9') ||
      (c == '-') || (c == '_') || (c == '.');
    if (!ok) return 0;
  }
  return 1;
}

int yai_path_runtime_home(char *out, size_t cap)
{
  const char *override = env_non_empty("YAI_RUNTIME_HOME");
  char home[PATH_MAX];
  int n;

  if (!out || cap < 16) return -1;

  if (override) {
    n = snprintf(out, cap, "%s", override);
    return (n > 0 && (size_t)n < cap) ? 0 : -1;
  }

  if (yai_home_dir(home, sizeof(home)) != 0) return -1;
  n = snprintf(out, cap, "%s/.yai/run", home);
  return (n > 0 && (size_t)n < cap) ? 0 : -1;
}

int yai_path_install_root(char *out, size_t cap)
{
  const char *override = env_non_empty("YAI_INSTALL_ROOT");
  int n;
  if (!out || cap < 8) return -1;
  if (override) {
    n = snprintf(out, cap, "%s", override);
    return (n > 0 && (size_t)n < cap) ? 0 : -1;
  }
  return derive_root_from_exe(out, cap);
}

int yai_path_detect_deploy_mode(yai_runtime_deploy_mode_t *out_mode)
{
  char install_root[PATH_MAX];
  char runtime_home[PATH_MAX];
  if (!out_mode) return -1;

  if (env_non_empty("YAI_QUALIFICATION_MODE")) {
    *out_mode = YAI_RUNTIME_DEPLOY_QUALIFICATION;
    return 0;
  }

  if (yai_path_install_root(install_root, sizeof(install_root)) == 0) {
    if (strstr(install_root, "Developer") || strstr(install_root, "DEV_CODE") || strstr(install_root, "/yai")) {
      *out_mode = YAI_RUNTIME_DEPLOY_REPO_DEV;
      return 0;
    }
    *out_mode = YAI_RUNTIME_DEPLOY_LOCAL_INSTALL;
    return 0;
  }

  if (yai_path_runtime_home(runtime_home, sizeof(runtime_home)) == 0 && path_exists_dir(runtime_home)) {
    *out_mode = YAI_RUNTIME_DEPLOY_PACKAGED;
    return 0;
  }

  *out_mode = YAI_RUNTIME_DEPLOY_UNKNOWN;
  return 0;
}

int yai_path_runtime_bin(char *out, size_t cap)
{
  return resolve_runtime_binary("YAI_RUNTIME_BIN", "yai", out, cap);
}

int yai_path_runtime_ingress_sock(char *out, size_t cap)
{
  const char *override = env_non_empty("YAI_RUNTIME_INGRESS");
  if (!override) {
    /* Compatibility alias kept for existing SDK/CLI callers. */
    override = env_non_empty("YAI_RUNTIME_SOCK");
  }
  char runtime_home[PATH_MAX];
  int n;
  if (!out || cap < 32) return -1;
  if (override) {
    n = snprintf(out, cap, "%s", override);
    return (n > 0 && (size_t)n < cap) ? 0 : -1;
  }
  if (yai_path_runtime_home(runtime_home, sizeof(runtime_home)) != 0) return -1;
  n = snprintf(out, cap, "%s/control.sock", runtime_home);
  return (n > 0 && (size_t)n < cap) ? 0 : -1;
}

int yai_path_scope_sock(const char *scope_id, char *out, size_t cap)
{
  char runtime_home[PATH_MAX];
  int n;
  if (!is_valid_scope_id(scope_id)) return -1;
  if (!out || cap < 32) return -1;
  if (yai_path_runtime_home(runtime_home, sizeof(runtime_home)) != 0) return -1;
  n = snprintf(out, cap, "%s/%s/control.sock", runtime_home, scope_id);
  return (n > 0 && (size_t)n < cap) ? 0 : -1;
}

int yai_path_scope_run_dir(const char *scope_id, char *out, size_t cap)
{
  char runtime_home[PATH_MAX];
  int n;
  if (!is_valid_scope_id(scope_id)) return -1;
  if (!out || cap < 32) return -1;
  if (yai_path_runtime_home(runtime_home, sizeof(runtime_home)) != 0) return -1;
  n = snprintf(out, cap, "%s/%s", runtime_home, scope_id);
  return (n > 0 && (size_t)n < cap) ? 0 : -1;
}

int yai_path_scope_db(const char *scope_id, char *out, size_t cap)
{
  char scope_dir[PATH_MAX];
  int n;
  if (yai_path_scope_run_dir(scope_id, scope_dir, sizeof(scope_dir)) != 0) return -1;
  n = snprintf(out, cap, "%s/semantic.sqlite", scope_dir);
  return (n > 0 && (size_t)n < cap) ? 0 : -1;
}
