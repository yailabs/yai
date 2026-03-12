#include <stdio.h>
#include <string.h>

#include <yai/policy/governance/loader.h>

int main(void) {
  yai_governance_runtime_t rt;
  char err[256] = {0};
  if (yai_governance_load_runtime(&rt, err, sizeof(err)) != 0) {
    fprintf(stderr, "contract_surface: runtime load failed: %s\n", err);
    return 1;
  }

  if (rt.entrypoint.domains_ref[0] == '\0' || rt.entrypoint.compliance_ref[0] == '\0') {
    fprintf(stderr, "contract_surface: missing runtime refs\n");
    return 1;
  }
  if (rt.manifest.domain_index_ref[0] == '\0' || rt.manifest.compliance_index_ref[0] == '\0') {
    fprintf(stderr, "contract_surface: missing manifest index refs\n");
    return 1;
  }
  if (rt.entrypoint.resolution_order_ref[0] == '\0' || rt.entrypoint.compatibility_ref[0] == '\0') {
    fprintf(stderr, "contract_surface: missing entrypoint contract refs\n");
    return 1;
  }
  if (strncmp(rt.entrypoint.governance_manifest_ref, "model/manifests/", 16) != 0 ||
      strncmp(rt.entrypoint.resolution_order_ref, "model/manifests/", 16) != 0 ||
      strncmp(rt.entrypoint.compatibility_ref, "model/manifests/", 16) != 0) {
    fprintf(stderr, "contract_surface: manifest refs are not canonical governance paths\n");
    return 1;
  }
  if (rt.compatibility.profile[0] == '\0') {
    fprintf(stderr, "contract_surface: missing compatibility profile\n");
    return 1;
  }

  puts("contract_surface: ok");
  return 0;
}
