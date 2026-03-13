/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/render.h"
#include "yai/shell/display_map.h"
#include "yai/shell/color.h"
#include "yai/shell/style_map.h"

#include <cJSON.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

static void print_line(FILE *stream, const char *line, int color_enabled, const char *color)
{
  if (!line) return;
  yai_color_print(stream, color_enabled, color, line);
  fputc('\n', stream);
}

static void print_json_block(FILE *stream, const char *label, const char *json_text)
{
  cJSON *doc = NULL;
  char *pretty = NULL;
  if (!stream || !label) return;
  fprintf(stream, "%s:\n", label);
  if (!json_text || !json_text[0]) {
    fprintf(stream, "{}\n");
    return;
  }
  doc = cJSON_Parse(json_text);
  if (!doc) {
    fprintf(stream, "%s\n", json_text);
    return;
  }
  pretty = cJSON_Print(doc);
  cJSON_Delete(doc);
  if (!pretty) {
    fprintf(stream, "%s\n", json_text);
    return;
  }
  fprintf(stream, "%s\n", pretty);
  free(pretty);
}

static int build_subject(const yai_render_opts_t *opts, const yai_sdk_reply_t *out, char *line, size_t line_cap)
{
  yai_display_label_t dl;
  if (!line || line_cap == 0 || !out) return 0;
  line[0] = '\0';
  yai_display_from_command(
      (opts && opts->command_id) ? opts->command_id : out->command_id,
      (opts) ? opts->argc : 0,
      (opts) ? opts->argv : NULL,
      &dl);
  snprintf(line, line_cap, "%s %s", dl.scope, dl.action);
  return 1;
}

static void extract_reply_human_fields(
    const yai_sdk_reply_t *out,
    char *summary, size_t summary_cap,
    char *hint_1, size_t hint_1_cap,
    char *hint_2, size_t hint_2_cap)
{
  if (summary && summary_cap) summary[0] = '\0';
  if (hint_1 && hint_1_cap) hint_1[0] = '\0';
  if (hint_2 && hint_2_cap) hint_2[0] = '\0';
  if (!out || !out->exec_reply_json || !out->exec_reply_json[0]) return;

  cJSON *doc = cJSON_Parse(out->exec_reply_json);
  if (!doc) return;

  const cJSON *s = cJSON_GetObjectItemCaseSensitive(doc, "summary");
  if (summary && summary_cap && cJSON_IsString(s) && s->valuestring && s->valuestring[0]) {
    snprintf(summary, summary_cap, "%s", s->valuestring);
  }

  const cJSON *h = cJSON_GetObjectItemCaseSensitive(doc, "hints");
  if (!cJSON_IsArray(h)) h = cJSON_GetObjectItemCaseSensitive(doc, "hint");
  if (cJSON_IsArray(h)) {
    const cJSON *h0 = cJSON_GetArrayItem(h, 0);
    const cJSON *h1 = cJSON_GetArrayItem(h, 1);
    if (hint_1 && hint_1_cap && cJSON_IsString(h0) && h0->valuestring && h0->valuestring[0]) {
      snprintf(hint_1, hint_1_cap, "%s", h0->valuestring);
    }
    if (hint_2 && hint_2_cap && cJSON_IsString(h1) && h1->valuestring && h1->valuestring[0]) {
      snprintf(hint_2, hint_2_cap, "%s", h1->valuestring);
    }
  }

  cJSON_Delete(doc);
}

static int is_container_command(const yai_sdk_reply_t *out, const yai_render_opts_t *opts)
{
  const char *cid = NULL;
  if (!out) return 0;
  cid = (opts && opts->command_id && opts->command_id[0]) ? opts->command_id : out->command_id;
  if (!cid || !cid[0]) return 0;
  return strncmp(cid, "yai.container.", 14) == 0;
}

static int is_source_command(const yai_sdk_reply_t *out, const yai_render_opts_t *opts)
{
  const char *cid = NULL;
  if (!out) return 0;
  cid = (opts && opts->command_id && opts->command_id[0]) ? opts->command_id : out->command_id;
  if (!cid || !cid[0]) return 0;
  return strncmp(cid, "yai.source.", 11) == 0;
}

static int is_lifecycle_command(const yai_sdk_reply_t *out)
{
  if (!out || !out->command_id[0]) return 0;
  return strcmp(out->command_id, "yai.lifecycle.up") == 0 ||
         strcmp(out->command_id, "yai.lifecycle.down") == 0 ||
         strcmp(out->command_id, "yai.lifecycle.restart") == 0;
}

static const char *lifecycle_operation(const yai_sdk_reply_t *out)
{
  if (!out) return "unknown";
  if (strcmp(out->command_id, "yai.lifecycle.up") == 0) return "up";
  if (strcmp(out->command_id, "yai.lifecycle.down") == 0) return "down";
  if (strcmp(out->command_id, "yai.lifecycle.restart") == 0) return "restart";
  return "unknown";
}

static void print_title(FILE *stream, const char *title)
{
  if (!stream || !title) return;
  fprintf(stream, "%s\n", title);
  fprintf(stream, "-----------------\n");
}

static void print_section(FILE *stream, const char *name)
{
  if (!stream || !name) return;
  fprintf(stream, "\n%s\n", name);
}

static void print_kv(FILE *stream, const char *key, const char *value)
{
  if (!stream || !key) return;
  fprintf(stream, "  %-18s %s\n", key, (value && value[0]) ? value : "—");
}

static int starts_with(const char *s, const char *prefix)
{
  size_t n = 0;
  if (!s || !prefix) return 0;
  n = strlen(prefix);
  return strncmp(s, prefix, n) == 0;
}

static const cJSON *json_get_path(const cJSON *root, const char *path)
{
  char token[64];
  size_t ti = 0;
  const cJSON *cur = root;
  const char *p = path;
  if (!root || !path || !path[0]) return NULL;
  while (*p) {
    if (*p == '.') {
      token[ti] = '\0';
      cur = cJSON_GetObjectItemCaseSensitive((cJSON *)cur, token);
      if (!cur) return NULL;
      ti = 0;
      p++;
      continue;
    }
    if (ti + 1 < sizeof(token)) token[ti++] = *p;
    p++;
  }
  token[ti] = '\0';
  if (!token[0]) return cur;
  return cJSON_GetObjectItemCaseSensitive((cJSON *)cur, token);
}

static const char *json_string_path(const cJSON *root, const char *path, const char *fallback)
{
  const cJSON *n = json_get_path(root, path);
  if (cJSON_IsString(n) && n->valuestring && n->valuestring[0]) return n->valuestring;
  return fallback;
}

static const char *json_bool_path(const cJSON *root, const char *path, const char *true_label, const char *false_label)
{
  const cJSON *n = json_get_path(root, path);
  if (cJSON_IsBool(n)) return cJSON_IsTrue(n) ? true_label : false_label;
  return false_label;
}

static int json_int_path(const cJSON *root, const char *path, int fallback)
{
  const cJSON *n = json_get_path(root, path);
  if (cJSON_IsNumber(n)) return n->valueint;
  return fallback;
}

static const char *semantic_bool(int v, const char *yes, const char *no)
{
  return v ? yes : no;
}

static int json_bool_value_path(const cJSON *root, const char *path, int *out)
{
  const cJSON *n = json_get_path(root, path);
  if (!out) return 0;
  if (!cJSON_IsBool(n)) return 0;
  *out = cJSON_IsTrue(n) ? 1 : 0;
  return 1;
}

static const cJSON *resolve_runtime_caps(const cJSON *data)
{
  const cJSON *caps = NULL;
  if (!data || !cJSON_IsObject(data)) return NULL;
  caps = json_get_path(data, "runtime_capabilities");
  if (caps && cJSON_IsObject(caps)) return caps;
  caps = json_get_path(data, "runtime");
  if (caps && cJSON_IsObject(caps)) return data;
  return NULL;
}

static const char *semantic_capability_ready(int known, int ready)
{
  if (!known) return "unknown";
  return ready ? "ready" : "not ready";
}

