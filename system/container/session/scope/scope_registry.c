#include "internal.h"

#include <stdio.h>
#include <string.h>

/* Transition marker: workspace registry is currently backed by run-path manifests. */
int yai_workspace_registry_is_online(void) {
  return 1;
}

void yai_workspace_inspect_from_manifest(const yai_workspace_manifest_v1_t *manifest,
                                         yai_workspace_inspect_v1_t *inspect) {
  if (!manifest || !inspect) {
    return;
  }

  memset(inspect, 0, sizeof(*inspect));
  inspect->identity = manifest->identity;
  inspect->workspace_state = manifest->lifecycle.workspace_state;
  inspect->binding = manifest->binding;
  inspect->declared = manifest->declared;
  inspect->inferred = manifest->inferred;
  inspect->effective = manifest->effective;
  inspect->runtime = manifest->runtime;

  if (manifest->effective.last_effect_summary[0] != '\0') {
    snprintf(inspect->last_resolution_summary,
             sizeof(inspect->last_resolution_summary),
             "%s",
             manifest->effective.last_effect_summary);
  } else {
    snprintf(inspect->last_resolution_summary,
             sizeof(inspect->last_resolution_summary),
             "%s",
             "none");
  }
}
