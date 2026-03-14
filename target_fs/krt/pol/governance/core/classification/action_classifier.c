#include "../internal.h"

#include <string.h>

int yai_governance_classify_action(const char *payload, char *out, size_t out_cap) {
  const char *v = "transform";
  if (!payload || !out || out_cap == 0) return -1;

  if (strstr(payload, "curl") || strstr(payload, "egress") || strstr(payload, "otel") || strstr(payload, "s3") ||
      strstr(payload, "outbound") || strstr(payload, "webhook") || strstr(payload, "upload")) {
    v = "egress";
  } else if (strstr(payload, "github") || strstr(payload, "comment") || strstr(payload, "publish") ||
             strstr(payload, "release")) {
    v = "publish";
  } else if (strstr(payload, "retrieve") || strstr(payload, "read") || strstr(payload, "fetch")) {
    v = "retrieve";
  } else if (strstr(payload, "distribution") || strstr(payload, "deliver") || strstr(payload, "share")) {
    v = "distribute";
  } else if (strstr(payload, "sink") || strstr(payload, "destination") || strstr(payload, "target")) {
    v = "sink-control";
  } else if (strstr(payload, "payment") || strstr(payload, "transfer") || strstr(payload, "settlement") ||
             strstr(payload, "ledger") || strstr(payload, "authorize")) {
    v = "authorize";
    if (strstr(payload, "settlement")) v = "settle";
    if (strstr(payload, "fraud") || strstr(payload, "freeze") || strstr(payload, "hold")) v = "hold";
  } else if (strstr(payload, "infer") || strstr(payload, "model")) {
    v = "infer";
  } else if (strstr(payload, "actuate")) {
    v = "actuate";
  } else if (strstr(payload, "experiment") || strstr(payload, "params")) {
    v = strstr(payload, "params") ? "parameter-modify" : "experiment-run";
  }

  if (yai_governance_safe_snprintf(out, out_cap, "%s", v) != 0) return -1;
  return 0;
}
