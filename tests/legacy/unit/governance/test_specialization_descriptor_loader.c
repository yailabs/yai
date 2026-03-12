#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/loader.h>

int main(void) {
  yai_governance_runtime_t rt;
  char err[256] = {0};
  char json[16384] = {0};

  if (yai_governance_load_runtime(&rt, err, sizeof(err)) != 0) return 1;

  if (yai_governance_load_specialization_descriptor(&rt, "payments", json, sizeof(json)) != 0) {
    fprintf(stderr, "specialization_descriptor_loader: payments descriptor not loaded\n");
    return 1;
  }
  if (!strstr(json, "\"kind\": \"specialization_descriptor.v1\"")) {
    fprintf(stderr, "specialization_descriptor_loader: descriptor kind mismatch\n");
    return 1;
  }
  if (!strstr(json, "\"specialization_id\": \"payments\"")) {
    fprintf(stderr, "specialization_descriptor_loader: specialization_id mismatch\n");
    return 1;
  }

  if (yai_governance_load_specialization_descriptor(&rt, "economic.payments", json, sizeof(json)) != 0) {
    fprintf(stderr, "specialization_descriptor_loader: canonical_name lookup failed\n");
    return 1;
  }
  if (!strstr(json, "\"family\": \"economic\"")) {
    fprintf(stderr, "specialization_descriptor_loader: family mismatch\n");
    return 1;
  }

  puts("specialization_descriptor_loader: ok");
  return 0;
}
