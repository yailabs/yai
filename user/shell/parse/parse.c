/* SPDX-License-Identifier: Apache-2.0 */
// src/parse/parse.c

#include "yai/shell/parse.h"
#include "yai/sdk/catalog.h"

#include <string.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdio.h>

static void req_zero(yai_porcelain_request_t* r) {
  memset(r, 0, sizeof(*r));
  r->kind = YAI_PORCELAIN_KIND_NONE;
  r->ws_id = NULL;
  r->role = "operator";
  r->arming = 1;
  r->color_mode = YAI_COLOR_AUTO;
  r->watch_interval_ms = 2000;
  r->watch_count = 0;
  r->watch_no_clear = 0;
  r->help_exit_code = 0;
}

static int set_err(yai_porcelain_request_t* r, const char* msg, const char* hint) {
  r->kind = YAI_PORCELAIN_KIND_ERROR;
  r->error = msg ? msg : "unknown error";
  r->error_hint = hint;
  return 1;
}

static int is_help_token(const char* s) {
  return s && (strcmp(s, "help") == 0 || strcmp(s, "--help") == 0 || strcmp(s, "-h") == 0);
}

static int is_version_token(const char* s) {
  return s && (strcmp(s, "--version") == 0 || strcmp(s, "-V") == 0 || strcmp(s, "version") == 0);
}

/* Global options are accepted before the command and are forwarded to the command argv.
   This parser only locates the command boundary deterministically. */
static int is_global_option(const char* s) {
  if (!s || s[0] != '-') return 0;
  return (
    strcmp(s, "--verbose") == 0 ||
    strcmp(s, "-v") == 0 ||
    strcmp(s, "--no-color") == 0 ||
    strcmp(s, "--color") == 0 ||
    strncmp(s, "--color=", 8) == 0 ||
    strcmp(s, "--pager") == 0 ||
    strcmp(s, "--no-pager") == 0 ||
    strcmp(s, "--interactive") == 0 ||
    strcmp(s, "--ws") == 0 ||
    strcmp(s, "--json") == 0 ||
    strcmp(s, "--format") == 0 ||
    strncmp(s, "--format=", 9) == 0 ||
    strcmp(s, "--quiet") == 0 ||
    strcmp(s, "--trace") == 0 ||
    strcmp(s, "--verbose-contract") == 0 ||
    strcmp(s, "--timeout-ms") == 0 ||
    strcmp(s, "--arming") == 0 ||
    strcmp(s, "--role") == 0 ||
    strcmp(s, "--interval") == 0 ||
    strcmp(s, "--interval-ms") == 0 ||
    strcmp(s, "--count") == 0 ||
    strcmp(s, "--once") == 0 ||
    strcmp(s, "--no-clear") == 0 ||
    strcmp(s, "-h") == 0 || strcmp(s, "--help") == 0 ||
    strcmp(s, "--version") == 0 ||
    strcmp(s, "-V") == 0
  );
}

static int global_option_takes_value(const char* s) {
  if (!s) return 0;
  return (
    strcmp(s, "--ws") == 0 ||
    strcmp(s, "--timeout-ms") == 0 ||
    strcmp(s, "--role") == 0 ||
    strcmp(s, "--arming") == 0 ||
    strcmp(s, "--format") == 0 ||
    strcmp(s, "--color") == 0 ||
    strcmp(s, "--interval") == 0 ||
    strcmp(s, "--interval-ms") == 0 ||
    strcmp(s, "--count") == 0
  );
}

static int has_verbose_contract_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (argv && argv[i] && strcmp(argv[i], "--verbose-contract") == 0) {
      return 1;
    }
  }
  return 0;
}

static int has_json_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    const char *v = NULL;
    if (argv && argv[i] && strcmp(argv[i], "--json") == 0) {
      return 1;
    }
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--format") == 0 && i + 1 < argc && argv[i + 1]) {
      v = argv[i + 1];
    } else if (strncmp(argv[i], "--format=", 9) == 0) {
      v = argv[i] + 9;
    }
    if (v && strcmp(v, "json") == 0) return 1;
  }
  return 0;
}

static int has_verbose_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--verbose") == 0 || strcmp(argv[i], "-v") == 0) {
      return 1;
    }
  }
  return 0;
}

static int has_no_color_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--no-color") == 0) {
      return 1;
    }
  }
  return 0;
}

static int has_quiet_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--quiet") == 0) return 1;
  }
  return 0;
}

static int has_trace_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--trace") == 0) return 1;
  }
  return 0;
}