static void print_runtime_capabilities(FILE *stream, const cJSON *data)
{
  const cJSON *caps = resolve_runtime_caps(data);
  int runtime_ready = 0, runtime_ready_known = 0;
  int selected = 0, selected_known = 0;
  int bound = 0, bound_known = 0;
  int data_ready = 0, data_known = 0;
  int graph_ready = 0, graph_known = 0;
  int cognition_ready = 0, cognition_known = 0;
  int exec_ready = 0, exec_known = 0;
  int recovery_tracked = 0, recovery_known = 0;
  const char *exec_state = "unknown";
  const char *recovery_state = "unknown";

  if (!stream || !caps) return;

  runtime_ready_known = json_bool_value_path(caps, "runtime.ready", &runtime_ready);
  selected_known =
    json_bool_value_path(caps, "container_binding.selected", &selected) ||
    json_bool_value_path(caps, "workspace_binding.selected", &selected);
  bound_known =
    json_bool_value_path(caps, "container_binding.bound", &bound) ||
    json_bool_value_path(caps, "workspace_binding.bound", &bound);
  data_known = json_bool_value_path(caps, "data.store_binding_ready", &data_ready);
  if (!data_known) data_known = json_bool_value_path(caps, "data.ready", &data_ready);
  graph_known = json_bool_value_path(caps, "graph.ready", &graph_ready);
  cognition_known = json_bool_value_path(caps, "cognition.ready", &cognition_ready);
  exec_known = json_bool_value_path(caps, "exec.ready", &exec_ready);
  recovery_known = json_bool_value_path(caps, "recovery.tracked", &recovery_tracked);
  exec_state = json_string_path(caps, "exec.state", "unknown");
  recovery_state = json_string_path(caps, "recovery.state", "unknown");

  print_section(stream, "Runtime capabilities");
  print_kv(stream, "Runtime baseline", semantic_capability_ready(runtime_ready_known, runtime_ready));
  print_kv(stream, "Container selected", selected_known ? semantic_bool(selected, "yes", "no") : "unknown");
  print_kv(stream, "Container bound", bound_known ? semantic_bool(bound, "yes", "no") : "unknown");
  print_kv(stream, "Data", semantic_capability_ready(data_known, data_ready));
  print_kv(stream, "Graph", semantic_capability_ready(graph_known, graph_ready));
  print_kv(stream, "Cognition", semantic_capability_ready(cognition_known, cognition_ready));
  print_kv(stream, "Exec", exec_known ? (exec_ready ? "ready" : exec_state) : exec_state);
  print_kv(stream, "Recovery tracked", recovery_known ? semantic_bool(recovery_tracked, "yes", "no") : "unknown");
  print_kv(stream, "Recovery state", recovery_state);
}

static int is_operator_runtime_surface_command(const yai_sdk_reply_t *out, const yai_render_opts_t *opts)
{
  const char *cid = NULL;
  if (!out) return 0;
  cid = (opts && opts->command_id && opts->command_id[0]) ? opts->command_id : out->command_id;
  if (!cid || !cid[0]) return 0;
  return (strcmp(cid, "yai.operator.inspect.runtime") == 0 ||
          strcmp(cid, "yai.operator.inspect.service") == 0 ||
          strcmp(cid, "yai.inspect.runtime") == 0 ||
          strcmp(cid, "yai.inspect.service") == 0 ||
          strcmp(cid, "yai.operator.doctor.runtime") == 0 ||
          strcmp(cid, "yai.operator.doctor.service") == 0 ||
          strcmp(cid, "yai.doctor.runtime") == 0 ||
          strcmp(cid, "yai.doctor.service") == 0 ||
          strcmp(cid, "yai.operator.verify.runtime") == 0 ||
          strcmp(cid, "yai.operator.verify.service") == 0 ||
          strcmp(cid, "yai.verify.runtime") == 0 ||
          strcmp(cid, "yai.verify.service") == 0);
}

static int detail_value_copy(const char *details, const char *key, char *out, size_t out_cap)
{
  const char *p;
  size_t key_len;
  size_t i = 0;
  if (!details || !key || !out || out_cap == 0) return 0;
  out[0] = '\0';
  key_len = strlen(key);
  p = strstr(details, key);
  if (!p) return 0;
  p += key_len;
  while (*p == ' ') p++;
  while (p[i] && p[i] != ' ' && i + 1 < out_cap) {
    out[i] = p[i];
    i++;
  }
  out[i] = '\0';
  return out[0] != '\0';
}

static void print_operator_runtime_surface(FILE *stream, const yai_sdk_reply_t *out)
{
  char liveness_buf[32];
  char readiness_buf[32];
  char selected_buf[16];
  char bound_buf[16];
  const char *liveness = "unknown";
  const char *readiness = "unknown";
  const char *container_selected = "unknown";
  const char *container_bound = "unknown";
  if (!stream || !out) return;
  if (strcmp(out->status, "ok") == 0 && strcmp(out->code, "OK") == 0) {
    liveness = "reachable";
    readiness = "ready";
  } else if (strcmp(out->code, "RUNTIME_NOT_READY") == 0) {
    liveness = "reachable";
    readiness = "not ready";
  } else if (strcmp(out->code, "SERVER_UNAVAILABLE") == 0) {
    liveness = "unreachable";
    readiness = "unavailable";
  }
  if (detail_value_copy(out->details, "liveness=", liveness_buf, sizeof(liveness_buf))) {
    liveness = liveness_buf;
  }
  if (detail_value_copy(out->details, "readiness=", readiness_buf, sizeof(readiness_buf))) {
    readiness = readiness_buf;
  }
  if (detail_value_copy(out->details, "container_selected=", selected_buf, sizeof(selected_buf))) {
    container_selected = selected_buf;
  }
  if (detail_value_copy(out->details, "container_bound=", bound_buf, sizeof(bound_buf))) {
    container_bound = bound_buf;
  }
  print_title(stream, "Runtime status");
  print_section(stream, "Baseline");
  print_kv(stream, "Liveness", liveness);
  print_kv(stream, "Readiness", readiness);
  print_kv(stream, "Target plane", out->target_plane[0] ? out->target_plane : "runtime");
  print_section(stream, "Container binding");
  print_kv(stream, "Container selected", container_selected);
  print_kv(stream, "Container bound", container_bound);
  print_kv(stream, "Hint", "Run: yai ws status");
  print_section(stream, "Capability families");
  print_kv(stream, "Exec", "unknown");
  print_kv(stream, "Data", "unknown");
  print_kv(stream, "Graph", "unknown");
  print_kv(stream, "Cognition", "unknown");
  print_kv(stream, "Recovery", "unknown");
}

static const char *prompt_icon(void)
{
  const char *ascii = getenv("YAI_WS_PROMPT_ASCII");
  const char *icon = getenv("YAI_WS_PROMPT_ICON");
  if (ascii && ascii[0] && strcmp(ascii, "0") != 0) return "o";
  if (icon && icon[0]) return icon;
  return "◉";
}

static void print_prompt_token(FILE *stream, const char *alias)
{
  if (!stream || !alias || !alias[0]) return;
  fprintf(stream, "%s %s\n", prompt_icon(), alias);
}

static const char *build_prompt_token(const char *alias, char *buf, size_t cap)
{
  if (!buf || cap == 0) return "none";
  if (!alias || !alias[0]) return "none";
  if (snprintf(buf, cap, "%s %s", prompt_icon(), alias) <= 0) return "none";
  return buf;
}

static void print_ws_current(FILE *stream, const cJSON *data)
{
  int runtime_attached = 0;
  const cJSON *rt = json_get_path(data, "runtime_attached");
  const char *binding = json_string_path(data, "binding_status", "none");
  const char *alias = json_string_path(data, "container_alias", "");
  char token[96];
  if (cJSON_IsBool(rt)) runtime_attached = cJSON_IsTrue(rt);
  print_title(stream, "Container current");
  print_section(stream, "Identity");
  print_kv(stream, "Id", json_string_path(data, "container_id", "—"));
  print_kv(stream, "Alias", json_string_path(data, "container_alias", "—"));
  print_kv(stream, "State", json_string_path(data, "state", "—"));
  print_kv(stream, "Root", json_string_path(data, "root_path", "—"));
  print_section(stream, "Session");
  print_kv(stream, "Binding", binding);
  print_kv(stream, "Runtime", semantic_bool(runtime_attached, "attached", "detached"));
  print_kv(stream, "Prompt token", (strcmp(binding, "active") == 0) ? build_prompt_token(alias, token, sizeof(token)) : "none");
  print_runtime_capabilities(stream, data);
}

