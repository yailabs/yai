#define _POSIX_C_SOURCE 200809L

#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <yai/edge/ids.h>

static unsigned int g_source_seq = 0;

static void slugify(const char *in, char *out, size_t out_cap)
{
  size_t j = 0;
  size_t i = 0;
  if (!out || out_cap == 0)
  {
    return;
  }
  if (!in || !in[0])
  {
    (void)snprintf(out, out_cap, "%s", "none");
    return;
  }

  for (i = 0; in[i] && j + 1 < out_cap; ++i)
  {
    unsigned char c = (unsigned char)in[i];
    if (isalnum(c))
    {
      out[j++] = (char)tolower(c);
    }
    else if (c == '-' || c == '_' || c == '/' || c == '.')
    {
      out[j++] = '-';
    }
  }
  out[j] = '\0';
  if (j == 0)
  {
    (void)snprintf(out, out_cap, "%s", "none");
  }
}

static int make_id(char *out,
                   size_t out_cap,
                   const char *prefix,
                   const char *a,
                   const char *b)
{
  char aslug[96];
  char bslug[96];
  long ts = (long)time(NULL);
  g_source_seq += 1U;

  if (!out || out_cap == 0 || !prefix || !prefix[0])
  {
    return -1;
  }

  slugify(a, aslug, sizeof(aslug));
  slugify(b, bslug, sizeof(bslug));

  if (snprintf(out,
               out_cap,
               "%s-%s-%s-%ld-%u",
               prefix,
               aslug,
               bslug,
               ts,
               g_source_seq) >= (int)out_cap)
  {
    return -1;
  }
  return 0;
}

int yai_source_id_node(char *out, size_t out_cap, const char *source_label)
{
  return make_id(out, out_cap, "sn", source_label, "node");
}

int yai_source_id_daemon_instance(char *out, size_t out_cap, const char *source_node_id)
{
  return make_id(out, out_cap, "sd", source_node_id, "instance");
}

int yai_source_id_binding(char *out, size_t out_cap, const char *source_node_id, const char *workspace_id)
{
  return make_id(out, out_cap, "sb", source_node_id, workspace_id);
}

int yai_source_id_asset(char *out, size_t out_cap, const char *source_binding_id, const char *locator)
{
  return make_id(out, out_cap, "sa", source_binding_id, locator);
}

int yai_source_id_acquisition_event(char *out,
                                    size_t out_cap,
                                    const char *source_binding_id,
                                    const char *event_type)
{
  return make_id(out, out_cap, "se", source_binding_id, event_type);
}

int yai_source_id_evidence_candidate(char *out,
                                     size_t out_cap,
                                     const char *source_acquisition_event_id,
                                     const char *candidate_type)
{
  return make_id(out, out_cap, "sc", source_acquisition_event_id, candidate_type);
}

int yai_source_id_action_point(char *out,
                               size_t out_cap,
                               const char *source_binding_id,
                               const char *action_ref)
{
  return make_id(out, out_cap, "sap", source_binding_id, action_ref);
}

int yai_source_id_owner_link(char *out, size_t out_cap, const char *source_node_id, const char *owner_ref)
{
  return make_id(out, out_cap, "sl", source_node_id, owner_ref);
}

int yai_source_id_enrollment_grant(char *out,
                                   size_t out_cap,
                                   const char *source_node_id,
                                   const char *daemon_instance_id)
{
  return make_id(out, out_cap, "sg", source_node_id, daemon_instance_id);
}

int yai_source_id_policy_snapshot(char *out,
                                  size_t out_cap,
                                  const char *source_node_id,
                                  const char *daemon_instance_id,
                                  const char *workspace_id)
{
  char join[224];
  if (!source_node_id || !daemon_instance_id || !workspace_id)
  {
    return -1;
  }
  if (snprintf(join, sizeof(join), "%s:%s", daemon_instance_id, workspace_id) >= (int)sizeof(join))
  {
    return -1;
  }
  return make_id(out, out_cap, "sps", source_node_id, join);
}

int yai_source_id_capability_envelope(char *out,
                                      size_t out_cap,
                                      const char *source_binding_id,
                                      const char *source_node_id,
                                      const char *workspace_id)
{
  char join[224];
  if (!source_binding_id || !source_node_id || !workspace_id)
  {
    return -1;
  }
  if (snprintf(join, sizeof(join), "%s:%s", source_node_id, workspace_id) >= (int)sizeof(join))
  {
    return -1;
  }
  return make_id(out, out_cap, "sce", source_binding_id, join);
}

int yai_source_id_workspace_peer_membership(char *out,
                                            size_t out_cap,
                                            const char *workspace_id,
                                            const char *source_node_id,
                                            const char *source_binding_id)
{
  char join[224];
  if (!workspace_id || !source_node_id || !source_binding_id)
  {
    return -1;
  }
  if (snprintf(join, sizeof(join), "%s:%s", workspace_id, source_binding_id) >= (int)sizeof(join))
  {
    return -1;
  }
  return make_id(out, out_cap, "spm", source_node_id, join);
}
