/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sup/lifecycle.h>
#include <yai/data/binding.h>
#include <yai/orch/transport.h>
#include <yai/cognition/cognition.h>
#include <yai/cognition/memory.h>
#include <yai/net/providers/catalog.h>
#include <yai/pol/policy_state.h>
#include <yai/pol/grants_state.h>
#include <yai/pol/containment_state.h>

#include <stdio.h>
#include <string.h>

static yai_runtime_capability_state_t g_runtime_caps = {0};

static void clear_caps(void)
{
  memset(&g_runtime_caps, 0, sizeof(g_runtime_caps));
}

int yai_runtime_capabilities_bind_workspace(const char *workspace_id,
                                            char *err,
                                            size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "workspace_id_missing");
    return -1;
  }

  if (yai_data_store_binding_attach_scope(workspace_id, err, err_cap) != 0) return -1;
  snprintf(g_runtime_caps.workspace_id, sizeof(g_runtime_caps.workspace_id), "%s", workspace_id);
  return 0;
}

int yai_runtime_capabilities_start(const char *workspace_id,
                                   const char *runtime_name,
                                   int enable_mock_provider,
                                   char *err,
                                   size_t err_cap)
{
  int rc;
  if (err && err_cap > 0) err[0] = '\0';
  if (g_runtime_caps.initialized) return 0;

  clear_caps();
  snprintf(g_runtime_caps.runtime_name,
           sizeof(g_runtime_caps.runtime_name),
           "%s",
           (runtime_name && runtime_name[0]) ? runtime_name : "yai-runtime");
  g_runtime_caps.enable_mock_provider = enable_mock_provider ? 1 : 0;

  rc = yai_data_store_binding_init(err, err_cap);
  if (rc != 0) return rc;

  rc = yai_runtime_capabilities_bind_workspace((workspace_id && workspace_id[0]) ? workspace_id : "user",
                                               err,
                                               err_cap);
  if (rc != 0) return rc;

  rc = yai_exec_transport_start();
  if (rc != YAI_MIND_OK) {
    if (err && err_cap > 0) snprintf(err, err_cap, "transport_init_failed:%d", rc);
    return -1;
  }
  g_runtime_caps.transport_ready = 1;

  rc = yai_knowledge_providers_start();
  if (rc != YAI_MIND_OK) {
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "providers_init_failed:%d", rc);
    return -1;
  }
  g_runtime_caps.providers_ready = 1;

  rc = yai_knowledge_memory_start();
  if (rc != YAI_MIND_OK) {
    (void)yai_knowledge_providers_stop();
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "memory_init_failed:%d", rc);
    return -1;
  }
  g_runtime_caps.memory_ready = 1;

  rc = yai_knowledge_cognition_start();
  if (rc != YAI_MIND_OK) {
    (void)yai_knowledge_memory_stop();
    (void)yai_knowledge_providers_stop();
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "cognition_init_failed:%d", rc);
    return -1;
  }
  g_runtime_caps.cognition_ready = 1;

  rc = yai_runtime_policy_start(g_runtime_caps.workspace_id, err, err_cap);
  if (rc != 0) {
    (void)yai_knowledge_cognition_stop();
    (void)yai_knowledge_memory_stop();
    (void)yai_knowledge_providers_stop();
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "runtime_policy_init_failed:%d", rc);
    return -1;
  }
  rc = yai_runtime_policy_apply_effective("runtime/bootstrap-policy", 1u, 0, "runtime_caps_bootstrap");
  if (rc != 0) {
    (void)yai_runtime_policy_stop();
    (void)yai_knowledge_cognition_stop();
    (void)yai_knowledge_memory_stop();
    (void)yai_knowledge_providers_stop();
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "runtime_policy_apply_failed:%d", rc);
    return -1;
  }
  g_runtime_caps.policy_ready = 1;

  rc = yai_runtime_grants_start(g_runtime_caps.workspace_id, err, err_cap);
  if (rc != 0) {
    (void)yai_runtime_policy_stop();
    (void)yai_knowledge_cognition_stop();
    (void)yai_knowledge_memory_stop();
    (void)yai_knowledge_providers_stop();
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "runtime_grants_init_failed:%d", rc);
    return -1;
  }
  rc = yai_runtime_grants_upsert("runtime-bootstrap-grant",
                                 "workspace/*",
                                 0,
                                 0,
                                 0,
                                 0);
  if (rc != 0 || yai_runtime_grants_refresh(0) != 0) {
    (void)yai_runtime_grants_stop();
    (void)yai_runtime_policy_stop();
    (void)yai_knowledge_cognition_stop();
    (void)yai_knowledge_memory_stop();
    (void)yai_knowledge_providers_stop();
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "runtime_grants_apply_failed:%d", rc);
    return -1;
  }
  g_runtime_caps.grants_ready = 1;

  rc = yai_runtime_containment_start(g_runtime_caps.workspace_id, err, err_cap);
  if (rc != 0 || yai_runtime_containment_evaluate("runtime_caps_bootstrap", 0) != 0) {
    (void)yai_runtime_grants_stop();
    (void)yai_runtime_policy_stop();
    (void)yai_knowledge_cognition_stop();
    (void)yai_knowledge_memory_stop();
    (void)yai_knowledge_providers_stop();
    (void)yai_exec_transport_stop();
    clear_caps();
    if (err && err_cap > 0) snprintf(err, err_cap, "runtime_containment_init_failed:%d", rc);
    return -1;
  }
  g_runtime_caps.containment_ready = 1;

  g_runtime_caps.initialized = 1;
  return 0;
}

int yai_runtime_capabilities_stop(char *err, size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!g_runtime_caps.initialized) return 0;

  (void)yai_knowledge_cognition_stop();
  g_runtime_caps.cognition_ready = 0;

  (void)yai_runtime_containment_stop();
  g_runtime_caps.containment_ready = 0;

  (void)yai_runtime_grants_stop();
  g_runtime_caps.grants_ready = 0;

  (void)yai_runtime_policy_stop();
  g_runtime_caps.policy_ready = 0;

  (void)yai_knowledge_memory_stop();
  g_runtime_caps.memory_ready = 0;

  (void)yai_knowledge_providers_stop();
  g_runtime_caps.providers_ready = 0;

  (void)yai_exec_transport_stop();
  g_runtime_caps.transport_ready = 0;

  g_runtime_caps.initialized = 0;
  g_runtime_caps.workspace_id[0] = '\0';
  return 0;
}

int yai_runtime_capabilities_is_ready(void)
{
  return g_runtime_caps.initialized;
}

const yai_runtime_capability_state_t *yai_runtime_capabilities_state(void)
{
  return &g_runtime_caps;
}