static void print_ws_status(FILE *stream, const cJSON *data)
{
  int active = 0;
  int binding_valid = 0;
  int runtime_attached = 0;
  int debug_mode = 0;
  int declared_present = 0;
  int effective_present = 0;
  const cJSON *n = NULL;
  n = json_get_path(data, "active");
  if (cJSON_IsBool(n)) active = cJSON_IsTrue(n);
  n = json_get_path(data, "binding_valid");
  if (cJSON_IsBool(n)) binding_valid = cJSON_IsTrue(n);
  n = json_get_path(data, "runtime_attached");
  if (cJSON_IsBool(n)) runtime_attached = cJSON_IsTrue(n);
  n = json_get_path(data, "debug_mode");
  if (cJSON_IsBool(n)) debug_mode = cJSON_IsTrue(n);
  n = json_get_path(data, "declared_context_present");
  if (cJSON_IsBool(n)) declared_present = cJSON_IsTrue(n);
  n = json_get_path(data, "effective_context_present");
  if (cJSON_IsBool(n)) effective_present = cJSON_IsTrue(n);

  print_title(stream, "Container status");
  print_section(stream, "Binding");
  print_kv(stream, "Active", semantic_bool(active, "yes", "no"));
  print_kv(stream, "Binding", json_string_path(data, "binding_status", "none"));
  print_kv(stream, "Validity", semantic_bool(binding_valid, "valid", "invalid"));
  print_section(stream, "Runtime");
  print_kv(stream, "Runtime", semantic_bool(runtime_attached, "attached", "detached"));
  print_kv(stream, "Isolation", json_string_path(data, "isolation_mode", "—"));
  print_kv(stream, "Debug", semantic_bool(debug_mode, "on", "off"));
  print_kv(stream, "Execution mode", json_string_path(data, "execution_mode_effective", "scoped"));
  print_kv(stream, "Execution degraded", json_bool_path(data, "execution_mode_degraded", "yes", "no"));
  print_kv(stream, "Degraded reason", json_string_path(data, "execution_degraded_reason", "none"));
  print_section(stream, "Context");
  print_kv(stream, "Declared", semantic_bool(declared_present, "present", "unset"));
  print_kv(stream, "Effective", semantic_bool(effective_present, "resolved", "not resolved"));
  print_kv(stream, "Reason", json_string_path(data, "reason", "none"));
  print_runtime_capabilities(stream, data);
}

static void print_ws_domain(FILE *stream, const cJSON *data, const char *title)
{
  char conf_buf[32];
  const cJSON *conf = json_get_path(data, "inferred.confidence");
  conf_buf[0] = '\0';
  if (cJSON_IsNumber(conf)) snprintf(conf_buf, sizeof(conf_buf), "%.2f", conf->valuedouble);

  print_title(stream, title);
  print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
  print_section(stream, "Declared");
  print_kv(stream, "Family", json_string_path(data, "declared.family", "unset"));
  print_kv(stream, "Specialization", json_string_path(data, "declared.specialization", "unset"));
  print_kv(stream, "Source", json_string_path(data, "declared.source", "unset"));
  if (json_get_path(data, "inferred.family") || json_get_path(data, "inferred.specialization")) {
    print_section(stream, "Inferred");
    print_kv(stream, "Family", json_string_path(data, "inferred.family", "not resolved"));
    print_kv(stream, "Specialization", json_string_path(data, "inferred.specialization", "not resolved"));
    print_kv(stream, "Confidence", conf_buf[0] ? conf_buf : "—");
  }
  if (json_get_path(data, "effective.family") || json_get_path(data, "effective.specialization")) {
    print_section(stream, "Effective");
    print_kv(stream, "Family", json_string_path(data, "effective.family", "not resolved"));
    print_kv(stream, "Specialization", json_string_path(data, "effective.specialization", "not resolved"));
  }
}

static void print_ws_inspect(FILE *stream, const cJSON *data)
{
  char conf_buf[32];
  char attach_count_buf[32];
  const cJSON *conf = json_get_path(data, "normative.inferred.confidence");
  const cJSON *attach_count = json_get_path(data, "governance.policy_attachment_count");
  int runtime_attached = 0;
  int cp_attached = 0;
  int debug_mode = 0;
  const cJSON *n = NULL;
  conf_buf[0] = '\0';
  attach_count_buf[0] = '\0';
  if (cJSON_IsNumber(conf)) snprintf(conf_buf, sizeof(conf_buf), "%.2f", conf->valuedouble);
  if (cJSON_IsNumber(attach_count)) snprintf(attach_count_buf, sizeof(attach_count_buf), "%d", attach_count->valueint);
  n = json_get_path(data, "session.runtime_attached");
  if (cJSON_IsBool(n)) runtime_attached = cJSON_IsTrue(n);
  n = json_get_path(data, "session.control_plane_attached");
  if (cJSON_IsBool(n)) cp_attached = cJSON_IsTrue(n);
  n = json_get_path(data, "session.debug_mode");
  if (cJSON_IsBool(n)) debug_mode = cJSON_IsTrue(n);

  print_title(stream, "Container inspect");
  print_section(stream, "Identity");
  print_kv(stream, "Id", json_string_path(data, "identity.container_id", "—"));
  print_kv(stream, "Alias", json_string_path(data, "identity.container_alias", "—"));
  print_kv(stream, "State", json_string_path(data, "identity.state", "—"));
  print_kv(stream, "Root", json_string_path(data, "identity.root_path", "—"));
  print_section(stream, "Session");
  print_kv(stream, "Binding", json_string_path(data, "binding_status", "none"));
  print_kv(stream, "Runtime", semantic_bool(runtime_attached, "attached", "detached"));
  print_kv(stream, "Control plane", semantic_bool(cp_attached, "attached", "detached"));
  print_kv(stream, "Isolation", json_string_path(data, "session.isolation_mode", "—"));
  print_kv(stream, "Debug", semantic_bool(debug_mode, "on", "off"));
  print_kv(stream, "Boundary", json_string_path(data, "boundary.state", "unknown"));
  print_kv(stream, "Boundary reason", json_string_path(data, "boundary.reason", "none"));
  print_kv(stream, "Execution mode", json_string_path(data, "execution.mode_effective", "scoped"));
  print_kv(stream, "Execution degraded", json_bool_path(data, "execution.degraded", "yes", "no"));
  print_kv(stream, "Degraded reason", json_string_path(data, "execution.degraded_reason", "none"));
  print_runtime_capabilities(stream, data);
  print_section(stream, "Normative context");
  print_kv(stream, "Declared family", json_string_path(data, "normative.declared.family", "unset"));
  print_kv(stream, "Declared spec", json_string_path(data, "normative.declared.specialization", "unset"));
  print_kv(stream, "Inferred family", json_string_path(data, "normative.inferred.family", "not resolved"));
  print_kv(stream, "Inferred spec", json_string_path(data, "normative.inferred.specialization", "not resolved"));
  print_kv(stream, "Inference conf.", conf_buf[0] ? conf_buf : "—");
  print_section(stream, "Resolution");
  print_kv(stream, "Stack", json_string_path(data, "normative.effective.stack_ref", "not resolved"));
  print_kv(stream, "Overlays", json_string_path(data, "normative.effective.overlays_ref", "none"));
  print_kv(stream, "Effect", json_string_path(data, "normative.effective.effect_summary", "not resolved"));
  print_kv(stream, "Authority", json_string_path(data, "normative.effective.authority_summary", "not available"));
  print_kv(stream, "Evidence", json_string_path(data, "normative.effective.evidence_summary", "not available"));
  print_kv(stream, "Summary", json_string_path(data, "inspect.last_resolution_summary", "not available"));
  print_kv(stream, "Trace", json_string_path(data, "inspect.last_resolution_trace_ref", "not available"));
  print_section(stream, "Governance");
  print_kv(stream, "Attachments", json_string_path(data, "governance.policy_attachments", "none"));
  print_kv(stream, "Attachment count", attach_count_buf[0] ? attach_count_buf : "0");
  print_section(stream, "Read path");
  print_kv(stream, "Mode", json_string_path(data, "read_path.mode", "unknown"));
  print_kv(stream, "Primary source", json_string_path(data, "read_path.primary_source", "unknown"));
  print_kv(stream, "DB first ready", json_bool_path(data, "read_path.db_first_ready", "yes", "no"));
  print_kv(stream, "Fallback active", json_bool_path(data, "read_path.fallback_active", "yes", "no"));
  print_kv(stream, "Fallback reason", json_string_path(data, "read_path.fallback_reason", "none"));
  print_section(stream, "Scientific");
  print_kv(stream, "Experiment", json_string_path(data, "scientific.experiment_context_summary", "not scientific context"));
  print_kv(stream, "Parameters", json_string_path(data, "scientific.parameter_governance_summary", "not scientific context"));
  print_kv(stream, "Reproducibility", json_string_path(data, "scientific.reproducibility_summary", "not scientific context"));
  print_kv(stream, "Dataset integrity", json_string_path(data, "scientific.dataset_integrity_summary", "not scientific context"));
  print_kv(stream, "Publication", json_string_path(data, "scientific.publication_control_summary", "not scientific context"));
  print_section(stream, "Digital");
  print_kv(stream, "Outbound", json_string_path(data, "digital.outbound_context_summary", "not digital context"));
  print_kv(stream, "Sink target", json_string_path(data, "digital.sink_target_summary", "not digital context"));
  print_kv(stream, "Publication", json_string_path(data, "digital.publication_control_summary", "not digital context"));
  print_kv(stream, "Retrieval", json_string_path(data, "digital.retrieval_control_summary", "not digital context"));
  print_kv(stream, "Distribution", json_string_path(data, "digital.distribution_control_summary", "not digital context"));
}

