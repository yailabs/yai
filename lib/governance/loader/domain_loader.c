#include "../internal.h"

#include <stdlib.h>
#include <string.h>

static const char *yai_governance_find_family_entry(const char *json, const char *canonical_name) {
  char needle[160];
  const char *p;
  if (!json || !canonical_name) return NULL;
  if (yai_governance_safe_snprintf(needle, sizeof(needle), "\"canonical_name\": \"%s\"", canonical_name) != 0) return NULL;
  p = strstr(json, needle);
  if (!p) return NULL;
  while (p > json && *p != '{') p--;
  return (*p == '{') ? p : NULL;
}

static const char *yai_governance_find_entry_by_key(const char *json, const char *key, const char *value) {
  char needle[192];
  const char *p;
  if (!json || !key || !value) return NULL;
  if (yai_governance_safe_snprintf(needle, sizeof(needle), "\"%s\": \"%s\"", key, value) != 0) return NULL;
  p = strstr(json, needle);
  if (!p) return NULL;
  while (p > json && *p != '{') p--;
  return (*p == '{') ? p : NULL;
}

static int yai_governance_extract_entry_string(const char *obj_start,
                                        const char *key,
                                        char *out,
                                        size_t out_cap) {
  char needle[96];
  const char *p;
  const char *q;
  if (!obj_start || !key || !out || out_cap == 0) return -1;
  if (yai_governance_safe_snprintf(needle, sizeof(needle), "\"%s\"", key) != 0) return -1;
  p = strstr(obj_start, needle);
  if (!p) return -1;
  p = strchr(p, ':');
  if (!p) return -1;
  p++;
  while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;
  if (*p != '"') return -1;
  p++;
  q = strchr(p, '"');
  if (!q) return -1;
  {
    size_t len = (size_t)(q - p);
    if (len >= out_cap) len = out_cap - 1;
    memcpy(out, p, len);
    out[len] = '\0';
  }
  return 0;
}

int yai_governance_load_control_family_descriptor(const yai_governance_runtime_t *rt,
                                           const char *lookup_id,
                                           char *out_json,
                                           size_t out_cap) {
  char family[96];
  char index_json[16384];
  char descriptor_ref[256];
  const char *entry;
  if (!rt || !lookup_id || !out_json || out_cap == 0) return -1;

  family[0] = '\0';
  if (yai_governance_domain_model_lookup(lookup_id,
                                  NULL,
                                  0,
                                  family,
                                  sizeof(family),
                                  NULL,
                                  0,
                                  NULL,
                                  0,
                                  NULL,
                                  0) != 0 ||
      family[0] == '\0') {
    (void)yai_governance_safe_snprintf(family, sizeof(family), "%s", lookup_id);
  }

  if (yai_governance_read_governance_surface_file(rt,
                                           "families/index/families.descriptors.index.json",
                                           index_json,
                                           sizeof(index_json)) != 0) {
    return -1;
  }

  entry = yai_governance_find_family_entry(index_json, family);
  if (!entry) return -1;

  descriptor_ref[0] = '\0';
  if (yai_governance_extract_entry_string(entry, "descriptor_ref", descriptor_ref, sizeof(descriptor_ref)) != 0 ||
      descriptor_ref[0] == '\0') {
    return -1;
  }

  return yai_governance_read_governance_surface_file(rt, descriptor_ref, out_json, out_cap);
}

