#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

#include "internal.h"

static int ensure_dir_component(const char *path)
{
  struct stat st;
  if (!path || !path[0])
  {
    return -1;
  }
  if (stat(path, &st) == 0)
  {
    return S_ISDIR(st.st_mode) ? 0 : -1;
  }
  if (mkdir(path, 0755) == 0)
  {
    return 0;
  }
  if (errno == EEXIST)
  {
    if (stat(path, &st) == 0 && S_ISDIR(st.st_mode))
    {
      return 0;
    }
  }
  return -1;
}

int yai_edge_write_file(const char *path, const char *payload)
{
  FILE *f = NULL;
  if (!path || !payload)
  {
    return -1;
  }
  f = fopen(path, "w");
  if (!f)
  {
    return -1;
  }
  if (fputs(payload, f) == EOF)
  {
    fclose(f);
    return -1;
  }
  fclose(f);
  return 0;
}

int yai_edge_mkdir_recursive(const char *path)
{
  char tmp[1024];
  size_t i = 0;

  if (!path || !path[0] || strlen(path) >= sizeof(tmp))
  {
    return -1;
  }
  memset(tmp, 0, sizeof(tmp));
  (void)snprintf(tmp, sizeof(tmp), "%s", path);

  for (i = 1; tmp[i]; ++i)
  {
    if (tmp[i] == '/')
    {
      tmp[i] = '\0';
      if (tmp[0] != '\0' && ensure_dir_component(tmp) != 0)
      {
        return -1;
      }
      tmp[i] = '/';
    }
  }

  if (ensure_dir_component(tmp) != 0)
  {
    return -1;
  }
  return 0;
}

void yai_edge_vlog(const char *instance_id,
                     const char *level,
                     const char *fmt,
                     va_list ap)
{
  time_t now = time(NULL);
  struct tm tmv;
  char ts[64];
  const char *id = instance_id && instance_id[0] ? instance_id : "yd-bootstrap";
  const char *lvl = level && level[0] ? level : "info";

  if (gmtime_r(&now, &tmv) == NULL)
  {
    memset(&tmv, 0, sizeof(tmv));
  }
  (void)strftime(ts, sizeof(ts), "%Y-%m-%dT%H:%M:%SZ", &tmv);

  fprintf(stderr, "[%s] yai-daemon %s %s: ", ts, id, lvl);
  vfprintf(stderr, fmt, ap);
  fputc('\n', stderr);
}
