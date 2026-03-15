#include "internal/static_string.h"

size_t yai_kernel_static_strlcpy(char *dst, const char *src, size_t dst_size) {
  size_t i = 0;

  if (!src) {
    if (dst && dst_size > 0) {
      dst[0] = '\0';
    }
    return 0;
  }

  if (!dst || dst_size == 0) {
    while (src[i] != '\0') {
      i += 1;
    }
    return i;
  }

  while (src[i] != '\0' && (i + 1) < dst_size) {
    dst[i] = src[i];
    i += 1;
  }

  dst[i] = '\0';

  while (src[i] != '\0') {
    i += 1;
  }

  return i;
}
