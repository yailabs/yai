/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/container/scope_runtime.h>

int yai_workspace_recover_runtime_capabilities(const char *workspace_id,
                                               char *err,
                                               size_t err_cap)
{
  return yai_workspace_bind_runtime_capabilities(workspace_id, err, err_cap);
}
