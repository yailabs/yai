/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/help.h"

#include "yai/shell/errors.h"
#include "yai/shell/pager.h"
#include "yai/sdk/catalog.h"

#include <stdio.h>
#include <string.h>
#include <stdarg.h>
#include <stdlib.h>

typedef struct {
  char *buf;
  size_t len;
  size_t cap;
} strbuf_t;

static int sb_reserve(strbuf_t *sb, size_t need)
{
  char *nb;
  size_t ncap;
  if (!sb) return 1;
  if (sb->len + need + 1 <= sb->cap) return 0;
  ncap = (sb->cap == 0) ? 1024 : sb->cap * 2;
  while (ncap < sb->len + need + 1) ncap *= 2;
  nb = (char *)realloc(sb->buf, ncap);
  if (!nb) return 1;
  sb->buf = nb;
  sb->cap = ncap;
  return 0;
}

static int sb_appendf(strbuf_t *sb, const char *fmt, ...)
{
  va_list ap;
  va_list ap2;
  int n;
  if (!sb || !fmt) return 1;
  va_start(ap, fmt);
  va_copy(ap2, ap);
  n = vsnprintf(NULL, 0, fmt, ap);
  va_end(ap);
  if (n < 0) { va_end(ap2); return 1; }
  if (sb_reserve(sb, (size_t)n) != 0) { va_end(ap2); return 1; }
  vsnprintf(sb->buf + sb->len, sb->cap - sb->len, fmt, ap2);
  sb->len += (size_t)n;
  va_end(ap2);
  return 0;
}

static void sb_free(strbuf_t *sb)
{
  if (!sb) return;
  free(sb->buf);
  sb->buf = NULL;
  sb->len = 0;
  sb->cap = 0;
}

static int help_error(const char *msg, const char *hint)
{
  fprintf(stderr, "help\n");
  fprintf(stderr, "BAD ARGS\n");
  fprintf(stderr, "%s\n", msg ? msg : "invalid help input");
  if (hint && hint[0]) fprintf(stderr, "Hint: %s\n", hint);
  return 20;
}

static const char *map_entrypoint(const char *group)
{
  if (!group) return "run";
  /* Legacy topology groups are still resolvable, but never canonical in help surfaces. */
  if (strcmp(group, "governance") == 0) return "gov";
  if (strcmp(group, "verify") == 0) return "verify";
  if (strcmp(group, "inspect") == 0) return "inspect";
  if (strcmp(group, "bundle") == 0) return "bundle";
  if (strcmp(group, "core") == 0) return "run";
  if (strcmp(group, "lifecycle") == 0) return "run";
  if (strcmp(group, "control") == 0) return "run";
  if (strcmp(group, "ingress") == 0 || strcmp(group, "boot") == 0 || strcmp(group, "exec") == 0 ||
      strcmp(group, "brain") == 0 || strcmp(group, "mind") == 0 ||
      strcmp(group, "memory") == 0 || strcmp(group, "substrate") == 0 || strcmp(group, "orch") == 0) return "run";
  return "config";
}

static int is_legacy_topology_group(const char *group)
{
  if (!group || !group[0]) return 0;
  return (strcmp(group, "brain") == 0 ||
          strcmp(group, "mind") == 0 ||
          strcmp(group, "memory") == 0 ||
          strcmp(group, "substrate") == 0 ||
          strcmp(group, "orch") == 0);
}

static void split_name_topic_op(const char *name, const char **topic, const char **op)
{
  static char t[64];
  static char o[64];
  const char *u;
  if (!name || !name[0]) { *topic = "general"; *op = "run"; return; }
  u = strchr(name, '_');
  if (!u) { *topic = "general"; *op = name; return; }
  {
    size_t tn = (size_t)(u - name);
    size_t on = strlen(u + 1);
    if (tn >= sizeof(t)) tn = sizeof(t) - 1;
    if (on >= sizeof(o)) on = sizeof(o) - 1;
    memcpy(t, name, tn); t[tn] = '\0';
    memcpy(o, u + 1, on); o[on] = '\0';
    *topic = t;
    *op = o;
  }
}

