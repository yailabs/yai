/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/shell/display_map.h"

#include <stdio.h>
#include <string.h>
#include <ctype.h>

static const char *find_flag_value(int argc, char **argv, const char *a, const char *b)
{
  for (int i = 0; i + 1 < argc; i++) {
    if (!argv || !argv[i] || !argv[i + 1]) continue;
    if ((a && strcmp(argv[i], a) == 0) || (b && strcmp(argv[i], b) == 0)) {
      return argv[i + 1];
    }
  }
  return NULL;
}

static int starts_with(const char *s, const char *prefix)
{
  size_t n;
  if (!s || !prefix) return 0;
  n = strlen(prefix);
  return strncmp(s, prefix, n) == 0;
}

void yai_display_reason_human(const char *reason, char *out, size_t out_cap)
{
  size_t j = 0;
  if (!out || out_cap == 0) return;
  out[0] = '\0';
  if (!reason || !reason[0]) {
    snprintf(out, out_cap, "operation failed");
    return;
  }
  for (size_t i = 0; reason[i] && j + 1 < out_cap; i++) {
    char ch = reason[i];
    if (ch == '_') ch = ' ';
    out[j++] = (char)tolower((unsigned char)ch);
  }
  out[j] = '\0';
}

void yai_display_from_command(const char *command_id, int argc, char **argv, yai_display_label_t *out)
{
  const char *container_id = NULL;
  if (!out) return;
  memset(out, 0, sizeof(*out));
  snprintf(out->scope, sizeof(out->scope), "command");
  snprintf(out->action, sizeof(out->action), "%s", (command_id && command_id[0]) ? command_id : "unknown");
  out->detail[0] = '\0';

  if (!command_id || !command_id[0]) return;

  if (strcmp(command_id, "yai.ingress.ping") == 0 ||
      strcmp(command_id, "yai.runtime.ping") == 0) {
    snprintf(out->scope, sizeof(out->scope), "runtime");
    snprintf(out->action, sizeof(out->action), "ping");
    return;
  }
  if (strcmp(command_id, "yai.core.ping") == 0) {
    snprintf(out->scope, sizeof(out->scope), "runtime");
    snprintf(out->action, sizeof(out->action), "ping");
    return;
  }
  if (strcmp(command_id, "yai.lifecycle.up") == 0 ||
      strcmp(command_id, "yai.lifecycle.down") == 0 ||
      strcmp(command_id, "yai.lifecycle.restart") == 0) {
    snprintf(out->scope, sizeof(out->scope), "runtime");
    snprintf(out->action, sizeof(out->action), "%s", strrchr(command_id, '.') + 1);
    return;
  }
  if (strcmp(command_id, "yai.inspect.service") == 0 ||
      strcmp(command_id, "yai.inspect.runtime") == 0 ||
      strcmp(command_id, "yai.doctor.service") == 0 ||
      strcmp(command_id, "yai.doctor.runtime") == 0 ||
      strcmp(command_id, "yai.verify.service") == 0 ||
      strcmp(command_id, "yai.verify.runtime") == 0) {
    snprintf(out->action, sizeof(out->action), "runtime");
    return;
  }
  if (strcmp(command_id, "yai.core.ws") == 0) {
    snprintf(out->scope, sizeof(out->scope), "runtime");
    if (argc > 0 && argv && argv[0]) {
      if (strcmp(argv[0], "create") == 0) snprintf(out->action, sizeof(out->action), "ws create");
      else if (strcmp(argv[0], "reset") == 0) snprintf(out->action, sizeof(out->action), "ws reset");
      else if (strcmp(argv[0], "destroy") == 0) snprintf(out->action, sizeof(out->action), "ws destroy");
      else snprintf(out->action, sizeof(out->action), "ws %s", argv[0]);
    } else {
      snprintf(out->action, sizeof(out->action), "ws");
    }
    container_id = find_flag_value(argc, argv, "--container-id", "--container");
    if (container_id && container_id[0]) {
      snprintf(out->detail, sizeof(out->detail), "container_id=%s", container_id);
    }
    return;
  }
  if (strcmp(command_id, "yai.core.ws_status") == 0) {
    snprintf(out->scope, sizeof(out->scope), "runtime");
    snprintf(out->action, sizeof(out->action), "ws status");
    container_id = find_flag_value(argc, argv, "--container-id", "--container");
    if (container_id && container_id[0]) snprintf(out->detail, sizeof(out->detail), "container_id=%s", container_id);
    return;
  }
  if (strcmp(command_id, "yai.core.ws_list") == 0) {
    snprintf(out->scope, sizeof(out->scope), "runtime");
    snprintf(out->action, sizeof(out->action), "ws list");
    return;
  }
  if (strcmp(command_id, "yai.source.enroll") == 0 ||
      strcmp(command_id, "yai.source.attach") == 0 ||
      strcmp(command_id, "yai.source.list") == 0 ||
      strcmp(command_id, "yai.source.status") == 0 ||
      strcmp(command_id, "yai.source.inspect") == 0 ||
      strcmp(command_id, "yai.source.retry_drain") == 0) {
    snprintf(out->scope, sizeof(out->scope), "source");
    if (strcmp(command_id, "yai.source.retry_drain") == 0) {
      snprintf(out->action, sizeof(out->action), "retry-drain");
    } else {
      snprintf(out->action, sizeof(out->action), "%s", strrchr(command_id, '.') + 1);
    }
    return;
  }

  /* fallback: yai.<scope>.<verb> */
  {
    const char *p = command_id;
    const char *d1 = strchr(p, '.');
    const char *d2 = d1 ? strchr(d1 + 1, '.') : NULL;
    if (d1 && d2 && d2[1]) {
      size_t sl = (size_t)(d2 - (d1 + 1));
      if (sl > 0 && sl < sizeof(out->scope)) {
        memcpy(out->scope, d1 + 1, sl);
        out->scope[sl] = '\0';
      }
      snprintf(out->action, sizeof(out->action), "%s", d2 + 1);
      for (size_t i = 0; out->action[i]; i++) {
        if (out->action[i] == '_') out->action[i] = ' ';
      }
    }
  }
}

