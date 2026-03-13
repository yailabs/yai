#pragma once

#include <stdint.h>

typedef struct {
  int initialized;
  char overlay_id[64];
  char local_node_id[64];
  char transport_mode[32];
  int active;
  int64_t updated_at_epoch;
} yai_overlay_transport_state_t;

int yai_overlay_transport_init(yai_overlay_transport_state_t *state,
                               const char *overlay_id,
                               const char *local_node_id,
                               const char *transport_mode,
                               int64_t now_epoch);

int yai_overlay_transport_set_active(yai_overlay_transport_state_t *state,
                                     int active,
                                     int64_t now_epoch);
