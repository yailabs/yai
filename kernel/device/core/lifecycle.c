#include <yai/drv/driver_lifecycle.h>

const char *yai_driver_state_name(yai_driver_state_t value) {
  switch (value) {
    case YAI_DRIVER_STATE_REGISTERED: return "registered";
    case YAI_DRIVER_STATE_READY: return "ready";
    case YAI_DRIVER_STATE_FAILED: return "failed";
    case YAI_DRIVER_STATE_UNREGISTERED:
    default: return "unregistered";
  }
}