static void print_ws_policy_effective(FILE *stream, const cJSON *data)
{
  char attach_count_buf[32];
  const cJSON *attach_count = json_get_path(data, "policy_attachment_count");
  attach_count_buf[0] = '\0';
  if (cJSON_IsNumber(attach_count)) snprintf(attach_count_buf, sizeof(attach_count_buf), "%d", attach_count->valueint);
  print_title(stream, "Container policy effective");
  print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
  print_section(stream, "Effective stack");
  print_kv(stream, "Family", json_string_path(data, "family_effective", "not resolved"));
  print_kv(stream, "Specialization", json_string_path(data, "specialization_effective", "not resolved"));
  print_kv(stream, "Stack", json_string_path(data, "effective_stack_ref", "not resolved"));
  print_kv(stream, "Overlays", json_string_path(data, "effective_overlays_ref", "none"));
  print_kv(stream, "Precedence", json_string_path(data, "precedence", "not available"));
  print_section(stream, "Outcome");
  print_kv(stream, "Effect", json_string_path(data, "effect_summary", "not resolved"));
  print_kv(stream, "Authority", json_string_path(data, "authority_summary", "not available"));
  print_kv(stream, "Evidence", json_string_path(data, "evidence_summary", "not available"));
  print_section(stream, "Governance");
  print_kv(stream, "Attachments", json_string_path(data, "policy_attachments", "none"));
  print_kv(stream, "Attachment count", attach_count_buf[0] ? attach_count_buf : "0");
  print_section(stream, "Scientific");
  print_kv(stream, "Experiment", json_string_path(data, "scientific.experiment_context_summary", "not scientific context"));
  print_kv(stream, "Parameters", json_string_path(data, "scientific.parameter_governance_summary", "not scientific context"));
  print_kv(stream, "Reproducibility", json_string_path(data, "scientific.reproducibility_summary", "not scientific context"));
  print_kv(stream, "Dataset integrity", json_string_path(data, "scientific.dataset_integrity_summary", "not scientific context"));
  print_kv(stream, "Publication", json_string_path(data, "scientific.publication_control_summary", "not scientific context"));
  print_section(stream, "Digital");
  print_kv(stream, "Outbound", json_string_path(data, "digital.outbound_context_summary", "not digital context"));
  print_kv(stream, "Sink target", json_string_path(data, "digital.sink_target_summary", "not digital context"));
  print_kv(stream, "Publication", json_string_path(data, "digital.publication_control_summary", "not digital context"));
  print_kv(stream, "Retrieval", json_string_path(data, "digital.retrieval_control_summary", "not digital context"));
  print_kv(stream, "Distribution", json_string_path(data, "digital.distribution_control_summary", "not digital context"));
}

static void print_ws_debug_resolution(FILE *stream, const cJSON *data)
{
  char conf_buf[32];
  const cJSON *conf = json_get_path(data, "inferred.confidence");
  conf_buf[0] = '\0';
  if (cJSON_IsNumber(conf)) snprintf(conf_buf, sizeof(conf_buf), "%.2f", conf->valuedouble);

  print_title(stream, "Container debug resolution");
  print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
  print_section(stream, "Context sources");
  print_kv(stream, "Source", json_string_path(data, "context_source", "unknown"));
  print_kv(stream, "Declared family", json_string_path(data, "declared.family", "unset"));
  print_kv(stream, "Declared spec", json_string_path(data, "declared.specialization", "unset"));
  print_kv(stream, "Inferred family", json_string_path(data, "inferred.family", "not resolved"));
  print_kv(stream, "Inferred spec", json_string_path(data, "inferred.specialization", "not resolved"));
  print_kv(stream, "Inference conf.", conf_buf[0] ? conf_buf : "—");
  print_section(stream, "Resolution");
  print_kv(stream, "Stack", json_string_path(data, "effective.stack_ref", "not resolved"));
  print_kv(stream, "Overlays", json_string_path(data, "effective.overlays_ref", "none"));
  print_kv(stream, "Precedence", json_string_path(data, "precedence_outcome", "not available"));
  print_kv(stream, "Effect", json_string_path(data, "effect_outcome", "not resolved"));
  print_kv(stream, "Summary", json_string_path(data, "last_resolution_summary", "not available"));
  print_kv(stream, "Trace", json_string_path(data, "last_resolution_trace_ref", "not available"));
  print_section(stream, "Scientific");
  print_kv(stream, "Experiment", json_string_path(data, "scientific.experiment_context_summary", "not scientific context"));
  print_kv(stream, "Parameters", json_string_path(data, "scientific.parameter_governance_summary", "not scientific context"));
  print_kv(stream, "Reproducibility", json_string_path(data, "scientific.reproducibility_summary", "not scientific context"));
  print_kv(stream, "Dataset integrity", json_string_path(data, "scientific.dataset_integrity_summary", "not scientific context"));
  print_kv(stream, "Publication", json_string_path(data, "scientific.publication_control_summary", "not scientific context"));
  print_section(stream, "Digital");
  print_kv(stream, "Outbound", json_string_path(data, "digital.outbound_context_summary", "not digital context"));
  print_kv(stream, "Sink target", json_string_path(data, "digital.sink_target_summary", "not digital context"));
  print_kv(stream, "Publication", json_string_path(data, "digital.publication_control_summary", "not digital context"));
  print_kv(stream, "Retrieval", json_string_path(data, "digital.retrieval_control_summary", "not digital context"));
  print_kv(stream, "Distribution", json_string_path(data, "digital.distribution_control_summary", "not digital context"));
}

static void print_ws_run(FILE *stream, const yai_render_opts_t *opts, const cJSON *data)
{
  const char *action = (opts && opts->argc > 0 && opts->argv && opts->argv[0]) ? opts->argv[0] : "unknown";
  const char *effect = json_string_path(data, "decision.effect",
                        json_string_path(data, "resolution_trace.final_effect", "not available"));
  print_title(stream, "Container run");
  print_section(stream, "Execution");
  print_kv(stream, "Action", action);
  print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
  print_kv(stream, "Family", json_string_path(data, "decision.family_id",
                        json_string_path(data, "resolution_trace.matched_family", "not resolved")));
  print_kv(stream, "Specialization", json_string_path(data, "decision.specialization_id",
                        json_string_path(data, "resolution_trace.matched_specialization", "not resolved")));
  print_section(stream, "Outcome");
  print_kv(stream, "Effect", effect);
  print_kv(stream, "Rationale", json_string_path(data, "decision.rationale", "not available"));
  print_kv(stream, "Authority", json_string_path(data, "resolution_trace.authority_profile", "not available"));
  print_kv(stream, "Evidence", json_string_path(data, "resolution_trace.evidence_profile", "not available"));
  print_kv(stream, "Trace", json_string_path(data, "evidence.trace_id", "not available"));
  print_section(stream, "Scientific");
  print_kv(stream, "Parameters", json_string_path(data, "scientific.parameter_governance_summary", "not scientific context"));
  print_kv(stream, "Reproducibility", json_string_path(data, "scientific.reproducibility_summary", "not scientific context"));
  print_kv(stream, "Dataset integrity", json_string_path(data, "scientific.dataset_integrity_summary", "not scientific context"));
  print_kv(stream, "Publication", json_string_path(data, "scientific.publication_control_summary", "not scientific context"));
  print_section(stream, "Digital");
  print_kv(stream, "Outbound", json_string_path(data, "digital.outbound_context_summary", "not digital context"));
  print_kv(stream, "Sink target", json_string_path(data, "digital.sink_target_summary", "not digital context"));
  print_kv(stream, "Publication", json_string_path(data, "digital.publication_control_summary", "not digital context"));
  print_kv(stream, "Retrieval", json_string_path(data, "digital.retrieval_control_summary", "not digital context"));
  print_kv(stream, "Distribution", json_string_path(data, "digital.distribution_control_summary", "not digital context"));
  print_kv(stream, "Next", "yai ws policy effective");
}

