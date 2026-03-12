#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/loader.h>

int main(void) {
  yai_governance_runtime_t rt;
  char err[256] = {0};
  char json[4096] = {0};

  if (yai_governance_load_runtime(&rt, err, sizeof(err)) != 0) return 1;
  if (yai_governance_load_compliance_index(&rt, json, sizeof(json)) != 0) return 1;

  if (!strstr(json, "gdpr-eu") || !strstr(json, "ai-act")) {
    fprintf(stderr, "compliance_loader: missing expected packs\n");
    return 1;
  }

  puts("compliance_loader: ok");
  return 0;
}
