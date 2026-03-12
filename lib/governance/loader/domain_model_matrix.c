#include "../internal.h"

#include <stdlib.h>
#include <string.h>

static int read_domain_model_matrix(char *json, size_t cap) {
  const char *env_file = getenv("YAI_GOVERNANCE_DOMAIN_MODEL");
  const char *candidates[] = {
    "specs/domains/indexes/domain-model.matrix.v1.json",
    "../yai/specs/domains/indexes/domain-model.matrix.v1.json"
  };
  size_t i;

  if (!json || cap == 0) return -1;
  if (env_file && env_file[0] != '\0' && yai_governance_read_text_file(env_file, json, cap) == 0) return 0;

  for (i = 0; i < (sizeof(candidates) / sizeof(candidates[0])); ++i) {
    if (yai_governance_read_text_file(candidates[i], json, cap) == 0) return 0;
  }
  return -1;
}

static const char *find_object_for_lookup(const char *json, const char *lookup_id) {
  char needle[192];
  const char *p;
  if (!json || !lookup_id) return NULL;
  if (yai_governance_safe_snprintf(needle, sizeof(needle), "\"lookup_id\": \"%s\"", lookup_id) != 0) return NULL;
  p = strstr(json, needle);
  if (!p) return NULL;
  while (p > json && *p != '{') p--;
  return (*p == '{') ? p : NULL;
}

