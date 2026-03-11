#include <yai/orchestration/source_plane.h>
#include <yai/orchestration/network_gate.h>
#include <yai/orchestration/resource_gate.h>
#include <yai/orchestration/transport.h>

#include <stdio.h>
#include <string.h>

int yai_source_plane_owner_canonical(void)
{
  return 1;
}

const char *yai_source_plane_topology_id(void)
{
  return YAI_SOURCE_PLANE_TOPOLOGY;
}

const char *yai_exec_source_plane_route_for_command(const char *command_id)
{
  if (!command_id || strncmp(command_id, "yai.source.", 11) != 0)
  {
    return "not_source_plane";
  }
  return "owner_ingest_v1";
}

int yai_exec_source_plane_prepare(const char *command_id,
                                  yai_source_plane_mediation_state_t *out_state,
                                  char *err,
                                  size_t err_cap)
{
  const char *route = yai_exec_source_plane_route_for_command(command_id);
  int transport_ready = 0;
  int network_ready = 0;
  int resource_ready = 0;
  int storage_ready = 0;

  if (out_state)
  {
    memset(out_state, 0, sizeof(*out_state));
  }
  if (err && err_cap > 0)
  {
    err[0] = '\0';
  }

  if (!command_id || strcmp(route, "not_source_plane") == 0)
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", "source_plane_command_required");
    }
    return -2;
  }

  if (!yai_source_plane_owner_canonical())
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", "source_plane_owner_not_canonical");
    }
    return -3;
  }

  if (!yai_exec_transport_is_ready())
  {
    (void)yai_exec_transport_start();
  }
  transport_ready = yai_exec_transport_is_ready() ? 1 : 0;
  network_ready = yai_exec_network_gate_ready() ? 1 : 0;
  resource_ready = yai_exec_resource_gate_ready() ? 1 : 0;
  /*
   * Storage gate readiness is tracked as an exec mediation capability signal
   * for source-plane v1. Concrete storage gate hardening is deferred to YD-8.
   */
  storage_ready = 1;

  if (out_state)
  {
    out_state->owner_canonical = 1;
    out_state->transport_ready = transport_ready;
    out_state->network_gate_ready = network_ready;
    out_state->resource_gate_ready = resource_ready;
    out_state->storage_gate_ready = storage_ready;
    (void)snprintf(out_state->route, sizeof(out_state->route), "%s", route);
    (void)snprintf(out_state->stage, sizeof(out_state->stage), "%s", "exec.runtime.source_plane_mediation.v1");
  }

  if (!transport_ready)
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", "source_plane_transport_not_ready");
    }
    return -3;
  }
  if (!network_ready)
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", "source_plane_network_gate_closed");
    }
    return -3;
  }
  if (!resource_ready)
  {
    if (err && err_cap > 0)
    {
      (void)snprintf(err, err_cap, "%s", "source_plane_resource_gate_closed");
    }
    return -3;
  }
  return 0;
}
