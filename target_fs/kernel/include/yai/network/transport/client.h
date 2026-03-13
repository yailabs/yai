#pragma once

#include <stddef.h>
#include <stdint.h>

#include <yai/network/discovery/discovery.h>

typedef struct {
  int initialized;
  uint32_t path_count;
  uint32_t degraded_paths;
  char reachability_state[32];
  int64_t updated_at_epoch;
} yai_mesh_transport_state_t;

typedef struct {
  int initialized;
  char endpoint[256];
  char transport_kind[32];
  int connected;
} yai_network_transport_client_t;

int yai_mesh_transport_refresh(yai_mesh_transport_state_t *transport,
                               const yai_mesh_peer_registry_t *registry,
                               int64_t now_epoch);

int yai_network_transport_client_init(yai_network_transport_client_t *client,
                                      const char *endpoint,
                                      const char *transport_kind);

int yai_network_transport_client_send(yai_network_transport_client_t *client,
                                      const void *payload,
                                      size_t payload_len);

int yai_network_transport_client_close(yai_network_transport_client_t *client);
