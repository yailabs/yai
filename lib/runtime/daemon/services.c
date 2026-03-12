#include <stdio.h>
#include <string.h>

#include <yai/runtime/daemon/services.h>

static void service_slot_init(yai_edge_service_slot_t *slot, const char *name)
{
  if (!slot)
  {
    return;
  }
  memset(slot, 0, sizeof(*slot));
  slot->name = name;
  slot->enabled = 1;
  slot->ready = 0;
  (void)snprintf(slot->mode, sizeof(slot->mode), "%s", "placeholder");
}

static void service_slot_start(yai_edge_service_slot_t *slot)
{
  if (!slot || !slot->enabled)
  {
    return;
  }
  slot->ready = 1;
  slot->ticks = 0;
}

static void service_slot_tick(yai_edge_service_slot_t *slot)
{
  if (!slot || !slot->enabled || !slot->ready)
  {
    return;
  }
  slot->ticks += 1U;
}

static void service_slot_stop(yai_edge_service_slot_t *slot)
{
  if (!slot)
  {
    return;
  }
  slot->ready = 0;
}

int yai_edge_edge_services_init(yai_edge_edge_services_t *services)
{
  if (!services)
  {
    return -1;
  }
  memset(services, 0, sizeof(*services));
  service_slot_init(&services->observation, "observation");
  service_slot_init(&services->state, "state");
  service_slot_init(&services->delegated_policy, "delegated_policy");
  service_slot_init(&services->mediation, "mediation");
  service_slot_init(&services->spool_retry, "spool_retry");
  service_slot_init(&services->health, "health");
  service_slot_init(&services->owner_link, "owner_link");
  return 0;
}

int yai_edge_edge_services_start(yai_edge_edge_services_t *services)
{
  if (!services)
  {
    return -1;
  }
  service_slot_start(&services->observation);
  service_slot_start(&services->state);
  service_slot_start(&services->delegated_policy);
  service_slot_start(&services->mediation);
  service_slot_start(&services->spool_retry);
  service_slot_start(&services->health);
  service_slot_start(&services->owner_link);
  return 0;
}

int yai_edge_edge_services_tick(yai_edge_edge_services_t *services)
{
  if (!services)
  {
    return -1;
  }
  service_slot_tick(&services->observation);
  service_slot_tick(&services->state);
  service_slot_tick(&services->delegated_policy);
  service_slot_tick(&services->mediation);
  service_slot_tick(&services->spool_retry);
  service_slot_tick(&services->health);
  service_slot_tick(&services->owner_link);
  return 0;
}

int yai_edge_edge_services_stop(yai_edge_edge_services_t *services)
{
  if (!services)
  {
    return -1;
  }
  service_slot_stop(&services->observation);
  service_slot_stop(&services->state);
  service_slot_stop(&services->delegated_policy);
  service_slot_stop(&services->mediation);
  service_slot_stop(&services->spool_retry);
  service_slot_stop(&services->health);
  service_slot_stop(&services->owner_link);
  return 0;
}
