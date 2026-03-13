#include <yai/supervisor/lifecycle.h>
#include <string.h>

const char *yai_core_startup_plan_name(void) {
  const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();
  if (!caps || !caps->initialized) return "runtime_unified_lifecycle_v2_prebind";
  if (caps->workspace_id[0] && strcmp(caps->workspace_id, "system") != 0) {
    return "runtime_unified_lifecycle_v2_workspace_bound";
  }
  return "runtime_unified_lifecycle_v2_global_only";
}