static int print_version(void)
{
  FILE *f = fopen("VERSION", "r");
  if (!f) { puts("yai-shell (version unknown)"); return 0; }
  {
    char buf[128];
    if (fgets(buf, sizeof(buf), f)) {
      size_t n = strlen(buf);
      while (n && (buf[n - 1] == '\n' || buf[n - 1] == '\r')) { buf[n - 1] = '\0'; n--; }
      printf("yai-shell %s\n", buf);
    } else {
      puts("yai-shell (version unknown)");
    }
  }
  fclose(f);
  return 0;
}

static int render_global_help(const yai_sdk_command_catalog_t *idx, strbuf_t *sb, int all)
{
  (void)idx;
  sb_appendf(sb, "YAI Command Surface\n\n");
  sb_appendf(sb, "Usage:\n");
  sb_appendf(sb, "  yai up|status|down|restart\n");
  sb_appendf(sb, "  yai <entrypoint> <topic> [op] [options]\n");
  sb_appendf(sb, "  yai help <entrypoint> [topic] [op]\n\n");
  sb_appendf(sb, "Operator Lifecycle:\n");
  sb_appendf(sb, "  up           start YAI service\n");
  sb_appendf(sb, "  status       check YAI service reachability\n");
  sb_appendf(sb, "  down         stop YAI service\n");
  sb_appendf(sb, "  restart      restart YAI service\n\n");
  sb_appendf(sb, "Main Surface Commands:\n");
  sb_appendf(sb, "  ws           container lifecycle\n");
  sb_appendf(sb, "  source       source-plane owner/daemon operations\n");
  sb_appendf(sb, "  run          runtime execution and control\n");
  sb_appendf(sb, "  gov          governance operations\n");
  sb_appendf(sb, "  verify       verification and proofs\n");
  sb_appendf(sb, "  inspect      read-only introspection\n");
  sb_appendf(sb, "  bundle       bundle operations\n");
  sb_appendf(sb, "  config       configuration\n");
  sb_appendf(sb, "  doctor       diagnostics\n");
  sb_appendf(sb, "  watch        watch mode\n");
  sb_appendf(sb, "  help         help and references\n");
  sb_appendf(sb, "  version      version information\n");
  if (all) {
    sb_appendf(sb, "\nCompatibility Aliases (non-authoritative):\n");
    sb_appendf(sb, "  ingress/core/boot/exec/control\n");
    sb_appendf(sb, "  legacy topology aliases are accepted but hidden from canonical help\n");
  }
  return 0;
}

static int render_plumbing_help(strbuf_t *sb)
{
  sb_appendf(sb, "YAI Command Surface\n\n");
  sb_appendf(sb, "Plumbing Commands (advanced/internal)\n");
  sb_appendf(sb, "  ingress\n");
  sb_appendf(sb, "  core\n");
  sb_appendf(sb, "  boot\n");
  sb_appendf(sb, "  exec\n");
  sb_appendf(sb, "  lifecycle (compat alias: use up|down|restart)\n");
  sb_appendf(sb, "  control\n");
  sb_appendf(sb, "\nLegacy aliases retained for compatibility only: brain, mind, memory, substrate, orch\n");
  sb_appendf(sb, "\nHint: Run 'yai help <entrypoint>' for details.\n");
  return 0;
}

static int is_gov_topic(const char *topic)
{
  return topic &&
         (strcmp(topic, "decision") == 0 ||
          strcmp(topic, "evidence") == 0 ||
          strcmp(topic, "event") == 0 ||
          strcmp(topic, "effect") == 0 ||
          strcmp(topic, "disclosure") == 0);
}

static int render_watch_entrypoint_help(strbuf_t *sb)
{
  sb_appendf(sb, "watch\n\n");
  sb_appendf(sb, "Topics:\n");
  sb_appendf(sb, "  runtime            watch runtime reachability loops\n");
  sb_appendf(sb, "  source             watch source-plane status loops\n");
  sb_appendf(sb, "  gov                watch governance command loops\n");
  sb_appendf(sb, "  verify             watch verification command loops\n");
  sb_appendf(sb, "  inspect            watch inspect command loops (runtime/container/catalog/context + projections)\n");
  sb_appendf(sb, "\nDX-2 projection targets via inspect topic:\n");
  sb_appendf(sb, "  yai watch inspect <source|edge|mesh|grant|transport|ingress|case> [--interval-ms <ms>] [--count <n>]\n");
  sb_appendf(sb, "\nUse: yai help watch <topic>\n");
  sb_appendf(sb, "Compatibility alias: service -> runtime\n");
  return 0;
}

