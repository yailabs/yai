#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <string.h>

#include <yai/fs/path.h>

static int append_segment(char *out, size_t out_cap, size_t *io_len, const char *seg) {
  int n;

  if (!out || !io_len || !seg || seg[0] == '\0') {
    return -1;
  }

  if (*io_len > 0 && out[*io_len - 1] != '/') {
    if (*io_len + 1 >= out_cap) {
      return -1;
    }
    out[*io_len] = '/';
    *io_len += 1;
    out[*io_len] = '\0';
  }

  n = snprintf(out + *io_len, out_cap - *io_len, "%s", seg);
  if (n < 0 || (size_t)n >= out_cap - *io_len) {
    return -1;
  }

  *io_len += (size_t)n;
  return 0;
}

int yai_fs_path_is_absolute(const char *path) {
  return (path && path[0] == '/') ? 1 : 0;
}

int yai_fs_path_join(const char *base,
                     const char *relative,
                     char *out,
                     size_t out_cap) {
  int n;

  if (!base || !relative || !out || out_cap == 0) {
    return -1;
  }

  if (relative[0] == '/') {
    n = snprintf(out, out_cap, "%s", relative);
  } else {
    n = snprintf(out, out_cap, "%s/%s", base, relative);
  }

  if (n < 0 || (size_t)n >= out_cap) {
    return -1;
  }

  return 0;
}

int yai_fs_path_normalize(const char *input,
                          char *out,
                          size_t out_cap) {
  char buf[1024];
  char *tok;
  char *save = NULL;
  char *segments[64];
  size_t seg_len = 0;
  size_t len = 0;
  size_t i;
  int absolute;

  if (!input || !out || out_cap == 0) {
    return -1;
  }

  if (snprintf(buf, sizeof(buf), "%s", input) >= (int)sizeof(buf)) {
    return -1;
  }

  absolute = (input[0] == '/');
  tok = strtok_r(buf, "/", &save);

  while (tok) {
    if (strcmp(tok, ".") == 0 || tok[0] == '\0') {
      tok = strtok_r(NULL, "/", &save);
      continue;
    }
    if (strcmp(tok, "..") == 0) {
      if (seg_len == 0) {
        if (absolute) {
          return -1;
        }
      } else {
        seg_len--;
      }
      tok = strtok_r(NULL, "/", &save);
      continue;
    }
    if (seg_len >= (sizeof(segments) / sizeof(segments[0]))) {
      return -1;
    }
    segments[seg_len++] = tok;
    tok = strtok_r(NULL, "/", &save);
  }

  out[0] = '\0';
  if (absolute) {
    if (out_cap < 2) {
      return -1;
    }
    out[0] = '/';
    out[1] = '\0';
    len = 1;
  }

  for (i = 0; i < seg_len; ++i) {
    if (append_segment(out, out_cap, &len, segments[i]) != 0) {
      return -1;
    }
  }

  if (out[0] == '\0') {
    if (snprintf(out, out_cap, "%s", absolute ? "/" : ".") >= (int)out_cap) {
      return -1;
    }
  }

  return 0;
}
