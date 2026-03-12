#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/loader.h>

int main(void) {
  yai_governance_runtime_t rt;
  char err[256] = {0};

  if (yai_governance_load_runtime(&rt, err, sizeof(err)) != 0) {
    fprintf(stderr, "no_legacy_primary_path: runtime load failed: %s\n", err);
    return 1;
  }

  if (strstr(rt.root, "/la" "w") != NULL || strstr(rt.root, "la" "w/") != NULL) {
    fprintf(stderr, "no_legacy_primary_path: legacy split-repo path selected as primary path\n");
    return 1;
  }

  if (strncmp(rt.root, "governance", 10) != 0) {
    fprintf(stderr, "no_legacy_primary_path: unexpected primary root: %s\n", rt.root);
    return 1;
  }

  puts("no_legacy_primary_path: ok");
  return 0;
}
