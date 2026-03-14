#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <yai/dmn/runtime/runtime_config.h>

static char *trim(char *s)
{
  char *end = NULL;
  while (*s && isspace((unsigned char)*s))
  {
    s++;
  }
  if (*s == '\0')
  {
    return s;
  }
  end = s + strlen(s) - 1;
  while (end > s && isspace((unsigned char)*end))
  {
    *end = '\0';
    end--;
  }
  return s;
}

int yai_edge_config_set_string(char *dst, size_t dst_cap, const char *value)
{
  if (!dst || dst_cap == 0 || !value)
  {
    return -1;
  }
  if (snprintf(dst, dst_cap, "%s", value) >= (int)dst_cap)
  {
    return -1;
  }
  return 0;
}

int yai_edge_config_parse_uint(const char *raw, unsigned int *out)
{
  unsigned long v = 0;
  char *end = NULL;
  if (!raw || !out)
  {
    return -1;
  }
  v = strtoul(raw, &end, 10);
  if (end == raw || *end != '\0')
  {
    return -1;
  }
  *out = (unsigned int)v;
  return 0;
}

int yai_edge_config_parse_int(const char *raw, int *out)
{
  long v = 0;
  char *end = NULL;
  if (!raw || !out)
  {
    return -1;
  }
  v = strtol(raw, &end, 10);
  if (end == raw || *end != '\0')
  {
    return -1;
  }
  *out = (int)v;
  return 0;
}

static void apply_kv(yai_edge_config_t *cfg, const char *key, const char *value)
{
  if (!cfg || !key || !value)
  {
    return;
  }
  if (!value[0])
  {
    return;
  }

  if (strcmp(key, "home") == 0)
  {
    (void)yai_edge_config_set_string(cfg->home, sizeof(cfg->home), value);
  }
  else if (strcmp(key, "owner_ref") == 0)
  {
    (void)yai_edge_config_set_string(cfg->owner_ref, sizeof(cfg->owner_ref), value);
  }
  else if (strcmp(key, "source_label") == 0)
  {
    (void)yai_edge_config_set_string(cfg->source_label, sizeof(cfg->source_label), value);
  }
  else if (strcmp(key, "log_level") == 0)
  {
    (void)yai_edge_config_set_string(cfg->log_level, sizeof(cfg->log_level), value);
  }
  else if (strcmp(key, "mode") == 0)
  {
    (void)yai_edge_config_set_string(cfg->mode, sizeof(cfg->mode), value);
  }
  else if (strcmp(key, "bindings_manifest") == 0)
  {
    (void)yai_edge_config_set_string(cfg->bindings_manifest, sizeof(cfg->bindings_manifest), value);
  }
  else if (strcmp(key, "tick_ms") == 0)
  {
    (void)yai_edge_config_parse_uint(value, &cfg->tick_ms);
  }
  else if (strcmp(key, "max_ticks") == 0)
  {
    (void)yai_edge_config_parse_int(value, &cfg->max_ticks);
  }
}

int yai_edge_config_defaults(yai_edge_config_t *cfg)
{
  const char *home = getenv("HOME");
  if (!cfg || !home || !home[0])
  {
    return -1;
  }
  memset(cfg, 0, sizeof(*cfg));
  if (snprintf(cfg->home, sizeof(cfg->home), "%s/.yai/daemon", home) >= (int)sizeof(cfg->home))
  {
    return -1;
  }
  if (snprintf(cfg->config_path, sizeof(cfg->config_path), "%s/config/daemon.env", cfg->home) >=
      (int)sizeof(cfg->config_path))
  {
    return -1;
  }
  if (snprintf(cfg->bindings_manifest,
               sizeof(cfg->bindings_manifest),
               "%s/config/source-bindings.manifest.json",
               cfg->home) >= (int)sizeof(cfg->bindings_manifest))
  {
    return -1;
  }
  (void)yai_edge_config_set_string(cfg->log_level, sizeof(cfg->log_level), "info");
  (void)yai_edge_config_set_string(cfg->mode, sizeof(cfg->mode), "foreground");
  cfg->tick_ms = 1000;
  cfg->max_ticks = 0;
  return 0;
}

