/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/network/providers/embedding.h>

#include <stddef.h>

int yai_embedder_mock_fill(const char *text, float *vector_out, size_t vector_dim)
{
  size_t seed = 0;
  if (!text || !text[0] || !vector_out || vector_dim == 0) return YAI_MIND_ERR_INVALID_ARG;

  for (const char *p = text; *p; ++p) seed += (unsigned char)(*p);
  for (size_t i = 0; i < vector_dim; i++) {
    vector_out[i] = (float)((seed + i) % 1000U) / 1000.0f;
  }
  return YAI_MIND_OK;
}
