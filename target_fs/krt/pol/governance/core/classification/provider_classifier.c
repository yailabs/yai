#include "../internal.h"

#include <string.h>

int yai_governance_classify_provider(const char *payload, char *out, size_t out_cap) {
  const char *v = "provider.default";
  if (!payload || !out || out_cap == 0) return -1;

  if (strstr(payload, "untrusted")) v = "untrusted-provider";
  else if (strstr(payload, "unknown")) v = "unknown-provider";
  else if (strstr(payload, "curl")) v = "curl";
  else if (strstr(payload, "otel")) v = "otel-exporter";
  else if (strstr(payload, "s3")) v = "aws-s3";
  else if (strstr(payload, "cdn")) v = "cdn-edge";
  else if (strstr(payload, "webhook")) v = "webhook-gateway";
  else if (strstr(payload, "slack")) v = "slack-api";
  else if (strstr(payload, "payment") || strstr(payload, "psp")) v = "payment-gateway";
  else if (strstr(payload, "ledger") || strstr(payload, "settlement")) v = "core-ledger";
  else if (strstr(payload, "github")) v = "github-api";
  else if (strstr(payload, "experiment") || strstr(payload, "params")) v = "experiment-runner";

  return yai_governance_safe_snprintf(out, out_cap, "%s", v);
}
