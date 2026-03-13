#pragma once

typedef enum {
  YAI_DRIVER_STATE_UNREGISTERED = 0,
  YAI_DRIVER_STATE_REGISTERED,
  YAI_DRIVER_STATE_READY,
  YAI_DRIVER_STATE_FAILED,
} yai_driver_state_t;

const char *yai_driver_state_name(yai_driver_state_t value);
