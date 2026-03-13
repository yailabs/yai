/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/orchestration/transport.h>

#include <yai/cognition/cognition.h>
#include <yai/cognition/memory.h>
#include <yai/network/providers/catalog.h>

#include <stdio.h>
#include <string.h>

static int parse_payload_after(const char *raw, const char *prefix, char *out, size_t out_cap)
{
  size_t n;
  if (!raw || !prefix || !out || out_cap == 0) return YAI_MIND_ERR_INVALID_ARG;
  if (strncmp(raw, prefix, strlen(prefix)) != 0) return YAI_MIND_ERR_INVALID_ARG;
  raw += strlen(prefix);
  while (*raw == ' ') raw++;
  n = strlen(raw);
  while (n > 0 && (raw[n - 1] == '\n' || raw[n - 1] == '\r')) n--;
  if (n >= out_cap) n = out_cap - 1;
  memcpy(out, raw, n);
  out[n] = '\0';
  return YAI_MIND_OK;
}

int yai_protocol_parse(const char *raw,
                            yai_protocol_request_t *request_out)
{
  if (!raw || !request_out) return YAI_MIND_ERR_INVALID_ARG;
  memset(request_out, 0, sizeof(*request_out));

  if (strncmp(raw, "PING", 4) == 0) {
    request_out->kind = YAI_MIND_PROTOCOL_PING;
    return YAI_MIND_OK;
  }
  if (strncmp(raw, "COMPLETE", 8) == 0) {
    request_out->kind = YAI_MIND_PROTOCOL_COMPLETE;
    return parse_payload_after(raw, "COMPLETE", request_out->payload, sizeof(request_out->payload));
  }
  if (strncmp(raw, "EMBED", 5) == 0) {
    request_out->kind = YAI_MIND_PROTOCOL_EMBED;
    return parse_payload_after(raw, "EMBED", request_out->payload, sizeof(request_out->payload));
  }
  if (strncmp(raw, "QUERY", 5) == 0) {
    request_out->kind = YAI_MIND_PROTOCOL_QUERY;
    return parse_payload_after(raw, "QUERY", request_out->payload, sizeof(request_out->payload));
  }
  if (strncmp(raw, "COGNITION", 9) == 0) {
    request_out->kind = YAI_MIND_PROTOCOL_COGNITION;
    return parse_payload_after(raw, "COGNITION", request_out->payload, sizeof(request_out->payload));
  }

  request_out->kind = YAI_MIND_PROTOCOL_UNKNOWN;
  return YAI_MIND_ERR_INVALID_ARG;
}

int yai_protocol_dispatch(const yai_protocol_request_t *request,
                               yai_protocol_response_t *response_out)
{
  if (!request || !response_out) return YAI_MIND_ERR_INVALID_ARG;
  memset(response_out, 0, sizeof(*response_out));

  switch (request->kind) {
    case YAI_MIND_PROTOCOL_PING:
      response_out->status = 200;
      response_out->code = YAI_MIND_OK;
      snprintf(response_out->payload, sizeof(response_out->payload), "pong");
      return YAI_MIND_OK;

    case YAI_MIND_PROTOCOL_COMPLETE:
      {
        yai_provider_response_t provider_response = {0};
        int rc = yai_client_completion(request->provider, request->payload, &provider_response);
        response_out->status = (rc == YAI_MIND_OK) ? 200 : 500;
        response_out->code = rc;
        snprintf(response_out->payload, sizeof(response_out->payload), "%s",
                 (rc == YAI_MIND_OK) ? provider_response.output : "provider completion failed");
        return rc;
      }

    case YAI_MIND_PROTOCOL_EMBED:
      {
        float vector_out[4] = {0};
        int rc = yai_client_embedding(request->provider, request->payload, vector_out, 4);
        response_out->status = (rc == YAI_MIND_OK) ? 200 : 500;
        response_out->code = rc;
        if (rc == YAI_MIND_OK) {
          snprintf(response_out->payload, sizeof(response_out->payload),
                   "[%.3f,%.3f,%.3f,%.3f]", vector_out[0], vector_out[1], vector_out[2], vector_out[3]);
        } else {
          snprintf(response_out->payload, sizeof(response_out->payload), "embedding failed");
        }
        return rc;
      }

    case YAI_MIND_PROTOCOL_QUERY:
      {
        yai_memory_query_t query = {0};
        yai_memory_result_t result = {0};
        int rc;
        snprintf(query.query, sizeof(query.query), "%.240s", request->payload);
        query.limit = 10;
        rc = yai_memory_query_run(&query, &result);
        response_out->status = (rc == YAI_MIND_OK) ? 200 : 500;
        response_out->code = rc;
        if (rc == YAI_MIND_OK) {
          snprintf(response_out->payload, sizeof(response_out->payload), "matches=%d %s",
                   result.match_count, result.summary);
        } else {
          snprintf(response_out->payload, sizeof(response_out->payload), "query failed");
        }
        return rc;
      }

    case YAI_MIND_PROTOCOL_COGNITION:
      {
        yai_cognition_response_t cognition = {0};
        int rc = yai_cognition_execute_text(request->payload,
                                                 "transport-session",
                                                 request->provider,
                                                 &cognition);
        response_out->status = (rc == YAI_MIND_OK) ? 200 : 500;
        response_out->code = rc;
        if (rc == YAI_MIND_OK) {
          snprintf(response_out->payload, sizeof(response_out->payload),
                   "role=%s score=%.2f %.340s",
                   yai_agent_role_name(cognition.selected_role),
                   cognition.score,
                   cognition.output);
        } else {
          snprintf(response_out->payload, sizeof(response_out->payload), "cognition failed");
        }
        return rc;
      }

    default:
      response_out->status = 400;
      response_out->code = YAI_MIND_ERR_INVALID_ARG;
      snprintf(response_out->payload, sizeof(response_out->payload), "unknown request type");
      return YAI_MIND_ERR_INVALID_ARG;
  }
}

int yai_protocol_format_response(const yai_protocol_response_t *response,
                                      char *out,
                                      size_t out_cap)
{
  if (!response || !out || out_cap == 0) return YAI_MIND_ERR_INVALID_ARG;
  snprintf(out, out_cap, "STATUS %d CODE %d %s\n", response->status, response->code, response->payload);
  return YAI_MIND_OK;
}
