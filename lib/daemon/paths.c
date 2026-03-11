#include <stdio.h>
#include <string.h>

#include <yai/daemon/paths.h>
#include <yai/support/paths.h>

#include "internal.h"

int yai_daemon_paths_build(const yai_daemon_config_t *cfg, yai_daemon_paths_t *paths)
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

int yai_daemon_paths_ensure(const yai_daemon_paths_t *paths)
{
  if (!paths)
  {
    return -1;
  }
  if (yai_daemon_mkdir_recursive(paths->home) != 0 ||
      yai_daemon_mkdir_recursive(paths->config_dir) != 0 ||
      yai_daemon_mkdir_recursive(paths->state_dir) != 0 ||
      yai_daemon_mkdir_recursive(paths->log_dir) != 0 ||
      yai_daemon_mkdir_recursive(paths->spool_dir) != 0 ||
      yai_daemon_mkdir_recursive(paths->bindings_dir) != 0 ||
      yai_daemon_mkdir_recursive(paths->identity_dir) != 0 ||
      yai_daemon_mkdir_recursive(paths->run_dir) != 0)
  {
    return -1;
  }
  return 0;
}
