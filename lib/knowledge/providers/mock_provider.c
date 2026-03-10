/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/providers.h>

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct yai_mind_mock_provider_state {
  int used;
} yai_mind_mock_provider_state_t;

int yai_mind_embedder_mock_fill(const char *text, float *vector_out, size_t vector_dim);

static int mock_completion(struct yai_mind_provider *provider,
                           const yai_mind_provider_request_t *request,
                           yai_mind_provider_response_t *response)
{
  yai_mind_mock_provider_state_t *state;
  if (!provider || !request || !response) return YAI_MIND_ERR_INVALID_ARG;

  state = (yai_mind_mock_provider_state_t *)provider->state;
  if (state) state->used++;

  response->status = 200;
  snprintf(response->output, sizeof(response->output),
           "mock completion for model=%s request=%s",
           request->model,
           request->request_id);
  return YAI_MIND_OK;
}

static int mock_embedding(struct yai_mind_provider *provider,
                          const char *text,
                          float *vector_out,
                          size_t vector_dim)
{
  (void)provider;
  return yai_mind_embedder_mock_fill(text, vector_out, vector_dim);
}

static void mock_destroy(struct yai_mind_provider *provider)
{
  if (!provider) return;
  free(provider->state);
  free(provider);
}

int yai_mind_mock_provider_create(yai_mind_provider_t **provider_out)
{
  static const yai_mind_provider_vtable_t vtable = {
    .completion = mock_completion,
    .embedding = mock_embedding,
    .destroy = mock_destroy,
  };
  yai_mind_provider_t *provider;
  yai_mind_mock_provider_state_t *state;

  if (!provider_out) return YAI_MIND_ERR_INVALID_ARG;

  provider = (yai_mind_provider_t *)calloc(1, sizeof(*provider));
  if (!provider) return YAI_MIND_ERR_NO_MEMORY;

  state = (yai_mind_mock_provider_state_t *)calloc(1, sizeof(*state));
  if (!state) {
    free(provider);
    return YAI_MIND_ERR_NO_MEMORY;
  }

  provider->name = "mock";
  provider->state = state;
  provider->vtable = &vtable;
  *provider_out = provider;
  return YAI_MIND_OK;
}