int yai_edge_config_apply_env(yai_edge_config_t *cfg)
{
  const char *raw = NULL;
  int home_overridden = 0;
  int config_overridden = 0;
  int manifest_overridden = 0;
  if (!cfg)
  {
    return -1;
  }

  raw = getenv("YAI_DAEMON_HOME");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_HOME");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_set_string(cfg->home, sizeof(cfg->home), raw);
    home_overridden = 1;
  }
  raw = getenv("YAI_DAEMON_CONFIG");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_CONFIG");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_set_string(cfg->config_path, sizeof(cfg->config_path), raw);
    config_overridden = 1;
  }
  raw = getenv("YAI_DAEMON_OWNER_REF");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_OWNER_REF");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_set_string(cfg->owner_ref, sizeof(cfg->owner_ref), raw);
  }
  raw = getenv("YAI_DAEMON_SOURCE_LABEL");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_SOURCE_LABEL");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_set_string(cfg->source_label, sizeof(cfg->source_label), raw);
  }
  raw = getenv("YAI_DAEMON_LOG_LEVEL");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_LOG_LEVEL");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_set_string(cfg->log_level, sizeof(cfg->log_level), raw);
  }
  raw = getenv("YAI_DAEMON_MODE");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_MODE");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_set_string(cfg->mode, sizeof(cfg->mode), raw);
  }
  raw = getenv("YAI_DAEMON_BINDINGS_MANIFEST");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_BINDINGS_MANIFEST");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_set_string(cfg->bindings_manifest, sizeof(cfg->bindings_manifest), raw);
    manifest_overridden = 1;
  }
  raw = getenv("YAI_DAEMON_TICK_MS");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_TICK_MS");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_parse_uint(raw, &cfg->tick_ms);
  }
  raw = getenv("YAI_DAEMON_MAX_TICKS");
  if ((!raw || !raw[0]))
  {
    raw = getenv("YAI_EDGE_MAX_TICKS");
  }
  if (raw && raw[0])
  {
    (void)yai_edge_config_parse_int(raw, &cfg->max_ticks);
  }

  if (cfg->home[0] && (cfg->config_path[0] == '\0' || (home_overridden && !config_overridden)))
  {
    (void)snprintf(cfg->config_path, sizeof(cfg->config_path), "%s/config/daemon.env", cfg->home);
  }
  if (cfg->home[0] && (cfg->bindings_manifest[0] == '\0' || (home_overridden && !manifest_overridden)))
  {
    (void)snprintf(cfg->bindings_manifest,
                   sizeof(cfg->bindings_manifest),
                   "%s/config/source-bindings.manifest.json",
                   cfg->home);
  }
  return 0;
}

int yai_edge_config_apply_file(yai_edge_config_t *cfg, const char *path)
{
  FILE *f = NULL;
  char line[1024];
  if (!cfg || !path || !path[0])
  {
    return -1;
  }
  f = fopen(path, "r");
  if (!f)
  {
    return 0;
  }

  while (fgets(line, sizeof(line), f))
  {
    char *eq = NULL;
    char *key = NULL;
    char *value = NULL;

    key = trim(line);
    if (!key[0] || key[0] == '#')
    {
      continue;
    }
    eq = strchr(key, '=');
    if (!eq)
    {
      continue;
    }
    *eq = '\0';
    value = trim(eq + 1);
    key = trim(key);
    apply_kv(cfg, key, value);
  }
  fclose(f);
  return 0;
}

int yai_edge_config_validate(const yai_edge_config_t *cfg)
{
  if (!cfg)
  {
    return -1;
  }
  if (!cfg->home[0])
  {
    return -2;
  }
  if (!cfg->log_level[0])
  {
    return -3;
  }
  if (!cfg->mode[0])
  {
    return -4;
  }
  if (strcmp(cfg->mode, "foreground") != 0 && strcmp(cfg->mode, "background") != 0)
  {
    return -5;
  }
  if (cfg->tick_ms == 0)
  {
    return -6;
  }
  if (cfg->max_ticks < 0)
  {
    return -7;
  }
  return 0;
}
