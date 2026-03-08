#pragma once

#include <yai/api/status.h>

typedef enum yai_runtime_mode
{
  YAI_RUNTIME_MODE_UNSPECIFIED = 0,
  YAI_RUNTIME_MODE_RUNTIME,
  YAI_RUNTIME_MODE_DIAGNOSTIC
} yai_runtime_mode_t;

#define YAI_BIN_MAIN "yai"