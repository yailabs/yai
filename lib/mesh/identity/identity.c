/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/mesh/identity.h>

#include <stdio.h>
#include <string.h>

static int copy_string(char *dst, size_t cap, const char *src)
{
  if (!dst || cap == 0 || !src || !src[0]) return -1;
  if (snprintf(dst, cap, "%s", src) >= (int)cap) return -1;
  return 0;
}

int yai_mesh_identity_init(yai_mesh_identity_t *identity,
                           const char *mesh_id,
                           const char *node_id,
                           const char *node_role,
                           int64_t started_at_epoch)
{
  if (!identity) return -1;
  memset(identity, 0, sizeof(*identity));
  if (copy_string(identity->mesh_id, sizeof(identity->mesh_id), mesh_id) != 0) return -1;
  if (copy_string(identity->node_id, sizeof(identity->node_id), node_id) != 0) return -1;
  if (copy_string(identity->node_role, sizeof(identity->node_role), node_role ? node_role : "peer") != 0) {
    return -1;
  }
  identity->started_at_epoch = started_at_epoch;
  return 0;
}

int yai_mesh_identity_is_valid(const yai_mesh_identity_t *identity)
{
  if (!identity) return 0;
  if (!identity->mesh_id[0] || !identity->node_id[0] || !identity->node_role[0]) return 0;
  return 1;
}
