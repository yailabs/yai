#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <sys/stat.h>

#include <yai/daemon/runtime_paths.h>
#include <yai/support/paths.h>

static int ensure_dir_component(const char *path)
{
  struct stat st;
  if (!path || !path[0]) return -1;
  if (stat(path, &st) == 0) return S_ISDIR(st.st_mode) ? 0 : -1;
  if (mkdir(path, 0755) == 0) return 0;
  if (errno == EEXIST && stat(path, &st) == 0 && S_ISDIR(st.st_mode)) return 0;
  return -1;
}

static int yai_runtime_local_mkdir_recursive(const char *path)
{
  char tmp[1024];
  size_t i = 0;
  if (!path || !path[0] || strlen(path) >= sizeof(tmp)) return -1;
  memset(tmp, 0, sizeof(tmp));
  (void)snprintf(tmp, sizeof(tmp), "%s", path);
  for (i = 1; tmp[i]; ++i)
  {
    if (tmp[i] == '/')
    {
      tmp[i] = '\0';
      if (tmp[0] != '\0' && ensure_dir_component(tmp) != 0) return -1;
      tmp[i] = '/';
    }
  }
  return ensure_dir_component(tmp);
}

int yai_edge_paths_build(const yai_edge_config_t *cfg, yai_edge_paths_t *paths)
{
  if (!cfg || !paths)
  {
    return -1;
  }
  memset(paths, 0, sizeof(*paths));

  if (snprintf(paths->home, sizeof(paths->home), "%s", cfg->home) >= (int)sizeof(paths->home))
  {
    return -1;
  }
  if (yai_path_join(paths->config_dir, sizeof(paths->config_dir), paths->home, "config") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->state_dir, sizeof(paths->state_dir), paths->home, "state") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->log_dir, sizeof(paths->log_dir), paths->home, "log") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->spool_dir, sizeof(paths->spool_dir), paths->home, "spool") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->bindings_dir, sizeof(paths->bindings_dir), paths->home, "bindings") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->identity_dir, sizeof(paths->identity_dir), paths->home, "identity") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->run_dir, sizeof(paths->run_dir), paths->home, "run") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->pid_file, sizeof(paths->pid_file), paths->run_dir, "yai-daemon.pid") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->health_file, sizeof(paths->health_file), paths->state_dir, "health.v1.json") != 0)
  {
    return -1;
  }
  if (yai_path_join(paths->instance_file,
                    sizeof(paths->instance_file),
                    paths->identity_dir,
                    "instance.v1.json") != 0)
  {
    return -1;
  }
  return 0;
}

int yai_edge_paths_ensure(const yai_edge_paths_t *paths)
{
  if (!paths)
  {
    return -1;
  }
  if (yai_runtime_local_mkdir_recursive(paths->home) != 0 ||
      yai_runtime_local_mkdir_recursive(paths->config_dir) != 0 ||
      yai_runtime_local_mkdir_recursive(paths->state_dir) != 0 ||
      yai_runtime_local_mkdir_recursive(paths->log_dir) != 0 ||
      yai_runtime_local_mkdir_recursive(paths->spool_dir) != 0 ||
      yai_runtime_local_mkdir_recursive(paths->bindings_dir) != 0 ||
      yai_runtime_local_mkdir_recursive(paths->identity_dir) != 0 ||
      yai_runtime_local_mkdir_recursive(paths->run_dir) != 0)
  {
    return -1;
  }
  return 0;
}
