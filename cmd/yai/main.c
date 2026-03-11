#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <signal.h>
#include <sys/select.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <unistd.h>

#include <yai/api/runtime.h>
#include <yai/api/version.h>
#include <yai/core/dispatch.h>
#include <yai/core/lifecycle.h>
#include <yai/data/binding.h>
#include <yai/exec/transport_client.h>
#include <yai/exec/runtime.h>

typedef enum yai_cli_mode
{
  YAI_CLI_MODE_INVALID = -1,
  YAI_CLI_MODE_HELP = 0,
  YAI_CLI_MODE_RUN,
  YAI_CLI_MODE_DOWN
} yai_cli_mode_t;

static volatile sig_atomic_t g_runtime_stop = 0;

static void yai_on_signal(int sig)
{
  (void)sig;
  g_runtime_stop = 1;
}

static int yai_runtime_path_from_rel(const char *rel, char *out, size_t out_cap)
{
  const char *home = getenv("HOME");
  if (!rel || !rel[0] || !out || out_cap == 0 || !home || !home[0])
  {
    return -1;
  }
  if (snprintf(out, out_cap, "%s/%s", home, rel) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}

static int yai_runtime_socket_path(char *out, size_t out_cap)
{
  return yai_runtime_ingress_path(out, (uint32_t)out_cap);
}

static int yai_runtime_peer_socket_path(char *out, size_t out_cap)
{
  return yai_runtime_peer_ingress_path(out, (uint32_t)out_cap);
}

static int yai_runtime_pidfile_path(char *out, size_t out_cap)
{
  char socket_path[256] = {0};
  char *slash = NULL;
  const char *override = getenv(YAI_RUNTIME_PIDFILE_ENV);

  if (override && override[0])
  {
    if (snprintf(out, out_cap, "%s", override) >= (int)out_cap)
    {
      return -1;
    }
    return 0;
  }

  if (yai_runtime_socket_path(socket_path, sizeof(socket_path)) == 0)
  {
    slash = strrchr(socket_path, '/');
    if (slash && slash != socket_path)
    {
      *slash = '\0';
      if (snprintf(out, out_cap, "%s/runtime.pid", socket_path) < (int)out_cap)
      {
        return 0;
      }
    }
  }

  return yai_runtime_path_from_rel(YAI_RUNTIME_PIDFILE_REL, out, out_cap);
}

static int yai_runtime_is_reachable(void)
{
  yai_rpc_client_t client = {0};
  int rc = yai_rpc_connect(&client, "system");
  if (rc != 0)
  {
    return 0;
  }
  rc = yai_rpc_handshake(&client, 0);
  yai_rpc_close(&client);
  return rc == 0 ? 1 : 0;
}

static int yai_runtime_write_pidfile(const char *pidfile_path)
{
  FILE *f = NULL;

  if (!pidfile_path || !pidfile_path[0])
  {
    return -1;
  }

  f = fopen(pidfile_path, "w");
  if (!f)
  {
    return -1;
  }

  fprintf(f, "%ld\n", (long)getpid());
  fclose(f);
  return 0;
}

static int yai_runtime_read_pidfile(const char *pidfile_path, pid_t *pid_out)
{
  FILE *f = NULL;
  long pid_raw = 0;

  if (!pidfile_path || !pidfile_path[0] || !pid_out)
  {
    return -1;
  }

  f = fopen(pidfile_path, "r");
  if (!f)
  {
    return -1;
  }

  if (fscanf(f, "%ld", &pid_raw) != 1)
  {
    fclose(f);
    return -1;
  }
  fclose(f);

  if (pid_raw <= 0)
  {
    return -1;
  }

  *pid_out = (pid_t)pid_raw;
  return 0;
}

static void yai_runtime_remove_pidfile(const char *pidfile_path)
{
  if (pidfile_path && pidfile_path[0])
  {
    (void)unlink(pidfile_path);
  }
}

static int yai_install_signal_handlers(void)
{
  struct sigaction sa;
  memset(&sa, 0, sizeof(sa));
  sa.sa_handler = yai_on_signal;
  sigemptyset(&sa.sa_mask);

  if (sigaction(SIGINT, &sa, NULL) != 0)
  {
    return -1;
  }
  if (sigaction(SIGTERM, &sa, NULL) != 0)
  {
    return -1;
  }
  return 0;
}

static int yai_runtime_drain_client(int client_fd, yai_runtime_ingress_kind_t ingress_kind)
{
  int handshake_done = 0;
  while (!g_runtime_stop)
  {
    yai_rpc_envelope_t env;
    char payload[YAI_MAX_PAYLOAD];
    ssize_t payload_len = yai_control_read_frame(client_fd, &env, payload, sizeof(payload));
    if (payload_len < 0)
    {
      break;
    }
    if (payload_len >= 0)
    {
      size_t term = (size_t)payload_len < sizeof(payload) ? (size_t)payload_len : (sizeof(payload) - 1);
      payload[term] = '\0';
    }
    if (yai_dispatch_frame(client_fd, &env, payload, payload_len, &handshake_done, ingress_kind) != 0)
    {
      break;
    }
  }
  return 0;
}

static int yai_runtime_serve_loop(const char *control_socket_path, const char *peer_socket_path)
{
  int control_listener_fd = yai_control_listen_at(control_socket_path);
  int peer_listener_fd = -1;
  if (control_listener_fd < 0)
  {
    fprintf(stderr, "yai: failed to open control ingress socket (%s)\n", control_socket_path);
    fprintf(stderr, "yai: hint: set %s to a writable UDS path (e.g. /tmp/yai-control.sock)\n",
            YAI_RUNTIME_INGRESS_ENV);
    return 1;
  }
  peer_listener_fd = yai_control_listen_at(peer_socket_path);
  if (peer_listener_fd < 0)
  {
    fprintf(stderr, "yai: failed to open peer ingress socket (%s)\n", peer_socket_path);
    fprintf(stderr, "yai: hint: set %s to a writable UDS path (e.g. /tmp/yai-peer.sock)\n",
            YAI_RUNTIME_PEER_INGRESS_ENV);
    close(control_listener_fd);
    (void)unlink(control_socket_path);
    return 1;
  }

  printf("yai: control ingress listening on %s\n", control_socket_path);
  printf("yai: peer ingress listening on %s\n", peer_socket_path);
  puts("yai: service is live; press Ctrl+C to stop");

  while (!g_runtime_stop)
  {
    fd_set rfds;
    int max_fd = control_listener_fd > peer_listener_fd ? control_listener_fd : peer_listener_fd;
    int ready = 0;
    FD_ZERO(&rfds);
    FD_SET(control_listener_fd, &rfds);
    FD_SET(peer_listener_fd, &rfds);
    ready = select(max_fd + 1, &rfds, NULL, NULL, NULL);
    if (ready < 0)
    {
      if (errno == EINTR)
      {
        continue;
      }
      perror("yai: select failed");
      close(control_listener_fd);
      close(peer_listener_fd);
      unlink(control_socket_path);
      unlink(peer_socket_path);
      return 1;
    }

    if (FD_ISSET(control_listener_fd, &rfds))
    {
      int client_fd = accept(control_listener_fd, NULL, NULL);
      if (client_fd < 0)
      {
        if (errno == EINTR)
        {
          continue;
        }
        perror("yai: control accept failed");
      }
      else
      {
        (void)yai_runtime_drain_client(client_fd, YAI_RUNTIME_INGRESS_CONTROL);
        close(client_fd);
      }
    }

    if (FD_ISSET(peer_listener_fd, &rfds))
    {
      int client_fd = accept(peer_listener_fd, NULL, NULL);
      if (client_fd < 0)
      {
        if (errno == EINTR)
        {
          continue;
        }
        perror("yai: peer accept failed");
      }
      else
      {
        (void)yai_runtime_drain_client(client_fd, YAI_RUNTIME_INGRESS_PEER);
        close(client_fd);
      }
    }
  }

  close(control_listener_fd);
  close(peer_listener_fd);
  unlink(control_socket_path);
  unlink(peer_socket_path);
  puts("yai: service stopped");
  return 0;
}

static void yai_print_help(void)
{
  puts("yai - YAI service host (internal)");
  printf("version: %s\n", YAI_VERSION_STRING);
  puts("");
  puts("usage: yai");
  puts("       yai up      (fallback start)");
  puts("       yai down    (fallback stop)");
  puts("       yai --help");
  puts("");
  puts("notes:");
  puts("  - this binary hosts the YAI service process");
  puts("  - operator entrypoint is CLI: `yai up|status|down` from repo cli");
  puts("  - this binary exposes fallback lifecycle only (up/down)");
  puts("  - edge acquisition companion binary is `yai-daemon` (YD-2 skeleton)");
  puts("  - local control ingress: $HOME/.yai/run/control.sock");
  puts("  - peer source-plane ingress: $HOME/.yai/run/peer.sock");
  puts("  - client flow: cli -> sdk -> yai ingress");
  puts("  - core, exec, data, graph and knowledge are internal runtime modules");
}

static yai_cli_mode_t yai_parse_mode(int argc, char **argv)
{
  if (argc <= 1)
    return YAI_CLI_MODE_RUN;

  if (strcmp(argv[1], "--help") == 0 ||
      strcmp(argv[1], "-h") == 0 ||
      strcmp(argv[1], "help") == 0)
  {
    return YAI_CLI_MODE_HELP;
  }

  if (strcmp(argv[1], "up") == 0)
  {
    return YAI_CLI_MODE_RUN;
  }

  if (strcmp(argv[1], "down") == 0)
  {
    return YAI_CLI_MODE_DOWN;
  }

  return YAI_CLI_MODE_INVALID;
}

static int yai_run_preflight(void)
{
  int rc = yai_run_preboot_checks();
  if (rc != 0)
  {
    fprintf(stderr, "yai: preboot checks failed (rc=%d)\n", rc);
    return 1;
  }

  rc = yai_ensure_runtime_layout(NULL);
  if (rc != 0)
  {
    fprintf(stderr, "yai: runtime layout failed (rc=%d)\n", rc);
    return 1;
  }

  puts("yai: preflight OK");
  return 0;
}

static int yai_run_exec_probe(void)
{
  int state = yai_exec_runtime_probe();

  printf("yai: exec runtime state=%s (%d)\n",
         yai_exec_runtime_state_name((yai_exec_runtime_state_t)state),
         state);

  return (state >= 0) ? 0 : 1;
}

static void yai_print_runtime_capability_snapshot(const char *phase)
{
  const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();
  int exec_state = yai_exec_runtime_probe();
  int data_ready = yai_data_store_binding_is_ready() != 0;
  int knowledge_ready = (caps && caps->providers_ready && caps->memory_ready && caps->cognition_ready) ? 1 : 0;
  int graph_ready = (yai_runtime_capabilities_is_ready() && data_ready &&
                     caps && caps->workspace_id[0] && strcmp(caps->workspace_id, "system") != 0) ? 1 : 0;

  printf("yai: [%s] runtime.ready=%s workspace=%s data.ready=%s graph.ready=%s knowledge.ready=%s exec.state=%s\n",
         phase ? phase : "state",
         yai_runtime_capabilities_is_ready() ? "true" : "false",
         (caps && caps->workspace_id[0]) ? caps->workspace_id : "none",
         data_ready ? "true" : "false",
         graph_ready ? "true" : "false",
         knowledge_ready ? "true" : "false",
         yai_exec_runtime_state_name((yai_exec_runtime_state_t)exec_state));
}

static int yai_run_runtime(void)
{
  char socket_path[256] = {0};
  char peer_socket_path[256] = {0};
  char pidfile_path[256] = {0};
  int pidfile_written = 0;
  char lifecycle_err[128] = {0};
  int rc = 0;

  puts("yai: preparing service...");
  rc = yai_run_preflight();
  if (rc != 0)
  {
    return rc;
  }

  puts("yai: probing exec plane...");
  rc = yai_run_exec_probe();
  if (rc != 0)
  {
    return rc;
  }
  yai_print_runtime_capability_snapshot("pre-start");

  if (yai_runtime_socket_path(socket_path, sizeof(socket_path)) != 0 ||
      yai_runtime_peer_socket_path(peer_socket_path, sizeof(peer_socket_path)) != 0 ||
      yai_runtime_pidfile_path(pidfile_path, sizeof(pidfile_path)) != 0)
  {
    fprintf(stderr, "yai: failed to resolve runtime paths\n");
    return 1;
  }

  if (yai_runtime_is_reachable())
  {
    fprintf(stderr, "yai: runtime already active on %s\n", socket_path);
    return 1;
  }

  if (yai_install_signal_handlers() != 0)
  {
    fprintf(stderr, "yai: failed to install signal handlers\n");
    return 1;
  }

  puts("yai: initializing unified runtime capabilities...");
  rc = yai_runtime_capabilities_start("system", "yai-runtime", 1, lifecycle_err, sizeof(lifecycle_err));
  if (rc != 0)
  {
    fprintf(stderr,
            "yai: runtime capability initialization failed (%d)%s%s\n",
            rc,
            lifecycle_err[0] ? " - " : "",
            lifecycle_err);
    return 1;
  }
  yai_print_runtime_capability_snapshot("post-start");

  if (yai_runtime_write_pidfile(pidfile_path) != 0)
  {
    fprintf(stderr, "yai: warning: failed to write runtime pidfile (%s)\n", pidfile_path);
  }
  else
  {
    pidfile_written = 1;
  }

  rc = yai_runtime_serve_loop(socket_path, peer_socket_path);

  if (pidfile_written)
  {
    yai_runtime_remove_pidfile(pidfile_path);
  }

  if (yai_runtime_capabilities_stop(lifecycle_err, sizeof(lifecycle_err)) != 0)
  {
    fprintf(stderr,
            "yai: runtime capability shutdown failed%s%s\n",
            lifecycle_err[0] ? " - " : "",
            lifecycle_err);
    return 1;
  }

  return rc;
}

static int yai_run_down(void)
{
  char pidfile_path[256] = {0};
  char socket_path[256] = {0};
  char peer_socket_path[256] = {0};
  pid_t pid = 0;
  int stopped = 0;

  if (yai_runtime_pidfile_path(pidfile_path, sizeof(pidfile_path)) == 0 &&
      yai_runtime_read_pidfile(pidfile_path, &pid) == 0)
  {
    if (kill(pid, SIGTERM) == 0 || errno == ESRCH)
    {
      stopped = 1;
    }
  }

  if (yai_runtime_socket_path(socket_path, sizeof(socket_path)) == 0)
  {
    (void)unlink(socket_path);
  }
  if (yai_runtime_peer_socket_path(peer_socket_path, sizeof(peer_socket_path)) == 0)
  {
    (void)unlink(peer_socket_path);
  }
  yai_runtime_remove_pidfile(pidfile_path);

  if (stopped)
  {
    puts("yai: fallback stop signal delivered");
    return 0;
  }

  puts("yai: no active runtime pid found; ingress artifacts cleaned");
  return 0;
}

int main(int argc, char **argv)
{
  yai_cli_mode_t mode = yai_parse_mode(argc, argv);

  switch (mode)
  {
  case YAI_CLI_MODE_HELP:
    yai_print_help();
    return 0;

  case YAI_CLI_MODE_RUN:
    return yai_run_runtime();

  case YAI_CLI_MODE_DOWN:
    return yai_run_down();

  case YAI_CLI_MODE_INVALID:
  default:
    fprintf(stderr, "yai: unsupported argument '%s'\n",
            (argc > 1 && argv[1]) ? argv[1] : "<null>");
    fprintf(stderr, "yai: operator commands are in CLI binary (repo cli)\n");
    yai_print_help();
    return 2;
  }
}
