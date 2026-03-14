/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

typedef enum yai_network_trust_state {
  YAI_NETWORK_TRUST_UNKNOWN = 0,
  YAI_NETWORK_TRUST_ESTABLISHED = 1,
  YAI_NETWORK_TRUST_REVOKED = 2
} yai_network_trust_state_t;
