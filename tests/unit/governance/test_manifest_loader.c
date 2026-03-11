#include <stdio.h>
#include <string.h>

#include <yai/law/loader.h>

int main(void) {
  yai_law_runtime_t rt;
  char err[256] = {0};

  if (yai_law_load_runtime(&rt, err, sizeof(err)) != 0) {
    fprintf(stderr, "manifest_loader: failed: %s\n", err);
    return 1;
  }

  if (rt.root[0] == '\0' || rt.entrypoint.entrypoint_id[0] == '\0') {
    fprintf(stderr, "manifest_loader: invalid runtime payload\n");
    return 1;
  }
  if (strncmp(rt.runtime_view.domain_resolution_ref, "governance/manifests/", 21) != 0 ||
      strncmp(rt.runtime_view.compliance_resolution_ref, "governance/manifests/", 21) != 0) {
    fprintf(stderr, "manifest_loader: runtime resolution refs are not canonical governance paths\n");
    return 1;
  }

  puts("manifest_loader: ok");
  return 0;
}