static int render_watch_topic_help(const char *topic, strbuf_t *sb)
{
  if (!topic || !topic[0]) {
    return help_error("missing topic for entrypoint 'watch'", "Run: yai help watch");
  }
  if (!(strcmp(topic, "service") == 0 ||
        strcmp(topic, "runtime") == 0 ||
        strcmp(topic, "source") == 0 ||
        strcmp(topic, "gov") == 0 ||
        strcmp(topic, "verify") == 0 ||
        strcmp(topic, "inspect") == 0)) {
    return help_error("unknown topic under 'watch'", "Run: yai help watch");
  }
  sb_appendf(sb, "watch %s\n\n", (strcmp(topic, "service") == 0) ? "runtime" : topic);
  sb_appendf(sb, "Usage:\n");
  if (strcmp(topic, "service") == 0 || strcmp(topic, "runtime") == 0) {
    sb_appendf(sb, "  yai watch runtime ping [--interval-ms <ms>] [--count <n>]\n");
  } else if (strcmp(topic, "source") == 0) {
    sb_appendf(sb, "  yai watch source status [--interval-ms <ms>] [--count <n>]\n");
  } else if (strcmp(topic, "gov") == 0) {
    sb_appendf(sb, "  yai watch gov decision status [--interval-ms <ms>] [--count <n>]\n");
  } else if (strcmp(topic, "verify") == 0) {
    sb_appendf(sb, "  yai watch verify law [--interval-ms <ms>] [--count <n>]\n");
  } else {
    sb_appendf(sb, "  yai watch inspect container [--interval-ms <ms>] [--count <n>]\n");
    sb_appendf(sb, "  yai watch inspect runtime [--interval-ms <ms>] [--count <n>]\n");
    sb_appendf(sb, "  yai watch inspect <source|edge|mesh|grant|transport|ingress|case> [--interval-ms <ms>] [--count <n>]\n");
  }
  sb_appendf(sb, "\nNotes:\n");
  sb_appendf(sb, "  Press 'q' or ESC to quit interactive watch mode.\n");
  return 0;
}

static int render_gov_entrypoint_help(strbuf_t *sb)
{
  sb_appendf(sb, "gov\n\n");
  sb_appendf(sb, "Topics:\n");
  sb_appendf(sb, "  decision           governance decisions and records\n");
  sb_appendf(sb, "  evidence           governance evidence collection\n");
  sb_appendf(sb, "  event              governance event records\n");
  sb_appendf(sb, "  effect             governed effect handling\n");
  sb_appendf(sb, "  disclosure         disclosure workflows\n");
  sb_appendf(sb, "\nUse: yai help gov <topic>\n");
  return 0;
}

static int append_op_once(strbuf_t *sb, char ops[][64], size_t *count, const char *op, const char *desc)
{
  if (!sb || !ops || !count || !op || !op[0]) return 1;
  for (size_t i = 0; i < *count; i++) {
    if (strcmp(ops[i], op) == 0) return 0;
  }
  if (*count >= 32) return 0;
  snprintf(ops[*count], 64, "%s", op);
  (*count)++;
  sb_appendf(sb, "  %-16s %s\n", op, (desc && desc[0]) ? desc : "governance operation");
  return 0;
}

