#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/loader.h>

int main(void) {
  yai_governance_runtime_t rt;
  char err[256] = {0};
  char json[8192] = {0};

  if (yai_governance_load_runtime(&rt, err, sizeof(err)) != 0) return 1;

  if (yai_governance_load_control_family_descriptor(&rt, "digital", json, sizeof(json)) != 0) {
    fprintf(stderr, "family_descriptor_loader: digital descriptor not loaded\n");
    return 1;
  }
  if (!strstr(json, "\"kind\": \"control_family_descriptor.v1\"")) {
    fprintf(stderr, "family_descriptor_loader: descriptor kind mismatch\n");
    return 1;
  }
  if (!strstr(json, "\"canonical_name\": \"digital\"")) {
    fprintf(stderr, "family_descriptor_loader: canonical_name mismatch\n");
    return 1;
  }

  if (yai_governance_load_control_family_descriptor(&rt, "D1-digital", json, sizeof(json)) != 0) {
    fprintf(stderr, "family_descriptor_loader: domain lookup id did not resolve family descriptor\n");
    return 1;
  }
  if (!strstr(json, "\"family_id\": \"D1\"")) {
    fprintf(stderr, "family_descriptor_loader: domain lookup descriptor family id mismatch\n");
    return 1;
  }

  puts("family_descriptor_loader: ok");
  return 0;
}
