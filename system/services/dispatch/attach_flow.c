#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <yai/supervisor/lifecycle.h>

int yai_core_attach_flow_enabled(void) {
  const char *disable = getenv("YAI_DISABLE_ATTACH_FLOW");
  if (disable && (strcmp(disable, "1") == 0 || strcasecmp(disable, "true") == 0)) return 0;
  if (yai_runtime_capabilities_is_ready() == 0) return 0;
  return 1;
}