static yai_color_mode_t find_color_mode(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    const char *v = NULL;
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--color") == 0 && i + 1 < argc && argv[i + 1]) {
      v = argv[i + 1];
    } else if (strncmp(argv[i], "--color=", 8) == 0) {
      v = argv[i] + 8;
    }
    if (!v) continue;
    if (strcmp(v, "always") == 0) return YAI_COLOR_ALWAYS;
    if (strcmp(v, "never") == 0) return YAI_COLOR_NEVER;
    return YAI_COLOR_AUTO;
  }
  return YAI_COLOR_AUTO;
}

static int has_pager_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--pager") == 0) return 1;
  }
  return 0;
}

static int has_no_pager_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--no-pager") == 0) return 1;
  }
  return 0;
}

static int has_interactive_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--interactive") == 0) return 1;
  }
  return 0;
}

static const char* find_global_value(int argc, char **argv, const char *flag)
{
  for (int i = 1; i + 1 < argc; i++) {
    if (argv && argv[i] && strcmp(argv[i], flag) == 0) {
      return argv[i + 1];
    }
  }
  return NULL;
}

static int find_global_arming(int argc, char **argv, int fallback)
{
  const char *v = find_global_value(argc, argv, "--arming");
  if (!v || !v[0]) return fallback;
  if (strcmp(v, "0") == 0 || strcmp(v, "false") == 0 || strcmp(v, "off") == 0) return 0;
  return 1;
}

static int find_watch_int_arg(int argc, char **argv, const char *flag, int fallback)
{
  for (int i = 1; i + 1 < argc; i++) {
    if (!argv || !argv[i] || !argv[i + 1]) continue;
    if (strcmp(argv[i], flag) == 0) {
      char *end = NULL;
      long v = strtol(argv[i + 1], &end, 10);
      if (end == argv[i + 1] || (end && *end != '\0')) return fallback;
      if (v < 0) return fallback;
      if (v > 2147483647L) v = 2147483647L;
      return (int)v;
    }
  }
  return fallback;
}

static int has_watch_no_clear_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--no-clear") == 0) return 1;
  }
  return 0;
}

static int has_watch_once_flag(int argc, char **argv)
{
  for (int i = 1; i < argc; i++) {
    if (!argv || !argv[i]) continue;
    if (strcmp(argv[i], "--once") == 0) return 1;
  }
  return 0;
}

static int find_watch_interval_ms(int argc, char **argv, int fallback_ms)
{
  int ms = find_watch_int_arg(argc, argv, "--interval-ms", -1);
  if (ms >= 0) return ms;
  for (int i = 1; i + 1 < argc; i++) {
    char *end = NULL;
    double sec;
    if (!argv || !argv[i] || !argv[i + 1]) continue;
    if (strcmp(argv[i], "--interval") != 0) continue;
    sec = strtod(argv[i + 1], &end);
    if (end == argv[i + 1] || (end && *end != '\0') || sec <= 0.0) return fallback_ms;
    if (sec > 3600.0) sec = 3600.0;
    return (int)(sec * 1000.0);
  }
  return fallback_ms;
}

/* Group abbreviations (UX layer). */
static const char* expand_group_alias(const char* g) {
  if (!g) return NULL;

  /* Canonical operator aliases */
  if (strcmp(g, "ws") == 0) return "container";
  if (strcmp(g, "gov") == 0) return "governance";
  if (strcmp(g, "ins") == 0) return "inspect";
  if (strcmp(g, "bun") == 0) return "bundle";
  if (strcmp(g, "ctl") == 0) return "control";

  /* Transitional aliases kept for runtime family migration */
  if (strcmp(g, "runtime") == 0) return "core";
  if (strcmp(g, "sub") == 0) return "substrate";
  if (strcmp(g, "lif") == 0) return "lifecycle";
  if (strcmp(g, "mem") == 0) return "memory";
  if (strcmp(g, "orc") == 0) return "orch";

  return g;
}

static const yai_sdk_command_ref_t*
find_by_group_name(const yai_sdk_command_catalog_t* cat, const char* group, const char* name) {
  return yai_sdk_command_catalog_find_command(cat, group, name);
}

