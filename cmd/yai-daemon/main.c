#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <yai/api/version.h>
#include <yai/daemon/config.h>
#include <yai/daemon/lifecycle.h>
#include <yai/daemon/runtime.h>

static void print_help(void)
{
  puts("yai-daemon - subordinate edge runtime daemon (ER-1 baseline)");
  printf("version: %s\n", YAI_VERSION_STRING);
  puts("");
  puts("usage: yai-daemon [options]");
  puts("  --help                    show help");
  puts("  --config <path>           config file path (KEY=VALUE lines)");
  puts("  --owner-ref <ref>         owner runtime reference placeholder");
  puts("  --source-label <label>    source node label override");
  puts("  --log-level <level>       log level (info/debug/warn/error)");
  puts("  --mode <foreground|background>");
  puts("  --tick-ms <n>             main loop tick milliseconds");
  puts("  --max-ticks <n>           exit after n ticks (0 = run until signal)");
  puts("");
  puts("environment:");
  puts("  YAI_DAEMON_HOME, YAI_DAEMON_CONFIG, YAI_DAEMON_OWNER_REF,");
  puts("  YAI_DAEMON_SOURCE_LABEL, YAI_DAEMON_LOG_LEVEL, YAI_DAEMON_MODE,");
  puts("  YAI_DAEMON_BINDINGS_MANIFEST, YAI_DAEMON_TICK_MS, YAI_DAEMON_MAX_TICKS");
  puts("");
  puts("guardrails:");
  puts("  - yai-daemon is subordinate to owner workspace sovereignty");
  puts("  - explicit lifecycle: bootstrap -> config -> identity -> scope -> runtime");
  puts("  - local action mediation/enforcement is delegated and owner-scoped");
  puts("  - local outcomes baseline: observe_only|allow|block|hold|execute|escalate");
  puts("  - no final authority/policy/graph/conflict truth locally");
  puts("  - no federated runtime/multi-owner semantics in v1");
}

static int read_hostname(char *dst, size_t dst_cap)
{
  if (!dst || dst_cap == 0)
  {
    return -1;
  }
  if (gethostname(dst, dst_cap - 1) != 0)
  {
    return -1;
  }
  dst[dst_cap - 1] = '\0';
  return 0;
}

static int parse_args(int argc, char **argv, yai_daemon_config_t *cfg)
{
  int i = 0;

  for (i = 1; i < argc; ++i)
  {
    if (strcmp(argv[i], "--help") == 0)
    {
      print_help();
      return 1;
    }
    if (strcmp(argv[i], "--config") == 0 && i + 1 < argc)
    {
      if (yai_daemon_config_set_string(cfg->config_path, sizeof(cfg->config_path), argv[++i]) != 0)
      {
        return -1;
      }
      continue;
    }
    if (strcmp(argv[i], "--owner-ref") == 0 && i + 1 < argc)
    {
      if (yai_daemon_config_set_string(cfg->owner_ref, sizeof(cfg->owner_ref), argv[++i]) != 0)
      {
        return -1;
      }
      continue;
    }
    if (strcmp(argv[i], "--source-label") == 0 && i + 1 < argc)
    {
      if (yai_daemon_config_set_string(cfg->source_label, sizeof(cfg->source_label), argv[++i]) != 0)
      {
        return -1;
      }
      continue;
    }
    if (strcmp(argv[i], "--log-level") == 0 && i + 1 < argc)
    {
      if (yai_daemon_config_set_string(cfg->log_level, sizeof(cfg->log_level), argv[++i]) != 0)
      {
        return -1;
      }
      continue;
    }
    if (strcmp(argv[i], "--mode") == 0 && i + 1 < argc)
    {
      if (yai_daemon_config_set_string(cfg->mode, sizeof(cfg->mode), argv[++i]) != 0)
      {
        return -1;
      }
      continue;
    }
    if (strcmp(argv[i], "--tick-ms") == 0 && i + 1 < argc)
    {
      if (yai_daemon_config_parse_uint(argv[++i], &cfg->tick_ms) != 0)
      {
        return -1;
      }
      continue;
    }
    if (strcmp(argv[i], "--max-ticks") == 0 && i + 1 < argc)
    {
      if (yai_daemon_config_parse_int(argv[++i], &cfg->max_ticks) != 0)
      {
        return -1;
      }
      continue;
    }

    fprintf(stderr, "yai-daemon: unknown/invalid argument: %s\n", argv[i]);
    return -1;
  }

  return 0;
}

int main(int argc, char **argv)
{
  yai_daemon_config_t cfg;
  yai_daemon_runtime_t rt;
  int rc = 0;

  if (yai_daemon_config_defaults(&cfg) != 0)
  {
    fprintf(stderr, "yai-daemon: failed to initialize defaults\n");
    return 70;
  }
  (void)yai_daemon_config_apply_env(&cfg);

  rc = parse_args(argc, argv, &cfg);
  if (rc == 1)
  {
    return 0;
  }
  if (rc != 0)
  {
    return 64;
  }

  (void)yai_daemon_config_apply_file(&cfg, cfg.config_path);
  /* Re-apply CLI args so explicit flags override file values deterministically. */
  rc = parse_args(argc, argv, &cfg);
  if (rc != 0)
  {
    return rc == 1 ? 0 : 64;
  }

  if (!cfg.source_label[0])
  {
    if (read_hostname(cfg.source_label, sizeof(cfg.source_label)) != 0)
    {
      (void)yai_daemon_config_set_string(cfg.source_label, sizeof(cfg.source_label), "source-node");
    }
  }

  rc = yai_daemon_config_validate(&cfg);
  if (rc != 0)
  {
    fprintf(stderr, "yai-daemon: config validation failed rc=%d\n", rc);
    return 78;
  }

  if (strcmp(cfg.mode, "background") == 0)
  {
    fprintf(stderr,
            "yai-daemon: background mode placeholder only; use --mode foreground in YD-2\n");
    return 78;
  }

  rc = yai_daemon_runtime_init(&rt, &cfg);
  if (rc != 0)
  {
    fprintf(stderr, "yai-daemon: runtime init failed rc=%d\n", rc);
    return 70;
  }
  if (yai_daemon_lifecycle_install_signals() != 0)
  {
    fprintf(stderr, "yai-daemon: failed to install signal handlers\n");
    return 70;
  }
  if (yai_daemon_runtime_start(&rt) != 0)
  {
    fprintf(stderr, "yai-daemon: runtime start failed\n");
    return 70;
  }

  rc = yai_daemon_lifecycle_run_foreground(&rt);
  if (rc != 0)
  {
    fprintf(stderr, "yai-daemon: foreground loop failed rc=%d\n", rc);
  }

  if (yai_daemon_runtime_shutdown(&rt) != 0)
  {
    fprintf(stderr, "yai-daemon: shutdown failed\n");
    return 70;
  }

  return rc == 0 ? 0 : 1;
}
