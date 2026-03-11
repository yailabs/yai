/* SPDX-License-Identifier: Apache-2.0 */

#include <assert.h>
#include <string.h>

#include <yai/runtime/policy.h>
#include <yai/runtime/grants.h>
#include <yai/runtime/containment.h>

static void test_policy_runtime_state(void)
{
  yai_runtime_policy_state_t state;
  char json[1024];

  assert(yai_runtime_policy_state_init(&state, "ws-runtime") == 0);
  assert(state.active == 0);
  assert(yai_runtime_policy_state_apply(&state,
                                        "snapshot.runtime.v1",
                                        3u,
                                        1111,
                                        "unit_apply") == 0);
  assert(state.active == 1);
  assert(state.effect_count == 3u);
  assert(strcmp(state.mode, "active") == 0);
  assert(yai_runtime_policy_state_json(&state, json, sizeof(json)) == 0);
  assert(strstr(json, "snapshot.runtime.v1") != NULL);
}

static void test_grants_runtime_state(void)
{
  yai_runtime_grants_state_t grants;
  char json[1024];

  assert(yai_runtime_grants_state_init(&grants, "ws-runtime") == 0);
  assert(yai_runtime_grants_state_upsert(&grants,
                                         "grant-1",
                                         "workspace/read",
                                         1000,
                                         0,
                                         0,
                                         0) == 0);
  assert(yai_runtime_grants_state_upsert(&grants,
                                         "grant-2",
                                         "workspace/write",
                                         1000,
                                         0,
                                         1001,
                                         0) == 0);
  assert(yai_runtime_grants_state_refresh_validity(&grants, 1002) == 0);
  assert(grants.valid_count == 1u);
  assert(yai_runtime_grants_state_has_valid_scope(&grants, "workspace/read") == 1);
  assert(yai_runtime_grants_state_has_valid_scope(&grants, "workspace/write") == 0);
  assert(yai_runtime_grants_state_json(&grants, json, sizeof(json)) == 0);
  assert(strstr(json, "\"valid_count\": 1") != NULL);
}

static void test_containment_policy_grants_coherence(void)
{
  yai_runtime_policy_state_t policy;
  yai_runtime_grants_state_t grants;
  yai_runtime_containment_state_t containment;

  assert(yai_runtime_policy_state_init(&policy, "ws-runtime") == 0);
  assert(yai_runtime_grants_state_init(&grants, "ws-runtime") == 0);
  assert(yai_runtime_containment_state_init(&containment, "ws-runtime") == 0);

  /* No active policy yet -> observe-only containment. */
  assert(yai_runtime_containment_state_apply(&containment,
                                             &policy,
                                             &grants,
                                             "pre_policy",
                                             2000) == 0);
  assert(strcmp(containment.mode, YAI_RUNTIME_CONTAINMENT_MODE_OBSERVE_ONLY) == 0);
  assert(yai_runtime_containment_state_allows(&containment, "inspect") == 1);
  assert(yai_runtime_containment_state_allows(&containment, "write") == 0);

  /* Active policy but no valid grants -> restricted containment. */
  assert(yai_runtime_policy_state_apply(&policy,
                                        "snapshot.runtime.v2",
                                        2u,
                                        2001,
                                        "activate_policy") == 0);
  assert(yai_runtime_containment_state_apply(&containment,
                                             &policy,
                                             &grants,
                                             "policy_no_grants",
                                             2002) == 0);
  assert(strcmp(containment.mode, YAI_RUNTIME_CONTAINMENT_MODE_RESTRICTED) == 0);
  assert(yai_runtime_containment_state_allows(&containment, "read") == 1);
  assert(yai_runtime_containment_state_allows(&containment, "write") == 0);

  /* Active policy + valid grants -> normal containment. */
  assert(yai_runtime_grants_state_upsert(&grants,
                                         "grant-runtime",
                                         "workspace/*",
                                         2000,
                                         0,
                                         0,
                                         0) == 0);
  assert(yai_runtime_grants_state_refresh_validity(&grants, 2003) == 0);
  assert(yai_runtime_containment_state_apply(&containment,
                                             &policy,
                                             &grants,
                                             "policy_and_grants",
                                             2004) == 0);
  assert(strcmp(containment.mode, YAI_RUNTIME_CONTAINMENT_MODE_NORMAL) == 0);
  assert(yai_runtime_containment_state_allows(&containment, "write") == 1);
}

int main(void)
{
  test_policy_runtime_state();
  test_grants_runtime_state();
  test_containment_policy_grants_coherence();
  return 0;
}
