#include <stdio.h>

#include <yai/lib/paths.h>

int yai_path_join(char *out, unsigned long out_cap, const char *a, const char *b) {
  int n;
  if (!out || !a || !b || out_cap == 0) {
    return -1;
  }
  n = snprintf(out, (size_t)out_cap, "%s/%s", a, b);
  if (n < 0 || (unsigned long)n >= out_cap) {
    return -1;
  }
  return 0;
}