static int extract_string_between(const char *start,
                                  const char *limit,
                                  const char *key,
                                  char *out,
                                  size_t out_cap) {
  char needle[96];
  const char *p;
  const char *q;

  if (!start || !limit || !key || !out || out_cap == 0 || start >= limit) return -1;
  if (yai_governance_safe_snprintf(needle, sizeof(needle), "\"%s\"", key) != 0) return -1;
  p = strstr(start, needle);
  if (!p || p >= limit) return -1;
  p = strchr(p, ':');
  if (!p || p >= limit) return -1;
  p++;
  while (p < limit && (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r')) p++;
  if (p >= limit || *p != '"') return -1;
  p++;
  q = strchr(p, '"');
  if (!q || q > limit) return -1;

  {
    size_t len = (size_t)(q - p);
    if (len >= out_cap) len = out_cap - 1;
    memcpy(out, p, len);
    out[len] = '\0';
  }
  return 0;
}

int yai_governance_read_domain_model_matrix(char *out_json, size_t out_cap) {
  return read_domain_model_matrix(out_json, out_cap);
}

int yai_governance_domain_model_lookup(const char *lookup_id,
                                char *kind,
                                size_t kind_cap,
                                char *family,
                                size_t family_cap,
                                char *domain_id,
                                size_t domain_cap,
                                char *manifest_ref,
                                size_t manifest_cap,
                                char *default_specialization,
                                size_t default_spec_cap) {
  char json[32768];
  const char *obj;
  const char *obj_end;

  if (!lookup_id) return -1;
  if (read_domain_model_matrix(json, sizeof(json)) != 0) return -1;

  obj = find_object_for_lookup(json, lookup_id);
  if (!obj) return -1;
  obj_end = strchr(obj, '}');
  if (!obj_end) return -1;

  if (kind && kind_cap > 0) {
    if (extract_string_between(obj, obj_end, "kind", kind, kind_cap) != 0) kind[0] = '\0';
  }
  if (family && family_cap > 0) {
    if (extract_string_between(obj, obj_end, "family", family, family_cap) != 0) family[0] = '\0';
  }
  if (domain_id && domain_cap > 0) {
    if (extract_string_between(obj, obj_end, "domain_id", domain_id, domain_cap) != 0) domain_id[0] = '\0';
  }
  if (manifest_ref && manifest_cap > 0) {
    if (extract_string_between(obj, obj_end, "manifest_ref", manifest_ref, manifest_cap) != 0) manifest_ref[0] = '\0';
  }
  if (default_specialization && default_spec_cap > 0) {
    if (extract_string_between(obj, obj_end, "default_specialization", default_specialization, default_spec_cap) != 0) {
      default_specialization[0] = '\0';
    }
  }
  return 0;
}

int yai_governance_domain_model_runtime_families(char out[][64], int max_out, int *out_count) {
  char json[32768];
  const char *p;
  const char *end;
  int count = 0;

  if (!out || max_out <= 0 || !out_count) return -1;
  *out_count = 0;
  if (read_domain_model_matrix(json, sizeof(json)) != 0) return -1;

  p = strstr(json, "\"runtime_families\"");
  if (!p) return -1;
  p = strchr(p, '[');
  if (!p) return -1;
  end = strchr(p, ']');
  if (!end) return -1;

  while (p < end && count < max_out) {
    const char *q;
    const char *r;
    p = strchr(p, '"');
    if (!p || p >= end) break;
    q = p + 1;
    r = strchr(q, '"');
    if (!r || r > end) break;

    {
      size_t len = (size_t)(r - q);
      if (len >= sizeof(out[count])) len = sizeof(out[count]) - 1;
      memcpy(out[count], q, len);
      out[count][len] = '\0';
    }

    count++;
    p = r + 1;
  }

  *out_count = count;
  return (count > 0) ? 0 : -1;
}

int yai_governance_domain_model_family_resolution(const char *family,
                                           char *domain_id,
                                           size_t domain_cap,
                                           char *default_specialization,
                                           size_t default_spec_cap,
                                           char candidates[][96],
                                           int max_candidates,
                                           int *out_candidate_count) {
  char json[32768];
  char needle[128];
  const char *section;
  const char *entries;
  const char *obj;
  const char *obj_end;
  const char *arr;
  const char *arr_end;
  int count = 0;

  if (!family) return -1;
  if (out_candidate_count) *out_candidate_count = 0;
  if (read_domain_model_matrix(json, sizeof(json)) != 0) return -1;

  section = strstr(json, "\"family_resolution\"");
  if (!section) return -1;
  entries = strstr(section, "\"entries\"");
  if (!entries) entries = json + strlen(json);

  if (yai_governance_safe_snprintf(needle, sizeof(needle), "\"family\": \"%s\"", family) != 0) return -1;
  obj = strstr(section, needle);
  if (!obj || obj >= entries) return -1;
  while (obj > section && *obj != '{') obj--;
  if (*obj != '{') return -1;
  obj_end = strchr(obj, '}');
  if (!obj_end || obj_end >= entries) return -1;

  if (domain_id && domain_cap > 0) {
    if (extract_string_between(obj, obj_end, "domain_id", domain_id, domain_cap) != 0) domain_id[0] = '\0';
  }
  if (default_specialization && default_spec_cap > 0) {
    if (extract_string_between(obj, obj_end, "default_specialization", default_specialization, default_spec_cap) != 0) {
      default_specialization[0] = '\0';
    }
  }

  if (candidates && max_candidates > 0) {
    arr = strstr(obj, "\"specialization_candidates\"");
    if (arr && arr < obj_end) {
      const char *p;
      arr = strchr(arr, '[');
      if (!arr) return 0;
      arr_end = strchr(arr, ']');
      if (!arr_end || arr_end > obj_end) return 0;
      p = arr;

      while (p < arr_end && count < max_candidates) {
        const char *q;
        const char *r;
        p = strchr(p, '"');
        if (!p || p >= arr_end) break;
        q = p + 1;
        r = strchr(q, '"');
        if (!r || r > arr_end) break;

        {
          size_t len = (size_t)(r - q);
          if (len >= sizeof(candidates[count])) len = sizeof(candidates[count]) - 1;
          memcpy(candidates[count], q, len);
          candidates[count][len] = '\0';
        }
        count++;
        p = r + 1;
      }
    }
  }

  if (out_candidate_count) *out_candidate_count = count;
  return 0;
}

int yai_governance_domain_model_specialization_exists(const char *family, const char *specialization_id) {
  char cands[16][96];
  int count = 0;
  int i;

  if (!family || !specialization_id) return 0;
  if (yai_governance_domain_model_family_resolution(family, NULL, 0, NULL, 0, cands, 16, &count) != 0) return 0;
  for (i = 0; i < count; ++i) {
    if (strcmp(cands[i], specialization_id) == 0) return 1;
  }
  return 0;
}
