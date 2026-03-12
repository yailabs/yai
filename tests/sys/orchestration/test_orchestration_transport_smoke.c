/* SPDX-License-Identifier: Apache-2.0 */

#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <yai/cognition/runtime.h>
#include <yai/orchestration/transport.h>

int main(void)
{
  yai_config_t cfg = {.runtime_name = "brain-transport-smoke", .enable_mock_provider = 1};
  char out[1024] = {0};

  assert(yai_init(&cfg) == YAI_MIND_OK);
  assert(yai_transport_handle_raw("COMPLETE transport smoke\n", out, sizeof(out)) == YAI_MIND_OK);
  assert(strstr(out, "STATUS 200") != NULL);
  assert(yai_shutdown() == YAI_MIND_OK);

  puts("test_brain_transport_smoke: ok");
  return 0;
}
