#include <stdio.h>
#include <string.h>

#include <yai/dmn/sources/action_points.h>

static unsigned long fnv1a(const char *s)
{
  unsigned long h = 2166136261u;
  size_t i = 0;
  if (!s)
  {
    return h;
  }
  for (i = 0; s[i]; ++i)
  {
    h ^= (unsigned long)(unsigned char)s[i];
    h *= 16777619u;
  }
  return h;
}

int yai_edge_action_point_id(char *out,
                               size_t out_cap,
                               const char *source_binding_id,
                               const char *action_ref)
{
  unsigned long h = 0;
  if (!out || out_cap == 0 || !source_binding_id || !source_binding_id[0] || !action_ref || !action_ref[0])
  {
    return -1;
  }
  h = fnv1a(source_binding_id) ^ fnv1a(action_ref);
  if (snprintf(out, out_cap, "sap-%lx", h) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}

