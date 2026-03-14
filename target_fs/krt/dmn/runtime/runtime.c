#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <yai/dmn/daemon.h>
#include <yai/hal/os.h>
#include <yai/orch/network_bridge.h>

#include "../internal/internal.h"

void yai_edge_log(const yai_edge_runtime_t *rt, const char *level, const char *message)
{
  yai_edge_logf(rt, level, "%s", message ? message : "");
}

void yai_edge_logf(const yai_edge_runtime_t *rt, const char *level, const char *fmt, ...)
{
  va_list ap;
  va_start(ap, fmt);
  yai_edge_vlog(rt ? rt->instance_id : NULL, level, fmt, ap);
  va_end(ap);
}

static int write_identity(const yai_edge_runtime_t *rt)
{
  char payload[1024];
  if (!rt)
  {
    return -1;
  }
  if (snprintf(payload,
               sizeof(payload),
               "{\n"
               "  \"type\": \"yai.daemon.identity.v1\",\n"
               "  \"instance_id\": \"%s\",\n"
               "  \"source_label\": \"%s\",\n"
               "  \"owner_ref\": \"%s\",\n"
               "  \"topology\": \"%s\"\n"
               "}\n",
               rt->instance_id,
               rt->config.source_label,
               rt->config.owner_ref,
               YAI_SOURCE_PLANE_TOPOLOGY) >= (int)sizeof(payload))
  {
    return -1;
  }
  return yai_edge_write_file(rt->paths.instance_file, payload);
}

static int write_edge_runtime_state(const yai_edge_runtime_t *rt)
{
  char payload[8192];
  if (!rt || !rt->edge_state_file[0])
  {
    return -1;
  }
  if (yai_edge_edge_state_json(&rt->edge_state, payload, sizeof(payload)) != 0)
  {
    return -1;
  }
  return yai_edge_write_file(rt->edge_state_file, payload);
}

static int write_pid(const yai_edge_runtime_t *rt)
{
  char payload[64];
  if (!rt)
  {
    return -1;
  }
  if (snprintf(payload, sizeof(payload), "%ld\n", (long)yai_os_getpid()) >= (int)sizeof(payload))
  {
    return -1;
  }
  return yai_edge_write_file(rt->paths.pid_file, payload);
}

int yai_edge_runtime_init(yai_edge_runtime_t *rt, const yai_edge_config_t *cfg)
{
  time_t now = 0;
  if (!rt || !cfg)
  {
    return -1;
  }
  memset(rt, 0, sizeof(*rt));
  rt->config = *cfg;

  if (yai_edge_paths_build(&rt->config, &rt->paths) != 0)
  {
    return -2;
  }
  if (yai_edge_paths_ensure(&rt->paths) != 0)
  {
    return -3;
  }

  now = time(NULL);
  if (snprintf(rt->instance_id,
               sizeof(rt->instance_id),
               "yd-%ld-%ld",
               (long)yai_os_getpid(),
               (long)now) >= (int)sizeof(rt->instance_id))
  {
    return -4;
  }

  if (write_identity(rt) != 0)
  {
    return -5;
  }
  if (snprintf(rt->edge_state_file,
               sizeof(rt->edge_state_file),
               "%s/edge-runtime-state.v1.json",
               rt->paths.state_dir) >= (int)sizeof(rt->edge_state_file))
  {
    return -6;
  }
  if (yai_edge_edge_state_init(&rt->edge_state, &rt->config, &rt->paths, rt->instance_id) != 0)
  {
    return -7;
  }
  (void)yai_edge_edge_state_set_phase(&rt->edge_state, YAI_EDGE_EDGE_PHASE_CONFIG_LOAD);
  (void)yai_edge_edge_state_set_phase(&rt->edge_state, YAI_EDGE_EDGE_PHASE_IDENTITY_INIT);
  (void)yai_edge_edge_state_set_phase(&rt->edge_state, YAI_EDGE_EDGE_PHASE_SCOPE_INIT);
  if (yai_edge_edge_services_init(&rt->services) != 0)
  {
    return -8;
  }

  rt->local = (yai_edge_local_runtime_t *)calloc(1, sizeof(*rt->local));
  if (!rt->local)
  {
    return -9;
  }
  if (yai_edge_local_runtime_init(rt->local,
                                    &rt->config,
                                    &rt->paths,
                                    rt->instance_id,
                                    rt->config.source_label) != 0)
  {
    free(rt->local);
    rt->local = NULL;
    return -10;
  }

  if (yai_edge_edge_state_refresh_from_local(&rt->edge_state, rt->local, 0U) != 0)
  {
    return -11;
  }
  if (write_edge_runtime_state(rt) != 0)
  {
    return -12;
  }

  yai_edge_logf(rt,
                  "info",
                  "bootstrap complete home=%s source=%s owner=%s",
                  rt->paths.home,
                  rt->config.source_label,
                  rt->config.owner_ref[0] ? rt->config.owner_ref : "unset");
  return 0;
}

