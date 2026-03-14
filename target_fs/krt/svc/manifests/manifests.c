/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/svc/manifests.h>

#include <stdio.h>

int yai_services_manifests_snapshot(char *out, size_t out_cap)
{
  if (!out || out_cap == 0) return -1;
  return (snprintf(out, out_cap, "{\"manifest_version\":\"v1\"}") < (int)out_cap) ? 0 : -1;
}