void yai_display_from_reply(const yai_sdk_reply_t *reply, yai_display_result_t *out)
{
  if (!out) return;
  memset(out, 0, sizeof(*out));
  if (!reply) {
    snprintf(out->status_label, sizeof(out->status_label), "INTERNAL ERROR");
    snprintf(out->detail, sizeof(out->detail), "reply missing");
    snprintf(out->hint, sizeof(out->hint), "Try: yai help");
    return;
  }

  if (strcmp(reply->status, "ok") == 0 && strcmp(reply->code, "OK") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "OK");
    return;
  }
  if (strcmp(reply->status, "nyi") == 0 || strcmp(reply->code, "NOT_IMPLEMENTED") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "NOT IMPLEMENTED");
    snprintf(out->detail, sizeof(out->detail), "This command is registered but not implemented yet.");
    snprintf(out->hint, sizeof(out->hint), "Try: yai help <group>");
    return;
  }
  if (strcmp(reply->code, "BAD_ARGS") == 0 || strcmp(reply->code, "INVALID_TARGET") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "BAD ARGS");
    if (strcmp(reply->reason, "family_not_found") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Invalid container domain arguments: unknown control family.");
      snprintf(out->hint, sizeof(out->hint), "Use: yai ws domain set --family <valid-family> --specialization <specialization>");
    } else if (strcmp(reply->reason, "specialization_not_found") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Invalid container domain arguments: unknown specialization.");
      snprintf(out->hint, sizeof(out->hint), "Check specialization names in embedded law surface.");
    } else if (strcmp(reply->reason, "specialization_family_mismatch") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Invalid container domain arguments: specialization does not belong to selected family.");
      snprintf(out->hint, sizeof(out->hint), "Set a specialization compatible with the selected family.");
    } else if (strcmp(reply->reason, "container_not_active") == 0) {
      snprintf(out->detail, sizeof(out->detail), "No active container selected for runtime execution.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai ws set <ws-id>");
    } else if (strcmp(reply->reason, "governable_object_not_found") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Unknown governable object id.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai law list");
    } else if (strcmp(reply->reason, "attachment_not_found") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Policy object is not attached to current container.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai ws policy effective");
    } else if (strcmp(reply->reason, "attachment_limit_reached") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Container attachment limit reached.");
      snprintf(out->hint, sizeof(out->hint), "Detach one object first: yai ws policy detach <object-id>");
    } else if (strcmp(reply->reason, "container_context_missing") == 0 ||
               strcmp(reply->reason, "container_not_active") == 0) {
      snprintf(out->detail, sizeof(out->detail), "No container is selected for this command.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai ws set <ws-id>");
    } else if (strcmp(reply->reason, "container_context_invalid") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Container is selected but binding state is invalid.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai ws status");
    } else if (starts_with(reply->reason, "container_postcondition_")) {
      snprintf(out->detail, sizeof(out->detail), "Container binding post-conditions failed: %s", reply->reason);
      snprintf(out->hint, sizeof(out->hint), "Run: yai ws inspect --verbose");
    } else if (strcmp(reply->reason, "source_enroll_missing_label") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Missing source label for enroll.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai source enroll <source-label>");
    } else if (strcmp(reply->reason, "source_attach_missing_source_node_id") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Missing source node id for attach.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai source attach <source-node-id>");
    } else if (strcmp(reply->reason, "source_retry_drain_not_implemented") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Source retry-drain is not implemented in this runtime wave.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai source inspect");
    } else {
      snprintf(out->detail, sizeof(out->detail), "The command arguments are invalid.");
      snprintf(out->hint, sizeof(out->hint), "Try: yai help <group> <command>");
    }
    return;
  }
  if (strcmp(reply->code, "UNAUTHORIZED") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "UNAUTHORIZED");
    snprintf(out->detail, sizeof(out->detail), "Operation blocked by authority policy.");
    snprintf(out->hint, sizeof(out->hint), "Check policy/authority/role configuration");
    return;
  }
  if (strcmp(reply->code, "POLICY_BLOCK") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "DENIED");
    snprintf(out->detail, sizeof(out->detail), "Operation denied by resolved policy stack.");
    snprintf(out->hint, sizeof(out->hint), "Run: yai ws policy effective");
    return;
  }
  if (strcmp(reply->code, "RUNTIME_NOT_READY") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "RUNTIME NOT READY");
    snprintf(out->detail, sizeof(out->detail), "Service is up but not ready for control calls.");
    snprintf(out->hint, sizeof(out->hint), "Try: yai doctor runtime");
    return;
  }
  if (strcmp(reply->code, "SERVER_UNAVAILABLE") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "SERVER UNAVAILABLE");
    if (starts_with(reply->command_id, "yai.container.")) {
      snprintf(out->detail, sizeof(out->detail), "Runtime control plane is unreachable before container checks.");
      snprintf(out->hint, sizeof(out->hint), "Run: yai up");
    } else if (strstr(reply->reason, "runtime_ingress") != NULL) {
      snprintf(out->detail, sizeof(out->detail), "Service ingress is unreachable.");
      snprintf(out->hint, sizeof(out->hint), "Start the service with: yai up");
    } else {
      snprintf(out->detail, sizeof(out->detail), "Service endpoint is unreachable.");
      snprintf(out->hint, sizeof(out->hint), "Start the service with: yai up");
    }
    return;
  }
  if (strcmp(reply->code, "PROTOCOL_ERROR") == 0) {
    snprintf(out->status_label, sizeof(out->status_label), "PROTOCOL ERROR");
    if (strcmp(reply->reason, "rpc_call_failed") == 0) {
      snprintf(out->detail, sizeof(out->detail), "Runtime routing failed for the requested command.");
      snprintf(out->hint, sizeof(out->hint), "Run with --verbose-contract and capture control_call/exec_reply");
    } else {
      snprintf(out->detail, sizeof(out->detail), "Reply payload is not compliant with exec_reply.v1.");
      snprintf(out->hint, sizeof(out->hint), "Try again with --verbose-contract and attach control_call/exec_reply");
    }
    return;
  }

  snprintf(out->status_label, sizeof(out->status_label), "INTERNAL ERROR");
  snprintf(out->detail, sizeof(out->detail), "Unexpected runtime failure.");
  snprintf(out->hint, sizeof(out->hint), "Try again with --verbose-contract and attach control_call/exec_reply");
}
