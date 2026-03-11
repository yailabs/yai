/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_MESH_ID_MAX 96
#define YAI_MESH_NODE_ID_MAX 96
#define YAI_MESH_ROLE_MAX 32

typedef struct yai_mesh_identity {
  char mesh_id[YAI_MESH_ID_MAX];
  char node_id[YAI_MESH_NODE_ID_MAX];
  char node_role[YAI_MESH_ROLE_MAX];
  int64_t started_at_epoch;
} yai_mesh_identity_t;

int yai_mesh_identity_init(yai_mesh_identity_t *identity,
                           const char *mesh_id,
                           const char *node_id,
                           const char *node_role,
                           int64_t started_at_epoch);
int yai_mesh_identity_is_valid(const yai_mesh_identity_t *identity);

#ifdef __cplusplus
}
#endif
