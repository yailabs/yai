/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

typedef enum yai_network_channel_state {
  YAI_NETWORK_CHANNEL_DOWN = 0,
  YAI_NETWORK_CHANNEL_UP = 1
} yai_network_channel_state_t;

int yai_network_transport_client_ready(void);
int yai_network_transport_server_ready(void);
int yai_network_overlay_transport_ready(void);
