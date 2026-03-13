#include <yai/arch/platform.h>

const char *yai_arch_platform_name(yai_arch_platform_t value) {
  switch (value) {
    case YAI_ARCH_PLATFORM_POSIX_HOST: return "posix_host";
    case YAI_ARCH_PLATFORM_UNKNOWN:
    default: return "unknown";
  }
}

yai_arch_platform_t yai_arch_platform_current(void) {
  return YAI_ARCH_PLATFORM_POSIX_HOST;
}
