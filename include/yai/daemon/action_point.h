#pragma once

#include <stddef.h>
#include <stdint.h>

#define YAI_DAEMON_ACTION_POINT_MAX 128
#define YAI_DAEMON_ACTION_POINT_KIND_MAX 64
#define YAI_DAEMON_ACTION_POINT_REF_MAX 256

typedef struct yai_daemon_action_point_descriptor {
  char action_point_id[YAI_DAEMON_ACTION_POINT_MAX];
  char action_kind[YAI_DAEMON_ACTION_POINT_KIND_MAX];
  char action_ref[YAI_DAEMON_ACTION_POINT_REF_MAX];
  char mediation_scope[128];
  char enforcement_scope[128];
  char controllability_state[48];
  int64_t updated_at_epoch;
} yai_daemon_action_point_descriptor_t;

int yai_daemon_action_point_id(char *out,
                               size_t out_cap,
                               const char *source_binding_id,
                               const char *action_ref);

