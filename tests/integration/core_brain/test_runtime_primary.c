/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/runtime.h>
#include <yai/exec/transport.h>

#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void)
{
  yai_mind_config_t cfg = {.runtime_name = "runtime-primary", .enable_mock_provider = 1};
  const yai_mind_runtime_t *rt;
  char out[1024] = {0};

  assert(yai_mind_init(&cfg) == YAI_MIND_OK);
  rt = yai_mind_runtime_state();
  assert(rt != NULL);
  assert(rt->initialized == 1);
  assert(rt->transport_ready == 1);
  assert(rt->providers_ready == 1);
  assert(rt->memory_ready == 1);
  assert(rt->cognition_ready == 1);

  assert(yai_mind_transport_handle_raw("COGNITION validate runtime switch\n", out, sizeof(out)) == YAI_MIND_OK);
  assert(strstr(out, "STATUS 200") != NULL);

  assert(yai_mind_shutdown() == YAI_MIND_OK);
  rt = yai_mind_runtime_state();
  assert(rt->initialized == 0);

  puts("test_runtime_primary: ok");
  return 0;
}