static void print_json_array_preview(FILE *stream, const char *label, const cJSON *arr)
{
  char count_buf[32];
  char preview[256];
  char *row = NULL;
  int n = 0;
  const cJSON *first = NULL;
  if (!stream || !label || !cJSON_IsArray(arr)) return;
  n = cJSON_GetArraySize((cJSON *)arr);
  snprintf(count_buf, sizeof(count_buf), "%d", n);
  print_kv(stream, label, count_buf);
  if (n <= 0) return;
  first = cJSON_GetArrayItem((cJSON *)arr, 0);
  if (!first) return;
  row = cJSON_PrintUnformatted((cJSON *)first);
  if (!row) return;
  snprintf(preview, sizeof(preview), "%-.220s", row);
  print_kv(stream, "Sample", preview);
  free(row);
}

static void print_object_fields(FILE *stream, const cJSON *obj, int max_fields)
{
  const cJSON *it = NULL;
  int emitted = 0;
  if (!stream || !cJSON_IsObject(obj)) return;
  for (it = obj->child; it; it = it->next) {
    char v[256];
    if (!it->string || !it->string[0]) continue;
    if (max_fields > 0 && emitted >= max_fields) {
      print_kv(stream, "More", "…");
      break;
    }
    if (cJSON_IsString(it) && it->valuestring) snprintf(v, sizeof(v), "%s", it->valuestring);
    else if (cJSON_IsBool(it)) snprintf(v, sizeof(v), "%s", cJSON_IsTrue(it) ? "yes" : "no");
    else if (cJSON_IsNumber(it)) {
      if (it->valuedouble == (double)it->valueint) snprintf(v, sizeof(v), "%d", it->valueint);
      else snprintf(v, sizeof(v), "%.3f", it->valuedouble);
    } else if (cJSON_IsArray(it)) {
      snprintf(v, sizeof(v), "%d items", cJSON_GetArraySize((cJSON *)it));
    } else if (cJSON_IsObject(it)) {
      snprintf(v, sizeof(v), "object");
    } else {
      snprintf(v, sizeof(v), "value");
    }
    print_kv(stream, it->string, v);
    emitted++;
  }
}

static void print_ws_query_result(FILE *stream, const cJSON *data, const char *title)
{
  const cJSON *summary = NULL;
  const cJSON *record = NULL;
  const cJSON *graph_summary = NULL;
  const cJSON *read_path = NULL;
  const cJSON *stores = NULL;
  const cJSON *columns = NULL;
  const cJSON *rows = NULL;
  const cJSON *items = NULL;
  if (!stream || !data) return;

  print_title(stream, title ? title : "Container query");
  print_section(stream, "Query");
  print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
  print_kv(stream, "Family", json_string_path(data, "query_family", "unknown"));
  print_kv(stream, "Shape", json_string_path(data, "result_shape", "unknown"));

  summary = json_get_path(data, "summary");
  if (cJSON_IsObject(summary)) {
    print_section(stream, "Summary");
    print_object_fields(stream, summary, 10);
  }
  graph_summary = json_get_path(data, "graph_query_summary");
  if (cJSON_IsObject(graph_summary)) {
    print_section(stream, "Graph summary");
    print_object_fields(stream, graph_summary, 10);
  }
  record = json_get_path(data, "record");
  if (cJSON_IsObject(record)) {
    print_section(stream, "Record");
    print_object_fields(stream, record, 12);
  }

  columns = json_get_path(data, "columns");
  rows = json_get_path(data, "rows");
  if (cJSON_IsArray(columns) || cJSON_IsArray(rows)) {
    char cols_buf[32];
    print_section(stream, "Table");
    if (cJSON_IsArray(columns)) {
      snprintf(cols_buf, sizeof(cols_buf), "%d", cJSON_GetArraySize((cJSON *)columns));
      print_kv(stream, "Columns", cols_buf);
    }
    if (cJSON_IsArray(rows)) {
      print_json_array_preview(stream, "Rows", rows);
    }
  }

  items = json_get_path(data, "items");
  if (cJSON_IsArray(items)) {
    print_section(stream, "Items");
    print_json_array_preview(stream, "Count", items);
  }

  read_path = json_get_path(data, "read_path");
  if (cJSON_IsObject(read_path)) {
    print_section(stream, "Read path");
    print_kv(stream, "Mode", json_string_path(data, "read_path.mode", "unknown"));
    print_kv(stream, "Primary source", json_string_path(data, "read_path.primary_source", "unknown"));
    print_kv(stream, "DB first ready", json_bool_path(data, "read_path.db_first_ready", "yes", "no"));
    print_kv(stream, "Fallback active", json_bool_path(data, "read_path.fallback_active", "yes", "no"));
    print_kv(stream, "Fallback reason", json_string_path(data, "read_path.fallback_reason", "none"));
  }

  stores = json_get_path(data, "stores");
  if (cJSON_IsObject(stores)) {
    print_section(stream, "Stores");
    print_object_fields(stream, stores, 6);
  }
}

static void print_ws_cognition_composed(FILE *stream, const cJSON *data)
{
  if (!stream || !data) return;
  print_title(stream, "Container cognition");
  print_section(stream, "Context");
  print_kv(stream, "Container", json_string_path(data, "identity.container_id", "—"));
  print_kv(stream, "Declared family", json_string_path(data, "context.declared.family", "unset"));
  print_kv(stream, "Declared spec", json_string_path(data, "context.declared.specialization", "unset"));
  print_section(stream, "Cognition");
  print_kv(stream, "Ready", json_bool_path(data, "runtime_capabilities.cognition.ready", "yes", "no"));
  print_kv(stream, "Transient authoritative", json_bool_path(data, "runtime_capabilities.cognition.transient_authoritative", "yes", "no"));
  print_kv(stream, "Last resolution", json_string_path(data, "read_path.last_resolution_summary", "not available"));
  print_kv(stream, "Trace", json_string_path(data, "read_path.last_resolution_trace_ref", "not available"));
  print_section(stream, "Read path");
  print_kv(stream, "Mode", json_string_path(data, "read_path.mode", "unknown"));
  print_kv(stream, "Primary source", json_string_path(data, "read_path.primary_source", "unknown"));
  print_kv(stream, "DB first ready", json_bool_path(data, "read_path.db_first_ready", "yes", "no"));
  print_kv(stream, "Fallback active", json_bool_path(data, "read_path.fallback_active", "yes", "no"));
  print_kv(stream, "Fallback reason", json_string_path(data, "read_path.fallback_reason", "none"));
}

