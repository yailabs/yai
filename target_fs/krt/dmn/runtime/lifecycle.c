#define _POSIX_C_SOURCE 200809L

#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

#include <yai/dmn/lifecycle.h>

static volatile sig_atomic_t g_stop_requested = 0;

static void on_signal(int sig)
{
  (void)sig;
  g_stop_requested = 1;
}

int yai_edge_lifecycle_install_signals(void)
{
  struct sigaction sa;
  memset(&sa, 0, sizeof(sa));
  sa.sa_handler = on_signal;
  sigemptyset(&sa.sa_mask);
  if (sigaction(SIGINT, &sa, NULL) != 0)
  {
    return -1;
  }
  if (sigaction(SIGTERM, &sa, NULL) != 0)
  {
    return -1;
  }
  return 0;
}

int yai_edge_lifecycle_should_stop(void)
{
  return g_stop_requested ? 1 : 0;
}

int yai_edge_lifecycle_run_foreground(yai_edge_runtime_t *rt)
{
  struct timespec req;
  int max_ticks = 0;
  if (!rt)
  {
    return -1;
  }
  max_ticks = rt->config.max_ticks;

  req.tv_sec = rt->config.tick_ms / 1000U;
  req.tv_nsec = (long)(rt->config.tick_ms % 1000U) * 1000000L;

  while (!yai_edge_lifecycle_should_stop())
  {
    if (yai_edge_runtime_tick(rt) != 0)
    {
      return -2;
    }
    if (max_ticks > 0 && (int)rt->tick_count >= max_ticks)
    {
      return 0;
    }
    nanosleep(&req, NULL);
  }
  return 0;
}
