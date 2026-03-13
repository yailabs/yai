#include <ctype.h>

#include <yai/support/strings.h>

int yai_str_is_blank(const char *s) {
  if (!s) {
    return 1;
  }
  while (*s) {
    if (!isspace((unsigned char)*s)) {
      return 0;
    }
    s++;
  }
  return 1;
}