static int render_source_data_human(const yai_sdk_reply_t *out, const yai_render_opts_t *opts, FILE *stream)
{
  cJSON *doc = NULL;
  cJSON *data = NULL;
  cJSON *summary = NULL;
  const char *cid = NULL;
  char num[32];
  if (!out || !stream || !out->exec_reply_json || !out->exec_reply_json[0]) return 0;
  cid = (opts && opts->command_id && opts->command_id[0]) ? opts->command_id : out->command_id;
  doc = cJSON_Parse(out->exec_reply_json);
  if (!doc) return 0;
  data = cJSON_GetObjectItemCaseSensitive(doc, "data");
  if (!data || !cJSON_IsObject(data)) {
    cJSON_Delete(doc);
    return 0;
  }

  if (cid && strcmp(cid, "yai.source.enroll") == 0) {
    print_title(stream, "Source enroll");
    print_section(stream, "Identity");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Source node", json_string_path(data, "source_node_id", "—"));
    print_kv(stream, "Daemon instance", json_string_path(data, "daemon_instance_id", "—"));
    print_kv(stream, "Owner link", json_string_path(data, "owner_link_id", "—"));
    print_kv(stream, "Registered", json_bool_path(data, "registered", "yes", "no"));
    print_section(stream, "Mediation");
    print_kv(stream, "Layer", json_string_path(data, "mediation.layer", "exec"));
    print_kv(stream, "Route", json_string_path(data, "mediation.route", "owner_ingest_v1"));
    print_kv(stream, "Transport ready", json_bool_path(data, "mediation.transport_ready", "yes", "no"));
    print_kv(stream, "Owner canonical", json_bool_path(data, "mediation.owner_canonical", "yes", "no"));
    cJSON_Delete(doc);
    return 1;
  }

  if (cid && strcmp(cid, "yai.source.attach") == 0) {
    print_title(stream, "Source attach");
    print_section(stream, "Binding");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Owner container", json_string_path(data, "owner_container_id", "—"));
    print_kv(stream, "Source node", json_string_path(data, "source_node_id", "—"));
    print_kv(stream, "Source binding", json_string_path(data, "source_binding_id", "—"));
    print_kv(stream, "Attachment", json_string_path(data, "attachment_status", "unknown"));
    print_section(stream, "Mediation");
    print_kv(stream, "Layer", json_string_path(data, "mediation.layer", "exec"));
    print_kv(stream, "Route", json_string_path(data, "mediation.route", "owner_ingest_v1"));
    print_kv(stream, "Transport ready", json_bool_path(data, "mediation.transport_ready", "yes", "no"));
    print_kv(stream, "Owner canonical", json_bool_path(data, "mediation.owner_canonical", "yes", "no"));
    cJSON_Delete(doc);
    return 1;
  }

  if (cid && (strcmp(cid, "yai.source.list") == 0 ||
              strcmp(cid, "yai.source.status") == 0 ||
              strcmp(cid, "yai.source.inspect") == 0)) {
    summary = cJSON_GetObjectItemCaseSensitive(data, "summary");
    print_title(stream, strcmp(cid, "yai.source.list") == 0 ? "Source list" :
                        (strcmp(cid, "yai.source.inspect") == 0 ? "Source inspect" : "Source status"));
    print_section(stream, "Identity");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Query family", json_string_path(data, "query_family", "source"));
    print_kv(stream, "Result shape", json_string_path(data, "result_shape", "summary"));

    print_section(stream, "Topology");
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_node_count", 0));
    print_kv(stream, "Source nodes", num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_daemon_instance_count", 0));
    print_kv(stream, "Daemon instances", num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_binding_count", 0));
    print_kv(stream, "Source bindings", num);

    print_section(stream, "Flow");
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_asset_count", 0));
    print_kv(stream, "Source assets", num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_acquisition_event_count", 0));
    print_kv(stream, "Acquisition events", num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_evidence_candidate_count", 0));
    print_kv(stream, "Evidence candidates", num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_owner_link_count", 0));
    print_kv(stream, "Owner links", num);

    print_section(stream, "Health and backlog");
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_spool_backlog_count",
                                                  json_int_path(summary, "spool_backlog_count", -1)));
    print_kv(stream, "Spool backlog", (strcmp(num, "-1") == 0) ? "not available" : num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_retry_due_count",
                                                  json_int_path(summary, "retry_due_count", -1)));
    print_kv(stream, "Retry due", (strcmp(num, "-1") == 0) ? "not available" : num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_disconnected_count",
                                                  json_int_path(summary, "disconnected_count", -1)));
    print_kv(stream, "Disconnected", (strcmp(num, "-1") == 0) ? "not available" : num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_degraded_count",
                                                  json_int_path(summary, "degraded_count", -1)));
    print_kv(stream, "Degraded", (strcmp(num, "-1") == 0) ? "not available" : num);

    print_section(stream, "Graph");
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_graph_node_count", 0));
    print_kv(stream, "Graph nodes", num);
    snprintf(num, sizeof(num), "%d", json_int_path(summary, "source_graph_edge_count", 0));
    print_kv(stream, "Graph edges", num);
    print_section(stream, "Read path");
    print_kv(stream, "Mode", json_string_path(data, "read_path.mode", "unknown"));
    print_kv(stream, "Primary source", json_string_path(data, "read_path.primary_source", "unknown"));
    print_kv(stream, "DB first ready", json_bool_path(data, "read_path.db_first_ready", "yes", "no"));
    print_kv(stream, "Fallback active", json_bool_path(data, "read_path.fallback_active", "yes", "no"));
    print_kv(stream, "Fallback reason", json_string_path(data, "read_path.fallback_reason", "none"));
    cJSON_Delete(doc);
    return 1;
  }

  if (cid && strcmp(cid, "yai.source.retry_drain") == 0) {
    print_title(stream, "Source retry-drain");
    print_kv(stream, "Summary", json_string_path(data, "summary", "not implemented"));
    cJSON_Delete(doc);
    return 1;
  }

  cJSON_Delete(doc);
  return 0;
}

static int render_container_data_human(const yai_sdk_reply_t *out, const yai_render_opts_t *opts, FILE *stream)
{
  cJSON *doc = NULL;
  cJSON *data = NULL;
  const char *cid = NULL;
  const cJSON *binding_status = NULL;
  const cJSON *container_alias = NULL;
  int printed = 0;
  if (!out || !stream || !out->exec_reply_json || !out->exec_reply_json[0]) return 0;
  cid = (opts && opts->command_id && opts->command_id[0]) ? opts->command_id : out->command_id;
  doc = cJSON_Parse(out->exec_reply_json);
  if (!doc) return 0;
  data = cJSON_GetObjectItemCaseSensitive(doc, "data");
  if (!data || !cJSON_IsObject(data)) {
    cJSON_Delete(doc);
    return 0;
  }

  if (cid && strcmp(cid, "yai.container.prompt_context") == 0) {
    binding_status = cJSON_GetObjectItemCaseSensitive(data, "binding_status");
    container_alias = cJSON_GetObjectItemCaseSensitive(data, "container_alias");
    if (cJSON_IsString(binding_status) && binding_status->valuestring &&
        strcmp(binding_status->valuestring, "active") == 0 &&
        cJSON_IsString(container_alias) && container_alias->valuestring && container_alias->valuestring[0]) {
      print_prompt_token(stream, container_alias->valuestring);
      printed = 1;
    }
    cJSON_Delete(doc);
    return printed;
  }

  if (cid && strcmp(cid, "yai.container.current") == 0) print_ws_current(stream, data);
  else if (cid && strcmp(cid, "yai.container.status") == 0) print_ws_status(stream, data);
  else if (cid && strcmp(cid, "yai.container.domain_get") == 0) print_ws_domain(stream, data, "Container domain");
  else if (cid && strcmp(cid, "yai.container.domain_set") == 0) print_ws_domain(stream, data, "Container domain updated");
  else if (cid && strcmp(cid, "yai.container.inspect") == 0) print_ws_inspect(stream, data);
  else if (cid && starts_with(cid, "yai.container.cognition.")) print_ws_cognition_composed(stream, data);
  else if (cid && strcmp(cid, "yai.container.policy_effective") == 0) print_ws_policy_effective(stream, data);
  else if (cid && strcmp(cid, "yai.container.policy_dry_run") == 0) {
    print_title(stream, "Container policy dry-run");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Object", json_string_path(data, "object_id", "—"));
    print_kv(stream, "Eligibility", json_string_path(data, "eligibility_result", "unknown"));
    print_kv(stream, "Compatibility", json_string_path(data, "compatibility_result", "unknown"));
    print_kv(stream, "Conflicts", json_string_path(data, "conflict_summary", "none"));
    print_kv(stream, "Warnings", json_string_path(data, "warnings", "none"));
    print_kv(stream, "Readiness", json_bool_path(data, "ready_for_attach", "true", "false"));
    print_kv(stream, "Precedence", json_string_path(data, "precedence_class", "—"));
  }
  else if (cid && strcmp(cid, "yai.container.policy_attach") == 0) {
    print_title(stream, "Container policy attach");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Object", json_string_path(data, "object_id", "—"));
    print_kv(stream, "Validity", json_string_path(data, "attachment_valid", "true"));
    print_kv(stream, "Eligibility", json_string_path(data, "eligibility_result", "eligible"));
    print_kv(stream, "Compatibility", json_string_path(data, "compatibility_result", "dry_run_passed"));
    print_kv(stream, "Conflicts", json_string_path(data, "conflict_summary", "none"));
    print_kv(stream, "State", json_string_path(data, "attachment_state", "attached_active"));
    print_kv(stream, "Activation", json_string_path(data, "activation_state", "active"));
    print_kv(stream, "Attachments", json_string_path(data, "policy_attachments", "none"));
  }
  else if (cid && strcmp(cid, "yai.container.policy_activate") == 0) {
    print_title(stream, "Container policy activate");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Object", json_string_path(data, "object_id", "—"));
    print_kv(stream, "State", json_string_path(data, "attachment_state", "attached_active"));
    print_kv(stream, "Activation", json_string_path(data, "activation_state", "active"));
    print_kv(stream, "Attachments", json_string_path(data, "policy_attachments", "none"));
  }
  else if (cid && strcmp(cid, "yai.container.policy_detach") == 0) {
    print_title(stream, "Container policy detach");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Object", json_string_path(data, "object_id", "—"));
    print_kv(stream, "Validity", json_string_path(data, "attachment_valid", "true"));
    print_kv(stream, "State", json_string_path(data, "attachment_state", "detached"));
    print_kv(stream, "Attachments", json_string_path(data, "policy_attachments", "none"));
  }
  else if (cid && strcmp(cid, "yai.container.debug_resolution") == 0) print_ws_debug_resolution(stream, data);
  else if (cid && strcmp(cid, "yai.container.run") == 0) print_ws_run(stream, opts, data);
  else if ((cid && starts_with(cid, "yai.container.graph.")) ||
           (cid && starts_with(cid, "yai.container.db.")) ||
           (cid && starts_with(cid, "yai.container.data.")) ||
           (cid && starts_with(cid, "yai.container.cognition.")) ||
           (cid && strcmp(cid, "yai.container.query") == 0) ||
           (cid && strcmp(cid, "yai.container.events.tail") == 0) ||
           (cJSON_IsString(cJSON_GetObjectItemCaseSensitive(data, "type")) &&
            strcmp(cJSON_GetObjectItemCaseSensitive(data, "type")->valuestring, "yai.container.query.result.v1") == 0)) {
    const char *title = "Container query";
    if (cid && starts_with(cid, "yai.container.graph.")) title = "Container graph";
    else if (cid && starts_with(cid, "yai.container.db.")) title = "Container db";
    else if (cid && starts_with(cid, "yai.container.data.")) title = "Container data";
    else if (cid && starts_with(cid, "yai.container.cognition.")) title = "Container cognition";
    else if (cid && strcmp(cid, "yai.container.events.tail") == 0) title = "Container data";
    else if (cid && strcmp(cid, "yai.container.query") == 0) {
      const char *family = json_string_path(data, "query_family", "");
      if (starts_with(family, "graph")) title = "Container graph";
      else if (starts_with(family, "transient") || starts_with(family, "memory")) title = "Container cognition";
      else if (starts_with(family, "events") || starts_with(family, "evidence") || starts_with(family, "governance") ||
               starts_with(family, "authority") || starts_with(family, "artifact") || starts_with(family, "enforcement")) title = "Container data";
      else if (starts_with(family, "container")) title = "Container db";
    }
    print_ws_query_result(stream, data, title);
  }
  else if (cid && (strcmp(cid, "yai.container.set") == 0 || strcmp(cid, "yai.container.switch") == 0)) {
    print_title(stream, strcmp(cid, "yai.container.switch") == 0 ? "Container switched" : "Container set");
    print_kv(stream, "Container", json_string_path(data, "container_id", "—"));
    print_kv(stream, "Binding", json_string_path(data, "binding_status", "active"));
    print_kv(stream, "Execution", json_string_path(data, "execution_mode_effective", "scoped"));
    print_kv(stream, "Prompt token", json_string_path(data, "container_alias", "none"));
  }
  else if (cid && strcmp(cid, "yai.container.unset") == 0) {
    print_title(stream, "Container unset");
    print_kv(stream, "Binding", json_string_path(data, "binding_status", "no_active"));
    print_kv(stream, "Prompt token", "none");
  }
  else if (cid && strcmp(cid, "yai.container.clear") == 0) {
    print_title(stream, "Container cleared");
    print_kv(stream, "Binding", json_string_path(data, "binding_status", "active"));
    print_kv(stream, "Runtime state", json_string_path(data, "runtime_state", "cleared"));
    print_kv(stream, "Prompt token", "unchanged");
  }
  else {
    print_title(stream, "Container command");
    print_kv(stream, "Summary", json_string_path(data, "summary", "ok"));
  }
  printed = 1;
  cJSON_Delete(doc);
  return printed;
}

