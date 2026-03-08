#include <stdio.h>
#include <string.h>

#include <yai/api/runtime.h>
#include <yai/api/version.h>
#include <yai/brain/brain.h>
#include <yai/brain/cognition.h>
#include <yai/core/lifecycle.h>
#include <yai/exec/runtime.h>

typedef enum yai_cli_mode
{
  YAI_CLI_MODE_INVALID = -1,
  YAI_CLI_MODE_HELP = 0,
  YAI_CLI_MODE_RUN,
  YAI_CLI_MODE_STATUS,
  YAI_CLI_MODE_BRAIN_CHECK,
  YAI_CLI_MODE_PREFLIGHT,
  YAI_CLI_MODE_EXEC_PROBE
} yai_cli_mode_t;

static void yai_print_help(void)
{
  puts("yai - unified entrypoint");
  printf("version: %s\n", YAI_VERSION_STRING);
  puts("");
  puts("usage:");
  puts("  yai up                start runtime baseline");
  puts("  yai status            runtime diagnostics");
  puts("  yai brain-check       cognition/runtime smoke");
  puts("  yai preflight         core preflight checks");
  puts("  yai exec-probe        execution plane probe");
  puts("  yai help              show this help");
  puts("  yai --help            show this help");
  puts("");
  puts("notes:");
  puts("  - yai is the single operator/runtime binary");
  puts("  - core, exec and brain are internal runtime modules");
  puts("  - legacy standalone runtime binaries are no longer canonical");
}

static yai_cli_mode_t yai_parse_mode(int argc, char **argv)
{
  if (argc <= 1)
  {
    return YAI_CLI_MODE_HELP;
  }

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

  if (strcmp(argv[1], "status") == 0)
  {
    return YAI_CLI_MODE_STATUS;
  }

  if (strcmp(argv[1], "brain-check") == 0)
  {
    return YAI_CLI_MODE_BRAIN_CHECK;
  }

  if (strcmp(argv[1], "preflight") == 0)
  {
    return YAI_CLI_MODE_PREFLIGHT;
  }

  if (strcmp(argv[1], "exec-probe") == 0)
  {
    return YAI_CLI_MODE_EXEC_PROBE;
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

  rc = yai_ensure_runtime_layout("system");
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

static int yai_run_brain_check(void)
{
  yai_mind_config_t cfg = {
      .runtime_name = "yai",
      .enable_mock_provider = 1};
  yai_mind_cognition_response_t out = {0};
  int rc = 0;

  rc = yai_mind_init(&cfg);
  if (rc != YAI_MIND_OK)
  {
    fprintf(stderr, "yai: brain init failed (%d)\n", rc);
    return 1;
  }

  rc = yai_mind_cognition_execute_text("brain check",
                                       "yai-check",
                                       "mock",
                                       &out);
  if (rc != YAI_MIND_OK)
  {
    fprintf(stderr, "yai: brain cognition probe failed (%d)\n", rc);
    (void)yai_mind_shutdown();
    return 1;
  }

  printf("yai: brain check OK role=%s score=%.2f\n",
         yai_mind_agent_role_name(out.selected_role),
         out.score);

  return (yai_mind_shutdown() == YAI_MIND_OK) ? 0 : 1;
}

static int yai_run_status(void)
{
  int preboot = yai_run_preboot_checks();
  int layout = yai_ensure_runtime_layout("system");
  int exec_state = yai_exec_runtime_probe();

  printf("yai: status core.preboot=%d core.layout=%d exec.state=%s(%d)\n",
         preboot,
         layout,
         yai_exec_runtime_state_name((yai_exec_runtime_state_t)exec_state),
         exec_state);

  return (preboot == 0 && layout == 0 && exec_state >= 0) ? 0 : 1;
}

static int yai_run_runtime(void)
{
  yai_mind_config_t cfg = {
      .runtime_name = "yai-runtime",
      .enable_mock_provider = 1};
  int rc = 0;

  puts("yai: starting runtime preflight...");
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

  puts("yai: initializing brain plane...");
  rc = yai_mind_init(&cfg);
  if (rc != YAI_MIND_OK)
  {
    fprintf(stderr, "yai: brain init failed (%d)\n", rc);
    return 1;
  }

  puts("yai: runtime composition ready (core+exec+brain)");

  puts("yai: shutting down brain plane...");
  if (yai_mind_shutdown() != YAI_MIND_OK)
  {
    fprintf(stderr, "yai: brain shutdown failed\n");
    return 1;
  }

  puts("yai: runtime shutdown complete");
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

  case YAI_CLI_MODE_STATUS:
    return yai_run_status();

  case YAI_CLI_MODE_BRAIN_CHECK:
    return yai_run_brain_check();

  case YAI_CLI_MODE_PREFLIGHT:
    return yai_run_preflight();

  case YAI_CLI_MODE_EXEC_PROBE:
    return yai_run_exec_probe();

  case YAI_CLI_MODE_INVALID:
  default:
    fprintf(stderr, "yai: unknown command '%s'\n",
            (argc > 1 && argv[1]) ? argv[1] : "<null>");
    yai_print_help();
    return 2;
  }
}