/* Local operator capability routes are deterministic and independent from runtime command handlers. */
static const char* map_operator_capability_command_id(const char* entrypoint, const char* topic)
{
  const char *normalized_topic = topic;
  if (!entrypoint || !topic || !topic[0]) return NULL;
  if (strcmp(topic, "service") == 0) normalized_topic = "runtime";

  if (strcmp(entrypoint, "doctor") == 0) {
    if (strcmp(normalized_topic, "env") == 0) return "yai.operator.doctor.env";
    if (strcmp(normalized_topic, "runtime") == 0) return "yai.operator.doctor.service";
    if (strcmp(normalized_topic, "container") == 0) return "yai.operator.doctor.container";
    if (strcmp(normalized_topic, "pins") == 0) return "yai.operator.doctor.pins";
    if (strcmp(normalized_topic, "config") == 0) return "yai.operator.doctor.config";
    if (strcmp(normalized_topic, "all") == 0) return "yai.operator.doctor.all";
    return NULL;
  }

  if (strcmp(entrypoint, "inspect") == 0) {
    if (strcmp(normalized_topic, "container") == 0) return "yai.operator.inspect.container";
    if (strcmp(normalized_topic, "runtime") == 0) return "yai.operator.inspect.service";
    if (strcmp(normalized_topic, "catalog") == 0) return "yai.operator.inspect.catalog";
    if (strcmp(normalized_topic, "context") == 0) return "yai.operator.inspect.context";
    if (strcmp(normalized_topic, "source") == 0) return "yai.source.inspect";
    if (strcmp(normalized_topic, "edge") == 0) return "yai.source.status";
    if (strcmp(normalized_topic, "mesh") == 0) return "yai.container.graph.container";
    if (strcmp(normalized_topic, "grant") == 0) return "yai.container.policy_effective";
    if (strcmp(normalized_topic, "transport") == 0) return "yai.operator.inspect.service";
    if (strcmp(normalized_topic, "ingress") == 0) return "yai.operator.inspect.service";
    if (strcmp(normalized_topic, "case") == 0) return "yai.container.graph.summary";
    return NULL;
  }

  if (strcmp(entrypoint, "verify") == 0) {
    if (strcmp(normalized_topic, "law") == 0) return "yai.operator.verify.law";
    if (strcmp(normalized_topic, "registry") == 0) return "yai.operator.verify.registry";
    if (strcmp(normalized_topic, "runtime") == 0) return "yai.operator.verify.service";
    if (strcmp(normalized_topic, "container") == 0) return "yai.operator.verify.container";
    if (strcmp(normalized_topic, "reply") == 0) return "yai.operator.verify.reply";
    if (strcmp(normalized_topic, "alignment") == 0) return "yai.operator.verify.alignment";
    return NULL;
  }

  return NULL;
}

/* Resolve "<group>-<name>" (alias) into a command. */
static const yai_sdk_command_ref_t*
find_by_alias(const yai_sdk_command_catalog_t* cat, const char* token, const char** out_group, const char** out_name) {
  if (out_group) *out_group = NULL;
  if (out_name)  *out_name  = NULL;
  if (!cat || !token || !token[0]) return NULL;

  /* must be non-flag and contain a '-' */
  if (token[0] == '-') return NULL;
  const char* dash = strchr(token, '-');
  if (!dash) return NULL;
  if (dash == token) return NULL;
  if (!dash[1]) return NULL;

  /* split without allocating: compare by temporary buffers */
  char gbuf[128];
  char nbuf[128];

  size_t glen = (size_t)(dash - token);
  size_t nlen = strlen(dash + 1);

  if (glen >= sizeof(gbuf) || nlen >= sizeof(nbuf)) return NULL;

  memcpy(gbuf, token, glen);
  gbuf[glen] = '\0';
  memcpy(nbuf, dash + 1, nlen);
  nbuf[nlen] = '\0';

  const char* g = expand_group_alias(gbuf);
  const char* n = nbuf;

  const yai_sdk_command_ref_t* c = find_by_group_name(cat, g, n);
  if (c) {
    if (out_group) *out_group = c->group;
    if (out_name)  *out_name  = c->name;
  }
  return c;
}

/* Compute the index where the command token starts (after global options). */
static int find_command_start(int argc, char** argv) {
  int i = 1;
  while (i < argc && argv && argv[i]) {
    const char* t = argv[i];

    /* stop at first non-option */
    if (!is_global_option(t)) break;

    /* help/version handled outside; do not treat as value-taking globals */
    if (strcmp(t, "--help") == 0 || strcmp(t, "-h") == 0 || strcmp(t, "--version") == 0 || strcmp(t, "-V") == 0) {
      break; /* command token starts here */
    }

    if (global_option_takes_value(t)) {
      if (i + 1 >= argc) return -1; /* missing value */
      i += 2;
      continue;
    }

    i += 1;
  }
  return i;
}

