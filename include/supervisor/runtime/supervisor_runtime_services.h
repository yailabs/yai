#pragma once

#include <stdint.h>

typedef struct yai_edge_service_slot {
  const char *name;
  int enabled;
  int ready;
  uint64_t ticks;
  char mode[32];
} yai_edge_service_slot_t;

typedef struct yai_edge_edge_services {
  yai_edge_service_slot_t observation;
  yai_edge_service_slot_t state;
  yai_edge_service_slot_t delegated_policy;
  yai_edge_service_slot_t mediation;
  yai_edge_service_slot_t spool_retry;
  yai_edge_service_slot_t health;
  yai_edge_service_slot_t owner_link;
} yai_edge_edge_services_t;

int yai_edge_edge_services_init(yai_edge_edge_services_t *services);
int yai_edge_edge_services_start(yai_edge_edge_services_t *services);
int yai_edge_edge_services_tick(yai_edge_edge_services_t *services);
int yai_edge_edge_services_stop(yai_edge_edge_services_t *services);