static int render_gov_topic_help(const yai_sdk_command_catalog_t *idx, const char *topic, strbuf_t *sb)
{
  const yai_sdk_command_group_t *g = NULL;
  char listed[32][64];
  size_t listed_count = 0;
  size_t i;
  if (!topic || !topic[0]) {
    return help_error("missing topic for entrypoint 'gov'", "Run: yai help gov");
  }
  if (!is_gov_topic(topic)) {
    return help_error("unknown topic under 'gov'", "Run: yai help gov");
  }

  memset(listed, 0, sizeof(listed));
  sb_appendf(sb, "gov %s\n\n", topic);
  sb_appendf(sb, "Operations:\n");

  if (strcmp(topic, "decision") == 0) {
    append_op_once(sb, listed, &listed_count, "make", "create a decision record");
    append_op_once(sb, listed, &listed_count, "status", "inspect decision state");
    append_op_once(sb, listed, &listed_count, "trace", "inspect traceability chain");
    append_op_once(sb, listed, &listed_count, "validate", "validate decision contract");
    append_op_once(sb, listed, &listed_count, "publish", "publish decision outcome");
  } else if (strcmp(topic, "evidence") == 0) {
    append_op_once(sb, listed, &listed_count, "status", "inspect evidence state");
    append_op_once(sb, listed, &listed_count, "list", "list evidence records");
  } else if (strcmp(topic, "event") == 0) {
    append_op_once(sb, listed, &listed_count, "list", "list governance events");
    append_op_once(sb, listed, &listed_count, "status", "inspect event status");
  }

  g = yai_sdk_command_catalog_find_group(idx, "governance");
  if (g) {
    for (i = 0; i < g->command_count; i++) {
      const char *name = g->commands[i].name;
      const char *u = NULL;
      const char *op = NULL;
      if (!name || !name[0]) continue;
      u = strchr(name, '_');
      if (!u) continue;
      if ((size_t)(u - name) != strlen(topic) || strncmp(name, topic, strlen(topic)) != 0) continue;
      op = u + 1;
      if (!op[0]) continue;
      append_op_once(sb, listed, &listed_count, op, g->commands[i].summary);
    }
  }

  if (listed_count == 0) {
    sb_appendf(sb, "  status           inspect %s state\n", topic);
  }
  sb_appendf(sb, "\nExamples:\n");
  if (strcmp(topic, "decision") == 0) {
    sb_appendf(sb, "  yai gov decision status\n");
    sb_appendf(sb, "  yai gov decision trace\n");
  } else {
    sb_appendf(sb, "  yai gov %s status\n", topic);
  }
  return 0;
}

