#include <yai/kernel/authority.h>
#include <stdio.h>
#include <string.h>

static int str_eq(const char *a, const char *b)
{
  return a && b && strcmp(a, b) == 0;
}

int yai_authority_policy_gate(const char *workspace_id,
                              const char *effect_name,
                              const char *authority_profile,
                              const char *authority_context,
                              yai_authority_evaluation_t *out,
                              char *err,
                              size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!out) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "authority_eval_missing");
    return -1;
  }

  memset(out, 0, sizeof(*out));
  out->decision = YAI_AUTHORITY_ALLOW;
  out->command_class = 0;
  out->operator_armed = 1;
  out->authority_lock = 0;

  if (!workspace_id || !workspace_id[0]) {
    out->decision = YAI_AUTHORITY_DENY;
    snprintf(out->reason, sizeof(out->reason), "%s", "workspace_missing");
    return 0;
  }
  if (!effect_name || !effect_name[0]) {
    out->decision = YAI_AUTHORITY_DENY;
    snprintf(out->reason, sizeof(out->reason), "%s", "effect_missing");
    return 0;
  }

  if (str_eq(effect_name, "deny") || str_eq(effect_name, "quarantine")) {
    out->decision = YAI_AUTHORITY_DENY;
    snprintf(out->reason, sizeof(out->reason), "%s", effect_name);
    return 0;
  }
  if (str_eq(effect_name, "review_required")) {
    out->decision = YAI_AUTHORITY_REVIEW_REQUIRED;
    snprintf(out->reason, sizeof(out->reason), "%s", "policy_review_required");
    return 0;
  }

  if (authority_context && strstr(authority_context, "missing_contract") != NULL) {
    out->decision = YAI_AUTHORITY_REVIEW_REQUIRED;
    snprintf(out->reason, sizeof(out->reason), "%s", "authority_contract_missing");
    return 0;
  }
  if (!authority_profile || !authority_profile[0] || strstr(authority_profile, "baseline_authority") != NULL) {
    out->decision = YAI_AUTHORITY_REVIEW_REQUIRED;
    snprintf(out->reason, sizeof(out->reason), "%s", "authority_profile_baseline");
    return 0;
  }

  snprintf(out->reason, sizeof(out->reason), "%s", "authority_profile_verified");
  return 0;
}