int yai_render_exec_short(const yai_sdk_reply_t *out, int rc, const yai_render_opts_t *opts)
{
  char subject[256];
  char reply_summary[512];
  char reply_hint_1[256];
  char reply_hint_2[256];
  int container_rendered = 0;
  int container_command = 0;
  int source_command = 0;
  int lifecycle_command = 0;
  int operator_runtime_surface = 0;
  const char *effective_detail = NULL;
  const char *effective_hint_1 = NULL;
  const char *effective_hint_2 = NULL;
  yai_display_result_t mapped;
  int use_color;
  yai_style_role_t status_role = YAI_STYLE_INFO;
  FILE *stream = (rc == 0) ? stdout : stderr;
  if (!out || !out->status[0] || !out->code[0] || !out->reason[0]) return 0;
  if (!build_subject(opts, out, subject, sizeof(subject))) return 0;
  yai_display_from_reply(out, &mapped);
  extract_reply_human_fields(out,
                             reply_summary, sizeof(reply_summary),
                             reply_hint_1, sizeof(reply_hint_1),
                             reply_hint_2, sizeof(reply_hint_2));
  container_command = is_container_command(out, opts);
  source_command = is_source_command(out, opts);
  lifecycle_command = is_lifecycle_command(out);
  operator_runtime_surface = is_operator_runtime_surface_command(out, opts);
  effective_detail = reply_summary[0] ? reply_summary : mapped.detail;
  effective_hint_1 = reply_hint_1[0] ? reply_hint_1 : mapped.hint;
  effective_hint_2 = reply_hint_2[0] ? reply_hint_2 : NULL;

  use_color = (opts && opts->use_color) ? 1 : 0;
  if (strcmp(out->status, "ok") == 0) status_role = YAI_STYLE_OK;
  else if (strcmp(out->status, "nyi") == 0 || strcmp(out->code, "RUNTIME_NOT_READY") == 0) status_role = YAI_STYLE_WARN;
  else status_role = YAI_STYLE_ERR;

  if (!container_command && !source_command && !operator_runtime_surface) {
    print_line(stream, subject, use_color, yai_style_color(YAI_STYLE_EMPH));
  }
  if (lifecycle_command) {
    const char *liveness = "unknown";
    const char *readiness = "unknown";
    print_title(stream, "Runtime lifecycle");
    print_kv(stream, "Operation", lifecycle_operation(out));
    if (strcmp(out->command_id, "yai.lifecycle.up") == 0) {
      if (strcmp(out->status, "ok") == 0) {
        liveness = "reachable";
        readiness = "ready (control plane reachable)";
      } else if (strcmp(out->code, "RUNTIME_NOT_READY") == 0) {
        liveness = "reachable";
        readiness = "not ready (startup converging)";
      } else if (strcmp(out->code, "SERVER_UNAVAILABLE") == 0) {
        liveness = "unreachable";
        readiness = "unavailable";
      }
      print_kv(stream, "Runtime liveness", liveness);
      print_kv(stream, "Runtime readiness", readiness);
      print_kv(stream, "Container binding", "unknown (run: yai ws status)");
    } else if (strcmp(out->command_id, "yai.lifecycle.down") == 0) {
      print_kv(stream, "Runtime liveness", strcmp(out->status, "ok") == 0 ? "stopped" : "unknown");
    } else if (strcmp(out->command_id, "yai.lifecycle.restart") == 0) {
      print_kv(stream, "Runtime liveness", strcmp(out->status, "ok") == 0 ? "reachable" : "unknown");
      print_kv(stream, "Runtime readiness", strcmp(out->status, "ok") == 0 ? "ready" : "unknown");
    }
    if (effective_detail && effective_detail[0]) print_kv(stream, "Summary", effective_detail);
    if (strcmp(out->command_id, "yai.lifecycle.up") == 0) print_kv(stream, "Next", "yai doctor runtime");
    container_rendered = 1;
  } else if (container_command && (strcmp(out->status, "ok") == 0 || strcmp(out->code, "POLICY_BLOCK") == 0)) {
    container_rendered = render_container_data_human(out, opts, stream);
  } else if (source_command && (strcmp(out->status, "ok") == 0 || strcmp(out->status, "nyi") == 0)) {
    container_rendered = render_source_data_human(out, opts, stream);
  } else if (operator_runtime_surface) {
    print_operator_runtime_surface(stream, out);
    container_rendered = 1;
  } else {
    print_line(stream, mapped.status_label, use_color, yai_style_color(status_role));
  }
  if (!container_rendered && effective_detail && effective_detail[0]) fprintf(stream, "%s\n", effective_detail);
  if (out->trace_id[0] &&
      ((opts && opts->show_trace) || strcmp(out->status, "ok") != 0) &&
      !((container_command || source_command) && container_rendered)) {
    yai_color_print(stream, use_color, yai_style_color(YAI_STYLE_MUTED), "Trace: ");
    fprintf(stream, "%s\n", out->trace_id);
  }
  if (!(opts && opts->quiet) &&
      effective_hint_1 && effective_hint_1[0] &&
      strcmp(out->status, "ok") != 0 &&
      !((container_command || source_command) && container_rendered)) {
    fprintf(stream, "Hint: %s\n", effective_hint_1);
    if (effective_hint_2 && effective_hint_2[0]) fprintf(stream, "Hint: %s\n", effective_hint_2);
  }
  return 1;
}