static int render_entrypoint_help(const yai_sdk_command_catalog_t *idx, const char *entrypoint, strbuf_t *sb)
{
  int found = 0;
  if (strcmp(entrypoint, "watch") == 0) {
    return render_watch_entrypoint_help(sb);
  }
  if (strcmp(entrypoint, "gov") == 0) {
    return render_gov_entrypoint_help(sb);
  }
  if (strcmp(entrypoint, "ws") == 0) {
    sb_appendf(sb, "ws\n\n");
    sb_appendf(sb, "Container commands:\n");
    sb_appendf(sb, "  graph                      container graph read surfaces\n");
    sb_appendf(sb, "  db                         container data-store/binding read surfaces\n");
    sb_appendf(sb, "  data                       container record query surfaces\n");
    sb_appendf(sb, "  cognition                  container cognition/transient query surfaces\n");
    sb_appendf(sb, "  policy                     container governance attach/effective surfaces\n");
    sb_appendf(sb, "  domain                     container domain context surfaces\n");
    sb_appendf(sb, "  recovery                   container recovery/load/reopen surfaces\n");
    sb_appendf(sb, "  debug                      container resolution diagnostics\n");
    sb_appendf(sb, "  query <family>             generic fallback family query\n");
    sb_appendf(sb, "\nLifecycle and context:\n");
    sb_appendf(sb, "  create <ws-id>             create container runtime state\n");
    sb_appendf(sb, "  open <ws-id>               open and bind existing container\n");
    sb_appendf(sb, "  reset <ws-id>              reset container runtime state\n");
    sb_appendf(sb, "  destroy <ws-id>            destroy container runtime state\n");
    sb_appendf(sb, "  set <ws-id>                set active container binding\n");
    sb_appendf(sb, "  switch <ws-id>             switch active container binding\n");
    sb_appendf(sb, "  current                    show current container\n");
    sb_appendf(sb, "  status                     show container binding/runtime status\n");
    sb_appendf(sb, "  inspect                    show expanded container snapshot\n");
    sb_appendf(sb, "  unset                      remove active container binding\n");
    sb_appendf(sb, "  clear                      clear active container runtime state\n");
    sb_appendf(sb, "  domain get                 show declared/inferred/effective domain context\n");
    sb_appendf(sb, "  domain set --family F --specialization S\n");
    sb_appendf(sb, "                             set declared container context\n");
    sb_appendf(sb, "  run <action> [tokens...]   run a real runtime action in active container context\n");
    sb_appendf(sb, "  policy dry-run <object-id> show eligibility/compatibility preview for container apply\n");
    sb_appendf(sb, "  policy attach <object-id>  attach governable policy object to active container\n");
    sb_appendf(sb, "  policy activate <object-id> activate an already attached governable object\n");
    sb_appendf(sb, "  policy detach <object-id>  detach governable policy object from active container\n");
    sb_appendf(sb, "  policy effective           show effective policy summary\n");
    sb_appendf(sb, "  debug resolution           show debug resolution summary\n");
    sb_appendf(sb, "  prompt-context             print compact container context payload\n");
    sb_appendf(sb, "  prompt-token               print compact prompt token (e.g. \u25c9 demo)\n");
    sb_appendf(sb, "\nUse: yai help ws <graph|db|data|cognition|policy|domain|recovery|debug>\n");
    return 0;
  }
  if (strcmp(entrypoint, "source") == 0) {
    sb_appendf(sb, "source\n\n");
    sb_appendf(sb, "Source-plane commands:\n");
    sb_appendf(sb, "  enroll <source-label>      enroll source node/daemon against owner container\n");
    sb_appendf(sb, "  attach <source-node-id>    attach source node to owner container context\n");
    sb_appendf(sb, "  list                       list source-plane summary for active container\n");
    sb_appendf(sb, "  status                     show source-plane health/summary for active container\n");
    sb_appendf(sb, "  inspect                    inspect source-plane summary/capability state\n");
    sb_appendf(sb, "  retry-drain                trigger retry-drain control path when supported\n");
    sb_appendf(sb, "\nNotes:\n");
    sb_appendf(sb, "  source-plane commands target owner runtime control plane.\n");
    sb_appendf(sb, "  edge process binary remains yai-daemon; operator shell namespace is yai source.\n");
    sb_appendf(sb, "\nUse: yai help source <enroll|attach|list|status|inspect|retry-drain>\n");
    return 0;
  }
  sb_appendf(sb, "%s\n\n", entrypoint);
  sb_appendf(sb, "Topics:\n");
  if (strcmp(entrypoint, "doctor") == 0) {
    sb_appendf(sb, "  env                environment readiness checks\n");
    sb_appendf(sb, "  runtime            runtime reachability checks\n");
    sb_appendf(sb, "  container          container binding checks\n");
    sb_appendf(sb, "  pins               dependency pin checks\n");
    sb_appendf(sb, "  config             configuration checks\n");
    sb_appendf(sb, "  all                aggregated diagnostics\n");
    sb_appendf(sb, "\nCompatibility alias: service -> runtime\n");
    return 0;
  }
  if (strcmp(entrypoint, "inspect") == 0) {
    sb_appendf(sb, "  container          container state snapshot\n");
    sb_appendf(sb, "  runtime            runtime state snapshot\n");
    sb_appendf(sb, "  catalog            command catalog snapshot\n");
    sb_appendf(sb, "  context            current context snapshot\n");
    sb_appendf(sb, "  source             source-plane inspect projection (owner-side)\n");
    sb_appendf(sb, "  edge               subordinate edge runtime inspect projection\n");
    sb_appendf(sb, "  mesh               mesh membership/coordination inspect projection\n");
    sb_appendf(sb, "  grant              delegated grant/policy inspect projection\n");
    sb_appendf(sb, "  transport          overlay/transport condition inspect projection\n");
    sb_appendf(sb, "  ingress            owner ingress condition inspect projection\n");
    sb_appendf(sb, "  case               governed case-state inspect projection\n");
    sb_appendf(sb, "\nCompatibility alias: service -> runtime\n");
    return 0;
  }
  if (strcmp(entrypoint, "verify") == 0) {
    sb_appendf(sb, "  law                law artifacts availability\n");
    sb_appendf(sb, "  registry           model/registry/catalog coherence\n");
    sb_appendf(sb, "  runtime            runtime contract reachability\n");
    sb_appendf(sb, "  container          container model checks\n");
    sb_appendf(sb, "  reply              reply architecture checks\n");
    sb_appendf(sb, "  alignment          cross-repo alignment checks\n");
    sb_appendf(sb, "\nCompatibility alias: service -> runtime\n");
    return 0;
  }
  for (size_t i = 0; i < idx->group_count; i++) {
    const yai_sdk_command_group_t *g = &idx->groups[i];
    const char *ep = map_entrypoint(g->group);
    if (is_legacy_topology_group(g->group)) continue;
    if (strcmp(ep, entrypoint) != 0) continue;
    sb_appendf(sb, "  %-18s %s\n", g->group, "registry topic");
    found = 1;
  }
  if (!found) return help_error("unknown entrypoint", "Run: yai help");
  return 0;
}

