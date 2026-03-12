/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

typedef struct yai_network_authority_binding {
  char authority_id[96];
  char binding_id[96];
  int64_t issued_at_epoch;
} yai_network_authority_binding_t;
