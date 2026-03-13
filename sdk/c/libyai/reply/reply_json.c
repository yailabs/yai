/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/reply/reply_json.h>

#include <string.h>

static const char *outcome_str(yai_outcome_t o)
{
  switch (o) {
    case YAI_OUTCOME_OK: return "ok";
    case YAI_OUTCOME_WARN: return "warn";
    case YAI_OUTCOME_DENY: return "deny";
    case YAI_OUTCOME_FAIL: return "fail";
    default: return "fail";
  }
}

char *yai_reply_to_json(const yai_reply_t *r)
{
  cJSON *root;
  cJSON *hint;
  cJSON *trace;
  cJSON *meta;
  char *out;
  size_t i;

  if (!r) return NULL;

  root = cJSON_CreateObject();
  if (!root) return NULL;

  cJSON_AddStringToObject(root, "type", "yai.exec.reply.v1");
  cJSON_AddStringToObject(root, "outcome", outcome_str(r->outcome));
  cJSON_AddStringToObject(root, "code", r->code ? r->code : "INTERNAL_ERROR");
  cJSON_AddStringToObject(root, "summary", r->summary ? r->summary : "operation failed");

  hint = cJSON_AddArrayToObject(root, "hint");
  for (i = 0; i < r->hint_count; i++) {
    if (r->hints && r->hints[i] && r->hints[i][0]) {
      cJSON_AddItemToArray(hint, cJSON_CreateString(r->hints[i]));
    }
  }

  if (r->data) {
    cJSON_AddItemReferenceToObject(root, "data", r->data);
  } else {
    cJSON_AddItemToObject(root, "data", cJSON_CreateObject());
  }

  trace = cJSON_AddObjectToObject(root, "trace");
  cJSON_AddStringToObject(trace, "trace_id", r->trace.trace_id ? r->trace.trace_id : "");
  cJSON_AddStringToObject(trace, "claim_id", r->trace.claim_id ? r->trace.claim_id : "");
  cJSON_AddStringToObject(trace, "container_id", r->trace.container_id ? r->trace.container_id : "");
  cJSON_AddStringToObject(trace, "law_ref", r->trace.law_ref ? r->trace.law_ref : "");
  cJSON_AddStringToObject(trace, "evidence_ref", r->trace.evidence_ref ? r->trace.evidence_ref : "");

  meta = cJSON_AddObjectToObject(root, "meta");
  cJSON_AddStringToObject(meta, "version", r->meta.version ? r->meta.version : "");
  cJSON_AddStringToObject(meta, "ts", r->meta.ts ? r->meta.ts : "");
  cJSON_AddNumberToObject(meta, "duration_ms", r->meta.duration_ms);
  cJSON_AddStringToObject(meta, "target_plane", r->meta.target_plane ? r->meta.target_plane : "");
  cJSON_AddStringToObject(meta, "command_id", r->meta.command_id ? r->meta.command_id : "");

  out = cJSON_PrintUnformatted(root);
  cJSON_Delete(root);
  return out;
}