static int render_topic_help(const yai_sdk_command_catalog_t *idx, const char *entrypoint, const char *topic, strbuf_t *sb)
{
  if (strcmp(entrypoint, "watch") == 0) {
    return render_watch_topic_help(topic, sb);
  }
  if (strcmp(entrypoint, "gov") == 0) {
    return render_gov_topic_help(idx, topic, sb);
  }
  if (strcmp(entrypoint, "ws") == 0) {
    if (strcmp(topic, "graph") == 0) {
      sb_appendf(sb, "ws graph\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws graph <summary|container|governance|decision|evidence|authority|artifact|lineage|recent>\n");
      return 0;
    }
    if (strcmp(topic, "db") == 0) {
      sb_appendf(sb, "ws db\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws db <status|bindings|stores|classes|count|tail>\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  canonical path: use ws db first; internal backing may compose status/inspect/query.\n");
      sb_appendf(sb, "  fallback substrate (secondary): yai ws query <family>.\n");
      return 0;
    }
    if (strcmp(topic, "data") == 0) {
      sb_appendf(sb, "ws data\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws data <events|evidence|governance|authority|artifacts|enforcement>\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  data family uses container query substrate with canonical ws grammar.\n");
      return 0;
    }
    if (strcmp(topic, "cognition") == 0) {
      sb_appendf(sb, "ws cognition\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws cognition <status|transient|memory|providers|context>\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  canonical path: use ws cognition first.\n");
      sb_appendf(sb, "  fallback substrate (secondary): yai ws query transient|memory|providers|context.\n");
      return 0;
    }
    if (strcmp(topic, "policy") == 0) {
      sb_appendf(sb, "ws policy\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws policy <dry-run|attach|activate|detach|effective> [object-id]\n");
      return 0;
    }
    if (strcmp(topic, "domain") == 0) {
      sb_appendf(sb, "ws domain\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws domain get\n");
      sb_appendf(sb, "  yai ws domain set --family <F> --specialization <S>\n");
      return 0;
    }
    if (strcmp(topic, "recovery") == 0) {
      sb_appendf(sb, "ws recovery\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws recovery <status|load|reopen> [ws-id]\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  canonical path: use ws recovery first.\n");
      sb_appendf(sb, "  some actions remain composition-backed until dedicated runtime ids are complete.\n");
      return 0;
    }
    if (strcmp(topic, "debug") == 0) {
      sb_appendf(sb, "ws debug\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws debug resolution\n");
      return 0;
    }
    if (strcmp(topic, "query") == 0) {
      sb_appendf(sb, "ws query\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai ws query <family>\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  query is fallback substrate, not preferred where dedicated ws families exist.\n");
      return 0;
    }
    return help_error("unknown topic under 'ws'", "Run: yai help ws");
  }
  if (strcmp(entrypoint, "source") == 0) {
    if (strcmp(topic, "enroll") == 0) {
      sb_appendf(sb, "source enroll\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai source enroll <source-label> [--owner-ref <uds://...>] [--source-node-id <id>] [--daemon-instance-id <id>] [--ws-id <ws-id>]\n");
      return 0;
    }
    if (strcmp(topic, "attach") == 0) {
      sb_appendf(sb, "source attach\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai source attach <source-node-id> [--scope <container|...>] [--constraints-ref <ref>] [--owner-container-id <ws-id>] [--ws-id <ws-id>]\n");
      return 0;
    }
    if (strcmp(topic, "list") == 0) {
      sb_appendf(sb, "source list\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai source list [--ws-id <ws-id>]\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  list uses owner-side source summary read model.\n");
      return 0;
    }
    if (strcmp(topic, "status") == 0) {
      sb_appendf(sb, "source status\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai source status [--ws-id <ws-id>]\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  status reports owner-side source-plane summary and readiness hints.\n");
      return 0;
    }
    if (strcmp(topic, "inspect") == 0) {
      sb_appendf(sb, "source inspect\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai source inspect [--ws-id <ws-id>]\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  inspect returns source-plane summary with graph/query counters.\n");
      return 0;
    }
    if (strcmp(topic, "retry-drain") == 0) {
      sb_appendf(sb, "source retry-drain\n\n");
      sb_appendf(sb, "Usage:\n");
      sb_appendf(sb, "  yai source retry-drain [--ws-id <ws-id>]\n");
      sb_appendf(sb, "\nNotes:\n");
      sb_appendf(sb, "  retry-drain is runtime-wave dependent; command may return not implemented.\n");
      return 0;
    }
    return help_error("unknown topic under 'source'", "Run: yai help source");
  }
  if (strcmp(entrypoint, "inspect") == 0) {
    if (!(strcmp(topic, "container") == 0 || strcmp(topic, "runtime") == 0 ||
          strcmp(topic, "catalog") == 0 || strcmp(topic, "context") == 0 ||
          strcmp(topic, "source") == 0 || strcmp(topic, "edge") == 0 ||
          strcmp(topic, "mesh") == 0 || strcmp(topic, "grant") == 0 ||
          strcmp(topic, "transport") == 0 || strcmp(topic, "ingress") == 0 ||
          strcmp(topic, "case") == 0 || strcmp(topic, "service") == 0)) {
      return help_error("unknown topic under 'inspect'", "Run: yai help inspect");
    }
    sb_appendf(sb, "%s %s\n\n", entrypoint, topic);
    sb_appendf(sb, "Usage:\n");
    sb_appendf(sb, "  yai %s %s [options]\n\n", entrypoint, topic);
    if (strcmp(topic, "source") == 0 || strcmp(topic, "edge") == 0 ||
        strcmp(topic, "mesh") == 0 || strcmp(topic, "grant") == 0 ||
        strcmp(topic, "transport") == 0 || strcmp(topic, "ingress") == 0 ||
        strcmp(topic, "case") == 0) {
      sb_appendf(sb, "Notes:\n");
      sb_appendf(sb, "  DX-2 projection topic: routed to canonical owner-side inspect/query surfaces.\n");
      sb_appendf(sb, "  Projection does not create a separate authority plane.\n");
      sb_appendf(sb, "  Use --json for machine output.\n");
    } else {
      sb_appendf(sb, "Notes:\n");
      sb_appendf(sb, "  Use --json for machine output.\n");
    }
    return 0;
  }
  if (strcmp(entrypoint, "doctor") == 0 || strcmp(entrypoint, "verify") == 0) {
    sb_appendf(sb, "%s %s\n\n", entrypoint, topic);
    sb_appendf(sb, "Usage:\n");
    sb_appendf(sb, "  yai %s %s [options]\n\n", entrypoint, topic);
    sb_appendf(sb, "Notes:\n");
    sb_appendf(sb, "  Use --json for machine output.\n");
    return 0;
  }
  const yai_sdk_command_group_t *g = yai_sdk_command_catalog_find_group(idx, topic);
  if (!g || strcmp(map_entrypoint(g->group), entrypoint) != 0) {
    return help_error("unknown topic", "Run: yai help <entrypoint>");
  }
  sb_appendf(sb, "%s %s\n\n", entrypoint, topic);
  sb_appendf(sb, "Operations:\n");
  for (size_t j = 0; j < g->command_count; j++) {
    const char *t = NULL;
    const char *op = NULL;
    split_name_topic_op(g->commands[j].name, &t, &op);
    if (strcmp(t, topic) == 0 || strcmp(t, "general") == 0) {
      sb_appendf(sb, "  %-20s %s\n", op, g->commands[j].summary);
    } else {
      sb_appendf(sb, "  %-20s %s\n", g->commands[j].name, g->commands[j].summary);
    }
  }
  return 0;
}

