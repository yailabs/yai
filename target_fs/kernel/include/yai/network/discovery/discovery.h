#pragma once

#include <stddef.h>
#include <stdint.h>

typedef struct {
  char peer_id[64];
  char endpoint[128];
  char role[32];
  int connected;
  int64_t observed_at_epoch;
} yai_mesh_peer_entry_t;

typedef struct {
  yai_mesh_peer_entry_t peers[64];
  size_t count;
  struct {
    char node_id[64];
  } self;
} yai_mesh_peer_registry_t;

typedef struct {
  char peer_id[64];
  char endpoint[128];
  char role[32];
  int connected;
  int64_t observed_at_epoch;
} yai_mesh_discovery_event_t;

int yai_mesh_peer_registry_upsert(yai_mesh_peer_registry_t *registry,
                                  const char *peer_id,
                                  const char *endpoint,
                                  const char *role,
                                  int connected,
                                  int64_t observed_at_epoch);

size_t yai_mesh_peer_registry_connected_count(const yai_mesh_peer_registry_t *registry);

int yai_mesh_discovery_apply(yai_mesh_peer_registry_t *registry,
                             const yai_mesh_discovery_event_t *event);
