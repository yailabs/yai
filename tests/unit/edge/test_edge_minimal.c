/* SPDX-License-Identifier: Apache-2.0 */

#include <assert.h>
#include <string.h>

#include <yai/edge/action_point.h>
#include <yai/edge/edge_binding.h>
#include <yai/edge/edge_services.h>

static void test_action_point_id(void)
{
  char id[128];
  assert(yai_edge_action_point_id(id, sizeof(id), "binding-a", "action/ref") == 0);
  assert(strncmp(id, "sap-", 4) == 0);
}

static void test_binding_normalization(void)
{
  assert(strcmp(yai_edge_binding_kind_normalize(NULL), YAI_EDGE_BINDING_KIND_OBSERVATIONAL) == 0);
  assert(strcmp(yai_edge_binding_kind_normalize("mediable"), YAI_EDGE_BINDING_KIND_MEDIABLE) == 0);
  assert(yai_edge_binding_is_mediable("mediable") == 1);
  assert(yai_edge_binding_is_mediable("observational") == 0);
  assert(strcmp(yai_edge_mediation_mode_normalize("allow_block_hold", NULL),
                YAI_EDGE_MEDIATION_MODE_ALLOW_BLOCK_HOLD) == 0);
}

static void test_services_lifecycle(void)
{
  yai_edge_edge_services_t services;
  assert(yai_edge_edge_services_init(&services) == 0);
  assert(yai_edge_edge_services_start(&services) == 0);
  assert(services.observation.ready == 1);
  assert(yai_edge_edge_services_tick(&services) == 0);
  assert(services.observation.ticks == 1u);
  assert(services.health.ticks == 1u);
  assert(yai_edge_edge_services_stop(&services) == 0);
  assert(services.observation.ready == 0);
}

int main(void)
{
  test_action_point_id();
  test_binding_normalization();
  test_services_lifecycle();
  return 0;
}
