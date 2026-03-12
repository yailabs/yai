/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#define YAI_NETWORK_PEER_ID_MAX 96

typedef struct yai_network_peer {
  char peer_id[YAI_NETWORK_PEER_ID_MAX];
  int64_t updated_at_epoch;
} yai_network_peer_t;
