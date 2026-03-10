/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/core/lifecycle.h>
#include <yai/core/workspace.h>
#include <yai/data/binding.h>

#include <stdio.h>

int yai_workspace_bind_runtime_capabilities(const char *workspace_id,
                                            char *err,
                                            size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "workspace_id_missing");
    return -1;
  }

  if (yai_data_store_binding_is_ready() == 0) {
    if (yai_data_store_binding_init(err, err_cap) != 0) return -1;
  }

  if (yai_data_store_binding_attach_workspace(workspace_id, err, err_cap) != 0) return -1;
  return yai_runtime_capabilities_bind_workspace(workspace_id, err, err_cap);
}