int yai_governance_load_specialization_descriptor(const yai_governance_runtime_t *rt,
                                           const char *lookup_id,
                                           char *out_json,
                                           size_t out_cap) {
  char kind[32];
  char specialization[96];
  char index_json[32768];
  char descriptor_ref[256];
  const char *entry;

  if (!rt || !lookup_id || !out_json || out_cap == 0) return -1;

  kind[0] = '\0';
  specialization[0] = '\0';
  if (yai_governance_domain_model_lookup(lookup_id,
                                  kind,
                                  sizeof(kind),
                                  NULL,
                                  0,
                                  NULL,
                                  0,
                                  NULL,
                                  0,
                                  NULL,
                                  0) == 0 &&
      strcmp(kind, "specialization") == 0) {
    (void)yai_governance_safe_snprintf(specialization, sizeof(specialization), "%s", lookup_id);
  } else {
    /* Accept canonical_name lookups too (e.g. economic.payments). */
    const char *dot = strrchr(lookup_id, '.');
    if (dot && dot[1] != '\0') {
      (void)yai_governance_safe_snprintf(specialization, sizeof(specialization), "%s", dot + 1);
    } else {
      (void)yai_governance_safe_snprintf(specialization, sizeof(specialization), "%s", lookup_id);
    }
  }

  if (yai_governance_read_governance_surface_file(rt,
                                           "specializations/index/specializations.descriptors.index.json",
                                           index_json,
                                           sizeof(index_json)) != 0) {
    return -1;
  }

  entry = yai_governance_find_entry_by_key(index_json, "specialization_id", specialization);
  if (!entry) {
    entry = yai_governance_find_entry_by_key(index_json, "canonical_name", lookup_id);
  }
  if (!entry) return -1;

  descriptor_ref[0] = '\0';
  if (yai_governance_extract_entry_string(entry, "descriptor_ref", descriptor_ref, sizeof(descriptor_ref)) != 0 ||
      descriptor_ref[0] == '\0') {
    return -1;
  }

  return yai_governance_read_governance_surface_file(rt, descriptor_ref, out_json, out_cap);
}

int yai_governance_load_domain_manifest(const yai_governance_runtime_t *rt,
                                 const char *domain_id,
                                 char *out_json,
                                 size_t out_cap) {
  char path[512];
  char manifest_ref[256];
  int rc;
  const char *roots_primary[] = {"domains"};
  const char *legacy_seed = "transitional/domain-family-seed";
  const char *allow_seed = getenv("YAI_GOVERNANCE_ENABLE_TRANSITIONAL_SEED");
  if (!allow_seed || allow_seed[0] == '\0') {
    /* Backward-compatible alias during migration cleanup. */
    allow_seed = getenv("YAI_GOVERNANCE_ENABLE_TRANSITIONAL_SEED");
  }
  size_t i;
  if (!rt || !domain_id || !out_json || out_cap == 0) return -1;

  /* Canonical lookup: index-driven domain model matrix. */
  manifest_ref[0] = '\0';
  if (yai_governance_domain_model_lookup(domain_id,
                                  NULL,
                                  0,
                                  NULL,
                                  0,
                                  NULL,
                                  0,
                                  manifest_ref,
                                  sizeof(manifest_ref),
                                  NULL,
                                  0) == 0 &&
      manifest_ref[0] != '\0') {
    if (yai_governance_safe_snprintf(path, sizeof(path), "%s/%s", rt->root, manifest_ref) == 0) {
      rc = yai_governance_read_text_file(path, out_json, out_cap);
      if (rc == 0) return 0;
    }
    /* Governance-root fallback for post-absorption canonical content. */
    if (yai_governance_safe_snprintf(path, sizeof(path), "governance/%s", manifest_ref) == 0) {
      rc = yai_governance_read_text_file(path, out_json, out_cap);
      if (rc == 0) return 0;
    }
  }

  /* Compatibility fallback: canonical materialized specializations + domains. */
  if (yai_governance_safe_snprintf(path, sizeof(path), "%s/specializations/materialized/%s/manifest.json", rt->root, domain_id) != 0) {
    return -1;
  }
  rc = yai_governance_read_text_file(path, out_json, out_cap);
  if (rc == 0) return 0;

  for (i = 0; i < (sizeof(roots_primary) / sizeof(roots_primary[0])); ++i) {
    if (yai_governance_safe_snprintf(path, sizeof(path), "%s/%s/%s/manifest.json", rt->root, roots_primary[i], domain_id) != 0) {
      return -1;
    }
    rc = yai_governance_read_text_file(path, out_json, out_cap);
    if (rc == 0) return 0;
  }
  /* Transitional seed is opt-in only and never part of default runtime lookup. */
  if (allow_seed && strcmp(allow_seed, "1") == 0) {
    if (yai_governance_safe_snprintf(path, sizeof(path), "%s/%s/%s/manifest.json", rt->root, legacy_seed, domain_id) != 0) {
      return -1;
    }
    rc = yai_governance_read_text_file(path, out_json, out_cap);
    if (rc == 0) return 0;
  }
  return -1;
}
