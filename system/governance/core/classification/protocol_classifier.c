#include "../internal.h"

#include <string.h>

int yai_governance_classify_protocol(const char *payload, char *out, size_t out_cap) {
  const char *v = "control";
  if (!payload || !out || out_cap == 0) return -1;

  if (strstr(payload, "http") || strstr(payload, "https") || strstr(payload, "curl")) v = "https";
  else if (strstr(payload, "grpc")) v = "grpc";
  else if (strstr(payload, "file")) v = "file";

  return yai_governance_safe_snprintf(out, out_cap, "%s", v);
}