int yai_render_exec_verbose(const yai_sdk_reply_t *out, int rc, const yai_render_opts_t *opts)
{
  yai_display_result_t mapped;
  FILE *stream = (rc == 0) ? stdout : stderr;
  char subject[256];
  char reply_summary[512];
  char reply_hint_1[256];
  char reply_hint_2[256];
  const char *effective_detail = NULL;
  const char *effective_hint_1 = NULL;
  const char *effective_hint_2 = NULL;
  int use_color = (opts && opts->use_color) ? 1 : 0;
  yai_style_role_t status_role = YAI_STYLE_INFO;
  if (!out || !out->status[0] || !out->code[0] || !out->reason[0]) return 0;
  if (!build_subject(opts, out, subject, sizeof(subject))) return 0;
  yai_display_from_reply(out, &mapped);
  extract_reply_human_fields(out,
                             reply_summary, sizeof(reply_summary),
                             reply_hint_1, sizeof(reply_hint_1),
                             reply_hint_2, sizeof(reply_hint_2));
  effective_detail = reply_summary[0] ? reply_summary : mapped.detail;
  effective_hint_1 = reply_hint_1[0] ? reply_hint_1 : mapped.hint;
  effective_hint_2 = reply_hint_2[0] ? reply_hint_2 : NULL;

  if (strcmp(out->status, "ok") == 0) status_role = YAI_STYLE_OK;
  else if (strcmp(out->status, "nyi") == 0 || strcmp(out->code, "RUNTIME_NOT_READY") == 0) status_role = YAI_STYLE_WARN;
  else status_role = YAI_STYLE_ERR;
  print_line(stream, subject, use_color, yai_style_color(YAI_STYLE_EMPH));
  print_line(stream, mapped.status_label, use_color, yai_style_color(status_role));
  yai_color_print(stream, use_color, yai_style_color(YAI_STYLE_INFO), "Details:");
  fputc('\n', stream);
  if (effective_detail && effective_detail[0]) fprintf(stream, "  summary=%s\n", effective_detail);
  fprintf(stream, "  status=%s\n", out->status);
  fprintf(stream, "  code=%s\n", out->code);
  fprintf(stream, "  reason=%s\n", out->reason);
  fprintf(stream, "  command_id=%s\n", out->command_id);
  fprintf(stream, "  target_plane=%s\n", out->target_plane);
  if (out->trace_id[0]) {
    yai_color_print(stream, use_color, yai_style_color(YAI_STYLE_MUTED), "Trace: ");
    fprintf(stream, "%s\n", out->trace_id);
  }
  if (effective_hint_1 && effective_hint_1[0] && strcmp(out->status, "ok") != 0) {
    fprintf(stream, "Hint: %s\n", effective_hint_1);
    if (effective_hint_2 && effective_hint_2[0]) fprintf(stream, "Hint: %s\n", effective_hint_2);
  }
  return 1;
}

int yai_render_exec_contract_verbose(const yai_sdk_reply_t *out, int rc, const char *control_call_json)
{
  cJSON *call_doc = NULL;
  cJSON *reply_doc = NULL;
  cJSON *argv_node = NULL;
  cJSON *hints = NULL;
  cJSON *hint = NULL;
  const cJSON *node = NULL;
  const char *s = NULL;
  int argv_count = 0;
  FILE *stream = (rc == 0) ? stdout : stderr;
  if (!out) return 0;

  print_title(stream, "Contract verbose");

  call_doc = cJSON_Parse((control_call_json && control_call_json[0]) ? control_call_json : "{}");
  reply_doc = (out->exec_reply_json && out->exec_reply_json[0]) ? cJSON_Parse(out->exec_reply_json) : NULL;

  print_section(stream, "Control call");
  if (call_doc) {
    node = cJSON_GetObjectItemCaseSensitive(call_doc, "type");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Type", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(call_doc, "target_plane");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Target plane", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(call_doc, "command_id");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Command id", node->valuestring);
    argv_node = cJSON_GetObjectItemCaseSensitive(call_doc, "argv");
    if (cJSON_IsArray(argv_node)) argv_count = cJSON_GetArraySize(argv_node);
    {
      char count_buf[32];
      snprintf(count_buf, sizeof(count_buf), "%d", argv_count);
      print_kv(stream, "Argument count", count_buf);
    }
    if (cJSON_IsArray(argv_node) && argv_count > 0) {
      const cJSON *arg0 = cJSON_GetArrayItem(argv_node, 0);
      const cJSON *arg1 = cJSON_GetArrayItem(argv_node, 1);
      const cJSON *arg2 = cJSON_GetArrayItem(argv_node, 2);
      if (cJSON_IsString(arg0) && arg0->valuestring) print_kv(stream, "Arg[0]", arg0->valuestring);
      if (cJSON_IsString(arg1) && arg1->valuestring) print_kv(stream, "Arg[1]", arg1->valuestring);
      if (cJSON_IsString(arg2) && arg2->valuestring) print_kv(stream, "Arg[2]", arg2->valuestring);
    }
  } else {
    print_kv(stream, "Summary", "unavailable");
  }

  print_section(stream, "Exec reply");
  if (reply_doc) {
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "type");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Type", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "status");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Status", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "code");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Code", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "reason");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Reason", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "summary");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Summary", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "command_id");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Command id", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "target_plane");
    if (cJSON_IsString(node) && node->valuestring) print_kv(stream, "Target plane", node->valuestring);
    node = cJSON_GetObjectItemCaseSensitive(reply_doc, "trace_id");
    if (cJSON_IsString(node) && node->valuestring && node->valuestring[0]) print_kv(stream, "Trace", node->valuestring);
    hints = cJSON_GetObjectItemCaseSensitive(reply_doc, "hints");
    if (cJSON_IsArray(hints)) {
      hint = cJSON_GetArrayItem(hints, 0);
      if (cJSON_IsString(hint) && hint->valuestring && hint->valuestring[0]) print_kv(stream, "Hint", hint->valuestring);
      hint = cJSON_GetArrayItem(hints, 1);
      if (cJSON_IsString(hint) && hint->valuestring && hint->valuestring[0]) print_kv(stream, "Hint 2", hint->valuestring);
    }
  } else {
    print_kv(stream, "Summary", "unavailable");
  }

  print_section(stream, "Raw payloads");
  s = (control_call_json && control_call_json[0]) ? control_call_json : "{}";
  print_json_block(stream, "control_call_json", s);
  if (out->exec_reply_json && out->exec_reply_json[0]) {
    print_json_block(stream, "exec_reply_json", out->exec_reply_json);
  } else {
    print_json_block(stream, "exec_reply_json", "{}");
  }

  if (call_doc) cJSON_Delete(call_doc);
  if (reply_doc) cJSON_Delete(reply_doc);
  return 1;
}

int yai_render_exec_json(const yai_sdk_reply_t *out)
{
  if (!out || !out->exec_reply_json) return 0;
  printf("%s\n", out->exec_reply_json);
  return 1;
}

int yai_render_exec_exit_code(const yai_sdk_reply_t *out, int sdk_rc)
{
  if (!out || !out->status[0] || !out->code[0]) {
    return (sdk_rc == 0) ? 0 : 50;
  }
  if (strcmp(out->status, "ok") == 0 && strcmp(out->code, "OK") == 0) return 0;
  if (strcmp(out->status, "nyi") == 0 || strcmp(out->code, "NOT_IMPLEMENTED") == 0) return 10;
  if (strcmp(out->code, "BAD_ARGS") == 0 || strcmp(out->code, "INVALID_TARGET") == 0) return 20;
  if (strcmp(out->code, "POLICY_BLOCK") == 0) return 30;
  if (strcmp(out->code, "UNAUTHORIZED") == 0) return 30;
  if (strcmp(out->code, "SERVER_UNAVAILABLE") == 0 || strcmp(out->code, "RUNTIME_NOT_READY") == 0) return 40;
  return 50;
}
