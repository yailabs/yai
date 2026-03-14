#pragma once

typedef enum {
  YAI_ARCH_PLATFORM_UNKNOWN = 0,
  YAI_ARCH_PLATFORM_POSIX_HOST,
} yai_arch_platform_t;

const char *yai_arch_platform_name(yai_arch_platform_t value);
yai_arch_platform_t yai_arch_platform_current(void);