int yai_porcelain_parse_argv(int argc, char** argv, yai_porcelain_request_t* req) {
  if (!req) return 1;
  req_zero(req);

  if (argc <= 0 || !argv || !argv[0]) {
    return set_err(req, "invalid argv", "Run: yai help");
  }

  /* `yai` -> show global help */
  if (argc == 1) {
    req->kind = YAI_PORCELAIN_KIND_HELP;
    req->help_token = NULL;
    return 0;
  }


  /* Early global flags: support `yai --help/-h` and `yai --version/-V` even before option scanning. */
  if (is_help_token(argv[1])) {
    req->kind = YAI_PORCELAIN_KIND_HELP;
    req->help_token = (2 < argc) ? argv[2] : NULL;
    req->help_token2 = (3 < argc) ? argv[3] : NULL;
    req->help_token3 = (4 < argc) ? argv[4] : NULL;
    return 0;
  }

  if (is_version_token(argv[1])) {
    req->kind = YAI_PORCELAIN_KIND_HELP;
    req->help_token = "version";
    return 0;
  }

  /* Determine where the command starts, allowing global options before it. */
  int cmdi = find_command_start(argc, argv);
  req->verbose_contract = has_verbose_contract_flag(argc, argv);
  req->verbose = has_verbose_flag(argc, argv);
  req->json_output = has_json_flag(argc, argv);
  req->quiet = has_quiet_flag(argc, argv);
  req->show_trace = has_trace_flag(argc, argv);
  req->no_color = has_no_color_flag(argc, argv);
  req->color_mode = find_color_mode(argc, argv);
  req->pager = has_pager_flag(argc, argv);
  req->no_pager = has_no_pager_flag(argc, argv);
  req->interactive = has_interactive_flag(argc, argv);
  req->ws_id = find_global_value(argc, argv, "--ws");
  req->role = find_global_value(argc, argv, "--role");
  if (!req->role || !req->role[0]) req->role = "operator";
  req->arming = find_global_arming(argc, argv, 1);
  if (cmdi < 0 || cmdi >= argc || !argv[cmdi]) {
    return set_err(req, "invalid global options (missing value)", "Run: yai help");
  }

  /* `yai --help` / `yai -h` (even with global options before) */
  if (is_help_token(argv[cmdi])) {
    req->kind = YAI_PORCELAIN_KIND_HELP;
    req->help_token = (cmdi + 1 < argc) ? argv[cmdi + 1] : NULL;
    req->help_token2 = (cmdi + 2 < argc) ? argv[cmdi + 2] : NULL;
    req->help_token3 = (cmdi + 3 < argc) ? argv[cmdi + 3] : NULL;
    return 0;
  }

  /* `yai --version` / `yai version` (even with global options before) */
  if (is_version_token(argv[cmdi])) {
    req->kind = YAI_PORCELAIN_KIND_HELP;
    req->help_token = "version";
    return 0;
  }

  /* `yai law ...` */
  if (strcmp(argv[cmdi], "law") == 0) {
    req->kind = YAI_PORCELAIN_KIND_LAW;
    req->law_argc = (argc > cmdi + 1) ? (argc - (cmdi + 1)) : 0;
    req->law_argv = (argc > cmdi + 1) ? &argv[cmdi + 1] : NULL;
    return 0;
  }

  /* `yai help ...` */
  if (strcmp(argv[cmdi], "help") == 0) {
    req->kind = YAI_PORCELAIN_KIND_HELP;
    req->help_token = (cmdi + 1 < argc) ? argv[cmdi + 1] : NULL;
    req->help_token2 = (cmdi + 2 < argc) ? argv[cmdi + 2] : NULL;
    req->help_token3 = (cmdi + 3 < argc) ? argv[cmdi + 3] : NULL;
    return 0;
  }

  /* Built-in lifecycle path (registry independent). */
  if (strcmp(argv[cmdi], "up") == 0) {
    req->kind = YAI_PORCELAIN_KIND_COMMAND;
    req->command_id = "yai.lifecycle.up";
    req->cmd_argc = argc - (cmdi + 1);
    req->cmd_argv = &argv[cmdi + 1];
    return 0;
  }

  if (strcmp(argv[cmdi], "down") == 0) {
    req->kind = YAI_PORCELAIN_KIND_COMMAND;
    req->command_id = "yai.lifecycle.down";
    req->cmd_argc = argc - (cmdi + 1);
    req->cmd_argv = &argv[cmdi + 1];
    return 0;
  }

  if (strcmp(argv[cmdi], "restart") == 0) {
    req->kind = YAI_PORCELAIN_KIND_COMMAND;
    req->command_id = "yai.lifecycle.restart";
    req->cmd_argc = argc - (cmdi + 1);
    req->cmd_argv = &argv[cmdi + 1];
    return 0;
  }

  if (strcmp(argv[cmdi], "status") == 0) {
    req->kind = YAI_PORCELAIN_KIND_COMMAND;
    req->command_id = "yai.operator.inspect.service";
    req->cmd_argc = argc - (cmdi + 1);
    req->cmd_argv = &argv[cmdi + 1];
    return 0;
  }

  if (strcmp(argv[cmdi], "lifecycle") == 0) {
    if (cmdi + 1 >= argc || !argv[cmdi + 1]) {
      return set_err(req, "missing lifecycle action", "Run: yai help run");
    }
    const char* action = argv[cmdi + 1];
    if (strcmp(action, "up") == 0) req->command_id = "yai.lifecycle.up";
    else if (strcmp(action, "down") == 0) req->command_id = "yai.lifecycle.down";
    else if (strcmp(action, "restart") == 0) req->command_id = "yai.lifecycle.restart";
    else return set_err(req, "unknown lifecycle action", "Run: yai help run");
    req->kind = YAI_PORCELAIN_KIND_COMMAND;
    req->cmd_argc = argc - (cmdi + 2);
    req->cmd_argv = &argv[cmdi + 2];
    return 0;
  }

  /* Canonical container command topology:
     - yai ws create|open|set|switch|current|status|inspect|unset|clear|reset|destroy
     - yai ws graph <summary|container|governance|decision|evidence|authority|artifact|lineage|recent>
     - yai ws db <status|bindings|stores|classes|count|tail>
     - yai ws data <events|evidence|governance|authority|artifacts|enforcement>
     - yai ws cognition <status|transient|memory|providers|context>
     - yai ws policy <effective|dry-run|attach|activate|detach>
     - yai ws domain <get|set>
     - yai ws recovery <status|load|reopen>
     - yai ws debug resolution
     - yai ws query <family> (fallback substrate) */
  if (strcmp(argv[cmdi], "ws") == 0) {
    if (cmdi + 1 >= argc || !argv[cmdi + 1]) {
      return set_err(req, "missing container action", "Run: yai help ws");
    }
    if (strcmp(argv[cmdi + 1], "create") == 0 ||
        strcmp(argv[cmdi + 1], "reset") == 0 ||
        strcmp(argv[cmdi + 1], "destroy") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing container id", "Run: yai ws create <ws-id>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      if (strcmp(argv[cmdi + 1], "create") == 0) req->command_id = "yai.container.create";
      else if (strcmp(argv[cmdi + 1], "reset") == 0) req->command_id = "yai.container.reset";
      else req->command_id = "yai.container.destroy";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "open") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing container id", "Run: yai ws open <ws-id>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.open";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "set") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing container id", "Run: yai ws set <ws-id>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.set";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "switch") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing container id", "Run: yai ws switch <ws-id>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.switch";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "current") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.current";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "status") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.status";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "inspect") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.inspect";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "unset") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.unset";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "clear") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.clear";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "domain") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2]) {
        return set_err(req, "missing domain action (get/set)", "Run: yai ws domain get|set ...");
      }
      if (strcmp(argv[cmdi + 2], "get") == 0) {
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.domain_get";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "set") == 0) {
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.domain_set";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      return set_err(req, "unknown domain action", "Run: yai ws domain get|set ...");
    }
    if (strcmp(argv[cmdi + 1], "policy") == 0) {
      if (cmdi + 2 < argc && argv[cmdi + 2] && strcmp(argv[cmdi + 2], "effective") == 0) {
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.policy_effective";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (cmdi + 2 < argc && argv[cmdi + 2] && strcmp(argv[cmdi + 2], "dry-run") == 0) {
        if (cmdi + 3 >= argc || !argv[cmdi + 3] || !argv[cmdi + 3][0]) {
          return set_err(req, "missing policy object id", "Run: yai ws policy dry-run <object-id>");
        }
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.policy_dry_run";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (cmdi + 2 < argc && argv[cmdi + 2] && strcmp(argv[cmdi + 2], "attach") == 0) {
        if (cmdi + 3 >= argc || !argv[cmdi + 3] || !argv[cmdi + 3][0]) {
          return set_err(req, "missing policy object id", "Run: yai ws policy attach <object-id>");
        }
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.policy_attach";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (cmdi + 2 < argc && argv[cmdi + 2] && strcmp(argv[cmdi + 2], "activate") == 0) {
        if (cmdi + 3 >= argc || !argv[cmdi + 3] || !argv[cmdi + 3][0]) {
          return set_err(req, "missing policy object id", "Run: yai ws policy activate <object-id>");
        }
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.policy_activate";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (cmdi + 2 < argc && argv[cmdi + 2] && strcmp(argv[cmdi + 2], "detach") == 0) {
        if (cmdi + 3 >= argc || !argv[cmdi + 3] || !argv[cmdi + 3][0]) {
          return set_err(req, "missing policy object id", "Run: yai ws policy detach <object-id>");
        }
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.policy_detach";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      return set_err(req, "unknown policy action", "Run: yai ws policy effective|dry-run|attach|activate|detach");
    }
    if (strcmp(argv[cmdi + 1], "run") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing container run action", "Run: yai ws run <action> [context tokens]");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.run";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "debug") == 0) {
      if (cmdi + 2 < argc && argv[cmdi + 2] && strcmp(argv[cmdi + 2], "resolution") == 0) {
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.debug_resolution";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      return set_err(req, "unknown debug action", "Run: yai ws debug resolution");
    }
    if (strcmp(argv[cmdi + 1], "graph") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2]) {
        return set_err(req, "missing graph action", "Run: yai ws graph <summary|container|governance|decision|evidence|authority|artifact|lineage|recent>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      if (strcmp(argv[cmdi + 2], "summary") == 0) req->command_id = "yai.container.graph.summary";
      else if (strcmp(argv[cmdi + 2], "container") == 0) req->command_id = "yai.container.graph.container";
      else if (strcmp(argv[cmdi + 2], "governance") == 0) req->command_id = "yai.container.graph.governance";
      else if (strcmp(argv[cmdi + 2], "decision") == 0) req->command_id = "yai.container.graph.decision";
      else if (strcmp(argv[cmdi + 2], "evidence") == 0) req->command_id = "yai.container.graph.evidence";
      else if (strcmp(argv[cmdi + 2], "authority") == 0) req->command_id = "yai.container.graph.authority";
      else if (strcmp(argv[cmdi + 2], "artifact") == 0) req->command_id = "yai.container.graph.artifact";
      else if (strcmp(argv[cmdi + 2], "lineage") == 0) req->command_id = "yai.container.graph.lineage";
      else if (strcmp(argv[cmdi + 2], "recent") == 0) req->command_id = "yai.container.graph.recent";
      else return set_err(req, "unknown graph action", "Run: yai ws graph <summary|container|governance|decision|evidence|authority|artifact|lineage|recent>");
      req->cmd_argc = argc - (cmdi + 3);
      req->cmd_argv = &argv[cmdi + 3];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "data") == 0) {
      const char *family = NULL;
      const char *command_id = NULL;
      if (cmdi + 2 >= argc || !argv[cmdi + 2]) {
        return set_err(req, "missing data action", "Run: yai ws data <events|evidence|governance|authority|artifacts|enforcement>");
      }
      if (strcmp(argv[cmdi + 2], "events") == 0) { family = "events"; command_id = "yai.container.data.events"; }
      else if (strcmp(argv[cmdi + 2], "evidence") == 0) { family = "evidence"; command_id = "yai.container.data.evidence"; }
      else if (strcmp(argv[cmdi + 2], "governance") == 0) { family = "governance"; command_id = "yai.container.data.governance"; }
      else if (strcmp(argv[cmdi + 2], "authority") == 0) { family = "authority"; command_id = "yai.container.data.authority"; }
      else if (strcmp(argv[cmdi + 2], "artifacts") == 0) { family = "artifact"; command_id = "yai.container.data.artifacts"; }
      else if (strcmp(argv[cmdi + 2], "enforcement") == 0) { family = "enforcement"; command_id = "yai.container.data.enforcement"; }
      else return set_err(req, "unknown data action", "Run: yai ws data <events|evidence|governance|authority|artifacts|enforcement>");
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = command_id;
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      req->cmd_argv[0] = (char *)family;
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "cognition") == 0) {
      const char *family = NULL;
      const char *command_id = NULL;
      if (cmdi + 2 >= argc || !argv[cmdi + 2]) {
        return set_err(req, "missing cognition action", "Run: yai ws cognition <status|transient|memory|providers|context>");
      }
      if (strcmp(argv[cmdi + 2], "status") == 0) {
        req->kind = YAI_PORCELAIN_KIND_COMMAND;
        req->command_id = "yai.container.inspect";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "transient") == 0) { family = "transient"; command_id = "yai.container.cognition.transient"; }
      else if (strcmp(argv[cmdi + 2], "memory") == 0) { family = "memory"; command_id = "yai.container.cognition.memory"; }
      else if (strcmp(argv[cmdi + 2], "providers") == 0) { family = "providers"; command_id = "yai.container.cognition.providers"; }
      else if (strcmp(argv[cmdi + 2], "context") == 0) { family = "context"; command_id = "yai.container.cognition.context"; }
      else return set_err(req, "unknown cognition action", "Run: yai ws cognition <status|transient|memory|providers|context>");
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = command_id;
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      req->cmd_argv[0] = (char *)family;
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "db") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2]) {
        return set_err(req, "missing db action", "Run: yai ws db <status|bindings|stores|classes|count|tail>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      if (strcmp(argv[cmdi + 2], "status") == 0) {
        req->command_id = "yai.container.status";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "bindings") == 0 || strcmp(argv[cmdi + 2], "stores") == 0) {
        req->command_id = "yai.container.inspect";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "classes") == 0) {
        req->command_id = "yai.container.db.classes";
        req->cmd_argc = argc - (cmdi + 2);
        req->cmd_argv = &argv[cmdi + 2];
        req->cmd_argv[0] = "container";
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "count") == 0) {
        req->command_id = "yai.container.db.count";
        req->cmd_argc = argc - (cmdi + 2);
        req->cmd_argv = &argv[cmdi + 2];
        req->cmd_argv[0] = "events";
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "tail") == 0) {
        req->command_id = "yai.container.db.tail";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      return set_err(req, "unknown db action", "Run: yai ws db <status|bindings|stores|classes|count|tail>");
    }
    if (strcmp(argv[cmdi + 1], "recovery") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2]) {
        return set_err(req, "missing recovery action", "Run: yai ws recovery <status|load|reopen>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      if (strcmp(argv[cmdi + 2], "status") == 0) {
        req->command_id = "yai.container.status";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "load") == 0) {
        req->command_id = "yai.container.lifecycle.maintain";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      if (strcmp(argv[cmdi + 2], "reopen") == 0) {
        req->command_id = "yai.container.open";
        req->cmd_argc = argc - (cmdi + 3);
        req->cmd_argv = &argv[cmdi + 3];
        return 0;
      }
      return set_err(req, "unknown recovery action", "Run: yai ws recovery <status|load|reopen>");
    }
    if (strcmp(argv[cmdi + 1], "query") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing query family", "Run: yai ws query <family>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.query";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "prompt-context") == 0 ||
        strcmp(argv[cmdi + 1], "prompt-token") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.container.prompt_context";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    return set_err(req, "unknown container action", "Run: yai help ws");
  }

  /* Canonical source-plane operator topology:
     - yai source enroll <source-label> [--owner-ref ...] [--source-node-id ...] [--daemon-instance-id ...]
     - yai source attach <source-node-id> [--scope ...] [--constraints-ref ...] [--owner-container-id ...]
     - yai source list
     - yai source status
     - yai source inspect
     - yai source retry-drain */
  if (strcmp(argv[cmdi], "source") == 0) {
    if (cmdi + 1 >= argc || !argv[cmdi + 1]) {
      return set_err(req, "missing source action", "Run: yai help source");
    }
    if (strcmp(argv[cmdi + 1], "enroll") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing source label", "Run: yai source enroll <source-label>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.source.enroll";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "attach") == 0) {
      if (cmdi + 2 >= argc || !argv[cmdi + 2] || !argv[cmdi + 2][0]) {
        return set_err(req, "missing source node id", "Run: yai source attach <source-node-id>");
      }
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.source.attach";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "list") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.source.list";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "status") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.source.status";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "inspect") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.source.inspect";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    if (strcmp(argv[cmdi + 1], "retry-drain") == 0) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = "yai.source.retry_drain";
      req->cmd_argc = argc - (cmdi + 2);
      req->cmd_argv = &argv[cmdi + 2];
      return 0;
    }
    return set_err(req, "unknown source action", "Run: yai help source");
  }

  if (strcmp(argv[cmdi], "doctor") == 0 ||
      strcmp(argv[cmdi], "inspect") == 0 ||
      strcmp(argv[cmdi], "verify") == 0) {
    const char *entrypoint = argv[cmdi];
    const char *topic = (cmdi + 1 < argc) ? argv[cmdi + 1] : NULL;
    const char *mapped = NULL;
    if (!topic || !topic[0]) {
      static char hint[64];
      snprintf(hint, sizeof(hint), "Run: yai help %s", entrypoint);
      return set_err(req, "missing topic", hint);
    }
    mapped = map_operator_capability_command_id(entrypoint, topic);
    if (!mapped) {
      static char hint[64];
      snprintf(hint, sizeof(hint), "Run: yai help %s", entrypoint);
      return set_err(req, "unknown topic", hint);
    }
    req->kind = YAI_PORCELAIN_KIND_COMMAND;
    req->command_id = mapped;
    req->cmd_argc = argc - (cmdi + 2);
    req->cmd_argv = &argv[cmdi + 2];
    return 0;
  }

  if (strcmp(argv[cmdi], "watch") == 0) {
    if (cmdi + 1 >= argc || !argv[cmdi + 1]) {
      return set_err(req, "missing watch target", "Run: yai watch <entrypoint> <topic> [op]");
    }
    req->kind = YAI_PORCELAIN_KIND_WATCH;
    req->cmd_argc = argc - (cmdi + 1);
    req->cmd_argv = &argv[cmdi + 1];
    req->watch_interval_ms = find_watch_int_arg(argc, argv, "--interval-ms", 2000);
    req->watch_interval_ms = find_watch_interval_ms(argc, argv, req->watch_interval_ms);
    req->watch_count = find_watch_int_arg(argc, argv, "--count", 0);
    if (has_watch_once_flag(argc, argv)) req->watch_count = 1;
    req->watch_no_clear = has_watch_no_clear_flag(argc, argv);
    return 0;
  }

  /* From here on, resolve commands via SDK catalog (shell stays registry-agnostic). */
  yai_sdk_command_catalog_t cat;
  if (yai_sdk_command_catalog_load(&cat) != 0) {
    return set_err(req, "catalog unavailable", "Run: yai help");
  }

  /* Support alias invocation:
     `yai <group>-<name> ...` (e.g. `yai gov-policy ...` or `yai substrate-manifest ...`) */
  {
    const yai_sdk_command_ref_t* c_alias = find_by_alias(&cat, argv[cmdi], NULL, NULL);
    if (c_alias && c_alias->id[0]) {
      req->kind = YAI_PORCELAIN_KIND_COMMAND;
      req->command_id = c_alias->id;
      req->cmd_argc = argc - (cmdi + 1);
      req->cmd_argv = &argv[cmdi + 1];
      yai_sdk_command_catalog_free(&cat);
      return 0;
    }
  }

  /* Primary form: `yai <group> <name> ...` requires at least 2 tokens from cmdi. */
  if (cmdi + 1 >= argc || !argv[cmdi + 1]) {
    const char* group_only = expand_group_alias(argv[cmdi]);
    const yai_sdk_command_group_t *group_view = yai_sdk_command_catalog_find_group(&cat, group_only);
    int group_exists = (group_view && group_view->command_count > 0) ? 1 : 0;
    if (group_exists) {
      req->kind = YAI_PORCELAIN_KIND_HELP;
      req->help_token = argv[cmdi];
      req->help_exit_code = 20;
      yai_sdk_command_catalog_free(&cat);
      return 0;
    }
    yai_sdk_command_catalog_free(&cat);
    return set_err(req, "missing command name (expected: yai <group> <name>)", "Run: yai help");
  }

  const char* group_raw = argv[cmdi];
  const char* name      = argv[cmdi + 1];

  /* `yai <group> help` -> group help (even with abbreviations) */
  if (is_help_token(name)) {
    req->kind = YAI_PORCELAIN_KIND_HELP;
    req->help_token = group_raw;
    yai_sdk_command_catalog_free(&cat);
    return 0;
  }

  const char* group = expand_group_alias(group_raw);

  const yai_sdk_command_ref_t* cmd = find_by_group_name(&cat, group, name);
  if (!cmd || !cmd->id[0]) {
    const yai_sdk_command_group_t *group_view = yai_sdk_command_catalog_find_group(&cat, group);
    int group_exists = (group_view && group_view->command_count > 0) ? 1 : 0;
    yai_sdk_command_catalog_free(&cat);
    if (group_exists) {
      return set_err(req, "unknown command name for group", "Run: yai help <group>");
    }
    {
      static char msgbuf[192];
      snprintf(msgbuf, sizeof(msgbuf), "Unknown command group: %s", group_raw ? group_raw : "unknown");
      return set_err(req, msgbuf, "Run: yai help --groups");
    }
  }

  req->kind = YAI_PORCELAIN_KIND_COMMAND;
  req->command_id = cmd->id;
  yai_sdk_command_catalog_free(&cat);

  /* Forward remaining argv to command execution (post global-option parsing). */
  req->cmd_argc = argc - (cmdi + 2);
  req->cmd_argv = &argv[cmdi + 2];
  return 0;
}