static int render_command_help(const yai_sdk_command_ref_t *c, strbuf_t *sb)
{
  const char *topic = NULL;
  const char *op = NULL;
  const char *ep;
  if (!c) return help_error("unknown command", "Run: yai help");
  ep = map_entrypoint(c->group);
  split_name_topic_op(c->name, &topic, &op);
  sb_appendf(sb, "Command:\n");
  sb_appendf(sb, "  %s %s %s\n\n", ep, c->group, c->name);
  sb_appendf(sb, "Description:\n");
  sb_appendf(sb, "  %s\n\n", c->summary);
  sb_appendf(sb, "Usage:\n");
  sb_appendf(sb, "  yai %s %s %s\n", ep, topic, op);
  if (is_legacy_topology_group(c->group)) {
    sb_appendf(sb, "  compatibility: legacy group route available (hidden from canonical help)\n\n");
  } else {
    sb_appendf(sb, "  compatibility: registry group route available\n\n");
  }
  sb_appendf(sb, "Metadata:\n");
  sb_appendf(sb, "  canonical_id: %s\n", c->id);
  return 0;
}

int yai_porcelain_help_print(const char *token1, const char *token2, const char *token3, int pager, int no_pager)
{
  yai_sdk_command_catalog_t idx;
  strbuf_t sb = {0};
  int rc;

  if (token1 && strcmp(token1, "version") == 0) return print_version();

  rc = yai_sdk_command_catalog_load(&idx);
  if (rc != 0) {
    yai_porcelain_err_print(YAI_PORCELAIN_ERR_DEP_MISSING, "registry unavailable (compatibility law export unreadable or invalid)");
    return yai_porcelain_err_exit_code(YAI_PORCELAIN_ERR_DEP_MISSING);
  }

  if (!token1 || !token1[0]) {
    rc = render_global_help(&idx, &sb, 0);
  } else if (strcmp(token1, "--all") == 0 || strcmp(token1, "-a") == 0) {
    rc = render_global_help(&idx, &sb, 1);
  } else if (strcmp(token1, "--plumbing") == 0) {
    rc = render_plumbing_help(&sb);
  } else if (strcmp(token1, "--groups") == 0 || strcmp(token1, "-g") == 0 || strcmp(token1, "topics") == 0) {
    rc = render_global_help(&idx, &sb, 0);
  } else if (strncmp(token1, "yai.", 4) == 0) {
    rc = render_command_help(yai_sdk_command_catalog_find_by_id(&idx, token1), &sb);
  } else if (token2 && token2[0] && token3 && token3[0]) {
    const yai_sdk_command_group_t *g = yai_sdk_command_catalog_find_group(&idx, token2);
    const yai_sdk_command_ref_t *c = g ? yai_sdk_command_catalog_find_command(&idx, token2, token3) : NULL;
    rc = render_command_help(c, &sb);
  } else if (token2 && token2[0]) {
    rc = render_topic_help(&idx, token1, token2, &sb);
  } else {
    rc = render_entrypoint_help(&idx, token1, &sb);
  }

  if (rc == 0 && sb.buf) yai_shell_page_if_needed(sb.buf, pager, no_pager);
  sb_free(&sb);
  yai_sdk_command_catalog_free(&idx);
  return rc;
}

int yai_porcelain_help_print_any(const char *token)
{
  return yai_porcelain_help_print(token, NULL, NULL, 0, 0);
}