int yai_edge_runtime_start(yai_edge_runtime_t *rt)
{
  if (!rt)
  {
    return -1;
  }

  /* Guardrail: daemon is edge acquisition only and cannot become owner truth. */
  yai_edge_logf(rt,
                  "info",
                  "starting instance=%s topology=%s mode=%s tick_ms=%u max_ticks=%d",
                  rt->instance_id,
                  YAI_SOURCE_PLANE_TOPOLOGY,
                  rt->config.mode,
                  rt->config.tick_ms,
                  rt->config.max_ticks);

  if (write_pid(rt) != 0)
  {
    return -2;
  }
  (void)yai_edge_edge_state_set_phase(&rt->edge_state, YAI_EDGE_EDGE_PHASE_RUNTIME_START);
  (void)yai_edge_edge_state_set_runtime_status(&rt->edge_state, "starting");

  if (yai_edge_edge_services_start(&rt->services) != 0)
  {
    return -3;
  }
  if (!rt->local || yai_edge_local_runtime_start(rt->local) != 0)
  {
    return -4;
  }

  (void)yai_edge_edge_state_refresh_from_local(&rt->edge_state, rt->local, rt->tick_count);
  (void)yai_edge_edge_state_set_phase(&rt->edge_state, YAI_EDGE_EDGE_PHASE_OBSERVATION_LOOP);
  (void)yai_edge_edge_state_set_runtime_status(&rt->edge_state, "running");
  (void)write_edge_runtime_state(rt);

  rt->running = 1;
  return 0;
}

int yai_edge_runtime_tick(yai_edge_runtime_t *rt)
{
  if (!rt || !rt->running)
  {
    return -1;
  }

  rt->tick_count += 1;
  if (yai_edge_edge_services_tick(&rt->services) != 0)
  {
    return -2;
  }
  if (!rt->local || yai_edge_local_runtime_tick(rt->local, rt->tick_count) != 0)
  {
    return -3;
  }
  (void)yai_edge_edge_state_refresh_from_local(&rt->edge_state, rt->local, rt->tick_count);
  (void)write_edge_runtime_state(rt);
  if ((rt->tick_count % 5U) == 0U)
  {
    yai_edge_logf(rt,
                    "debug",
                    "heartbeat tick=%u phase=%s health=%s connectivity=%s freshness=%s owner_connected=%d",
                    rt->tick_count,
                    rt->edge_state.phase,
                    rt->edge_state.health_state,
                    rt->edge_state.connectivity_state,
                    rt->edge_state.freshness_state,
                    rt->edge_state.owner_connected);
  }
  return 0;
}

int yai_edge_runtime_shutdown(yai_edge_runtime_t *rt)
{
  if (!rt)
  {
    return -1;
  }
  (void)yai_edge_edge_state_set_phase(&rt->edge_state, YAI_EDGE_EDGE_PHASE_SHUTDOWN);
  (void)yai_edge_edge_state_set_runtime_status(&rt->edge_state, "stopping");
  if (rt->local)
  {
    (void)yai_edge_local_runtime_stop(rt->local);
    free(rt->local);
    rt->local = NULL;
  }
  (void)yai_edge_edge_services_stop(&rt->services);
  (void)yai_edge_edge_state_set_phase(&rt->edge_state, YAI_EDGE_EDGE_PHASE_STOPPED);
  (void)yai_edge_edge_state_set_runtime_status(&rt->edge_state, "stopped");
  (void)write_edge_runtime_state(rt);
  rt->running = 0;
  yai_edge_logf(rt, "info", "shutdown complete ticks=%u", rt->tick_count);
  return 0;
}
