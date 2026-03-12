/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/providers/catalog.h>

#include <assert.h>
#include <string.h>

static int stub_completion(struct yai_provider *provider,
                           const yai_provider_request_t *request,
                           yai_provider_response_t *response)
{
  (void)provider;
  (void)request;
  (void)response;
  return YAI_MIND_OK;
}

static int stub_embedding(struct yai_provider *provider,
                          const char *text,
                          float *vector_out,
                          size_t vector_dim)
{
  (void)provider;
  (void)text;
  (void)vector_out;
  (void)vector_dim;
  return YAI_MIND_OK;
}

static const yai_provider_vtable_t g_vtable = {
  .completion = stub_completion,
  .embedding = stub_embedding,
  .destroy = NULL,
};

static void test_registry_descriptors_and_lookup(void)
{
  yai_provider_registry_t registry;
  yai_provider_t trusted = {.name = "trusted-inference", .state = NULL, .vtable = &g_vtable};
  yai_provider_t mock = {.name = "mock-embed", .state = NULL, .vtable = &g_vtable};
  yai_provider_descriptor_t trusted_desc = {
    .provider_id = "trusted-inference",
    .provider_class = YAI_PROVIDER_CLASS_INFERENCE,
    .capability_mask = YAI_PROVIDER_CAPABILITY_INFERENCE,
    .trust_level = YAI_PROVIDER_TRUST_TRUSTED,
    .is_mock = 0,
    .invocation_mode = "sync",
  };
  yai_provider_descriptor_t mock_desc = {
    .provider_id = "mock-embed",
    .provider_class = YAI_PROVIDER_CLASS_EMBEDDING,
    .capability_mask = YAI_PROVIDER_CAPABILITY_EMBEDDING,
    .trust_level = YAI_PROVIDER_TRUST_SANDBOXED,
    .is_mock = 1,
    .invocation_mode = "sync",
  };

  assert(yai_provider_registry_init(&registry) == YAI_MIND_OK);
  assert(yai_provider_registry_register_with_descriptor(&registry, &trusted, &trusted_desc, 1) == YAI_MIND_OK);
  assert(yai_provider_registry_register_with_descriptor(&registry, &mock, &mock_desc, 0) == YAI_MIND_OK);

  assert(yai_provider_registry_get(&registry, "trusted-inference") == &trusted);
  assert(yai_provider_registry_default(&registry) == &trusted);

  {
    const yai_provider_descriptor_t *desc = yai_provider_registry_get_descriptor(&registry, "mock-embed");
    assert(desc != NULL);
    assert(desc->is_mock == 1);
    assert(desc->capability_mask == YAI_PROVIDER_CAPABILITY_EMBEDDING);
  }

  yai_provider_registry_shutdown(&registry);
}

static void test_selection_and_policy(void)
{
  yai_provider_registry_t registry;
  yai_provider_t trusted = {.name = "trusted-inference", .state = NULL, .vtable = &g_vtable};
  yai_provider_t mock = {.name = "mock-embed", .state = NULL, .vtable = &g_vtable};
  yai_provider_descriptor_t trusted_desc = {
    .provider_id = "trusted-inference",
    .provider_class = YAI_PROVIDER_CLASS_INFERENCE,
    .capability_mask = YAI_PROVIDER_CAPABILITY_INFERENCE,
    .trust_level = YAI_PROVIDER_TRUST_TRUSTED,
    .is_mock = 0,
    .invocation_mode = "sync",
  };
  yai_provider_descriptor_t mock_desc = {
    .provider_id = "mock-embed",
    .provider_class = YAI_PROVIDER_CLASS_EMBEDDING,
    .capability_mask = YAI_PROVIDER_CAPABILITY_EMBEDDING,
    .trust_level = YAI_PROVIDER_TRUST_SANDBOXED,
    .is_mock = 1,
    .invocation_mode = "sync",
  };
  yai_provider_selection_request_t request = {0};
  yai_provider_policy_t policy = {0};
  yai_provider_t *resolved = NULL;

  assert(yai_provider_registry_init(&registry) == YAI_MIND_OK);
  assert(yai_provider_registry_register_with_descriptor(&registry, &trusted, &trusted_desc, 1) == YAI_MIND_OK);
  assert(yai_provider_registry_register_with_descriptor(&registry, &mock, &mock_desc, 0) == YAI_MIND_OK);

  yai_provider_policy_default(&policy);
  assert(yai_provider_set_policy(&policy) == YAI_MIND_OK);

  request.capability = YAI_PROVIDER_CAPABILITY_INFERENCE;
  request.min_trust = YAI_PROVIDER_TRUST_SANDBOXED;
  assert(yai_provider_select(&registry, &request, NULL, &resolved) == YAI_MIND_OK);
  assert(resolved == &trusted);

  request.provider_name = "mock-embed";
  request.capability = YAI_PROVIDER_CAPABILITY_EMBEDDING;
  request.min_trust = YAI_PROVIDER_TRUST_UNTRUSTED;
  assert(yai_provider_select(&registry, &request, NULL, &resolved) == YAI_MIND_OK);
  assert(resolved == &mock);

  policy.allow_mock_providers = 0;
  assert(yai_provider_set_policy(&policy) == YAI_MIND_OK);
  assert(yai_provider_select(&registry, &request, NULL, &resolved) == YAI_MIND_ERR_PROVIDER);

  yai_provider_policy_default(&policy);
  assert(yai_provider_set_policy(&policy) == YAI_MIND_OK);
  yai_provider_registry_shutdown(&registry);
}

int main(void)
{
  test_registry_descriptors_and_lookup();
  test_selection_and_policy();
  return 0;
}
