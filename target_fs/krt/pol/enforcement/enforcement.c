#include <yai/pol/enforcement.h>
#include <yai/pol/events.h>
#include <yai/pol/governance/policy_effects.h>
#include <yai/lib/logger.h>
#include <string.h>
#include <stdio.h>

/* ------------------------------------------------------------
   Internal Helpers
   ------------------------------------------------------------ */

static const char *skip_ws(const char *p) {
    while (p && (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n'))
        p++;
    return p;
}

static int extract_string_field_from(
    const char *json,
    const char *key,
    char *out,
    size_t cap)
{
    if (!json || !key || !out || cap < 2)
        return 0;

    const char *p = strstr(json, key);
    if (!p) return 0;

    p += strlen(key);
    p = strchr(p, ':');
    if (!p) return 0;
    p++;

    p = skip_ws(p);

    if (*p != '"') return 0;
    p++;

    const char *end = strchr(p, '"');
    if (!end) return 0;

    size_t len = (size_t)(end - p);
    if (len == 0 || len >= cap)
        return 0;

    memcpy(out, p, len);
    out[len] = '\0';
    return 1;
}

static int extract_bool_field_from(
    const char *json,
    const char *key,
    int *out_bool)
{
    if (!json || !key || !out_bool)
        return 0;

    const char *p = strstr(json, key);
    if (!p) return 0;

    p += strlen(key);
    p = strchr(p, ':');
    if (!p) return 0;
    p++;

    p = skip_ws(p);

    if (strncmp(p, "true", 4) == 0) {
        *out_bool = 1;
        return 1;
    }

    if (strncmp(p, "false", 5) == 0) {
        *out_bool = 0;
        return 1;
    }

    return 0;
}

static int contains_token(const char *s, const char *token) {
    return s && token && strstr(s, token) != NULL;
}

static int is_allowed_type_phase1(const char *t) {
    if (!t || t[0] == '\0') return 0;

    return (strcmp(t, "ping") == 0) ||
           (strcmp(t, "protocol_handshake") == 0) ||
           (strcmp(t, "status") == 0);
}

/* ------------------------------------------------------------
   Public Validation Entry
   ------------------------------------------------------------ */

int yai_validate_envelope_v1(
    const char *line,
    const char *expected_ws,
    char *out_request_type,
    size_t req_cap)
{
    if (!line || !out_request_type || req_cap < 2)
        return YAI_E_BAD_ARG;

    if (!contains_token(line, "\"v\":1") &&
        !contains_token(line, "\"v\": 1"))
        return YAI_E_BAD_VERSION;

    char ws_buf[64] = {0};

    if (!extract_string_field_from(
            line, "\"ws_id\"", ws_buf, sizeof(ws_buf)))
        return YAI_E_MISSING_WS;

    if (expected_ws && expected_ws[0] != '\0') {
        if (strcmp(ws_buf, expected_ws) != 0)
            return YAI_E_WS_MISMATCH;
    }

    char type_buf[64] = {0};

    const char *req_pos = strstr(line, "\"request\"");
    if (req_pos) {
        extract_string_field_from(
            req_pos, "\"type\"", type_buf, sizeof(type_buf));
    }

    if (type_buf[0] == '\0')
        extract_string_field_from(
            line, "\"type\"", type_buf, sizeof(type_buf));

    if (type_buf[0] == '\0')
        return YAI_E_MISSING_TYPE;

    size_t tlen = strlen(type_buf);
    if (tlen + 1 > req_cap)
        return YAI_E_MISSING_TYPE;

    memcpy(out_request_type, type_buf, tlen + 1);

    if (!is_allowed_type_phase1(out_request_type))
        return YAI_E_TYPE_NOT_ALLOWED;

    int arming = 0;

    if (extract_bool_field_from(line, "\"arming\"", &arming) && arming) {
        char role_buf[32] = {0};

        if (!extract_string_field_from(
                line, "\"role\"", role_buf, sizeof(role_buf)))
            return YAI_E_ROLE_REQUIRED;

        if (strcmp(role_buf, "operator") != 0)
            return YAI_E_ROLE_REQUIRED;
    }

    return YAI_E_OK;
}

int yai_enforcement_finalize_control_call(const yai_rpc_envelope_t *env,
                                          const char *workspace_id,
                                          const yai_governance_resolution_output_t *law_out,
                                          const yai_runtime_capability_state_t *caps,
                                          yai_enforcement_decision_t *out,
                                          char *err,
                                          size_t err_cap)
{
    yai_authority_evaluation_t cmd_eval = {0};
    yai_authority_evaluation_t policy_eval = {0};
    const char *effect_name;
    size_t used = 0;
    int i;
    if (err && err_cap > 0) err[0] = '\0';
    if (!env || !workspace_id || !law_out || !out) {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "enforcement_bad_args");
        return -1;
    }

    memset(out, 0, sizeof(*out));
    snprintf(out->status, sizeof(out->status), "%s", "ok");
    snprintf(out->code, sizeof(out->code), "%s", "OK");
    snprintf(out->reason, sizeof(out->reason), "%s", "allow");
    snprintf(out->review_state, sizeof(out->review_state), "%s", "clear");
    out->authority_constraints[0] = '\0';
    out->authority_constraint_count = 0;
    out->authority_decision = YAI_AUTHORITY_ALLOW;
    out->runtime_bound = 0;

    effect_name = yai_governance_effect_name(law_out->decision.final_effect);
    if (!effect_name || !effect_name[0]) effect_name = "deny";

    for (i = 0; i < law_out->decision.authority_requirement_count; i++) {
        int n;
        const char *token = law_out->decision.authority_requirements[i];
        if (!token || !token[0]) continue;
        n = snprintf(out->authority_constraints + used,
                     sizeof(out->authority_constraints) - used,
                     "%s%s",
                     used == 0 ? "" : ",",
                     token);
        if (n <= 0 || (size_t)n >= (sizeof(out->authority_constraints) - used)) break;
        used += (size_t)n;
        out->authority_constraint_count++;
    }

    if (yai_authority_command_gate(NULL,
                                   env->command_id,
                                   env->role,
                                   env->arming,
                                   &cmd_eval,
                                   err,
                                   err_cap) != 0) {
        return -1;
    }
    if (cmd_eval.decision == YAI_AUTHORITY_DENY) {
        out->authority_decision = cmd_eval.decision;
        snprintf(out->status, sizeof(out->status), "%s", "error");
        snprintf(out->code, sizeof(out->code), "%s", "AUTHORITY_BLOCK");
        snprintf(out->reason, sizeof(out->reason), "%s", cmd_eval.reason[0] ? cmd_eval.reason : "authority_block");
        snprintf(out->review_state, sizeof(out->review_state), "%s", "blocked");
        return 0;
    }

    if (yai_authority_policy_gate(workspace_id,
                                  effect_name,
                                  law_out->decision.stack.authority_profile,
                                  law_out->evidence.authority_context,
                                  &policy_eval,
                                  err,
                                  err_cap) != 0) {
        return -1;
    }

    if (caps && caps->initialized &&
        workspace_id[0] &&
        strcmp(workspace_id, "system") != 0 &&
        caps->workspace_id[0] &&
        strcmp(caps->workspace_id, workspace_id) != 0) {
        out->authority_decision = YAI_AUTHORITY_DENY;
        snprintf(out->status, sizeof(out->status), "%s", "error");
        snprintf(out->code, sizeof(out->code), "%s", "WORKSPACE_BIND_MISMATCH");
        snprintf(out->reason, sizeof(out->reason), "%s", "runtime_workspace_mismatch");
        snprintf(out->review_state, sizeof(out->review_state), "%s", "blocked");
        return 0;
    }

    if (caps && caps->initialized && caps->workspace_id[0] && strcmp(caps->workspace_id, "system") != 0) {
        out->runtime_bound = 1;
    }

    out->authority_decision = policy_eval.decision;
    if (policy_eval.decision == YAI_AUTHORITY_DENY) {
        snprintf(out->status, sizeof(out->status), "%s", "error");
        snprintf(out->code, sizeof(out->code), "%s", "POLICY_BLOCK");
        snprintf(out->reason, sizeof(out->reason), "%s", effect_name);
        snprintf(out->review_state, sizeof(out->review_state), "%s", "blocked");
        return 0;
    }
    if (policy_eval.decision == YAI_AUTHORITY_REVIEW_REQUIRED) {
        snprintf(out->status, sizeof(out->status), "%s", "ok");
        snprintf(out->code, sizeof(out->code), "%s", "REVIEW_REQUIRED");
        snprintf(out->reason, sizeof(out->reason), "%s", effect_name);
        snprintf(out->review_state, sizeof(out->review_state), "%s", "pending_review");
        return 0;
    }

    snprintf(out->status, sizeof(out->status), "%s", "ok");
    snprintf(out->code, sizeof(out->code), "%s", "OK");
    snprintf(out->reason, sizeof(out->reason), "%s", effect_name);
    snprintf(out->review_state, sizeof(out->review_state), "%s", "clear");
    return 0;
}

const char *yai_enforcement_invariant_class_name(yai_enforcement_invariant_class_t invariant_class)
{
    switch (invariant_class) {
    case YAI_ENFORCE_INV_AUTHORITY_COMMAND_GATE: return "authority.command_gate";
    case YAI_ENFORCE_INV_AUTHORITY_POLICY_GATE: return "authority.policy_gate";
    case YAI_ENFORCE_INV_RESOLUTION_PRECEDENCE: return "resolution.precedence";
    case YAI_ENFORCE_INV_POLICY_APPLICATION: return "policy.application";
    case YAI_ENFORCE_INV_GRANTS_VALIDITY: return "grants.validity";
    case YAI_ENFORCE_INV_CONTAINMENT_MODE: return "containment.mode";
    case YAI_ENFORCE_INV_PROTOCOL_CONTROL_ENVELOPE: return "protocol.control_envelope";
    case YAI_ENFORCE_INV_REVIEW_ESCALATION: return "review.escalation";
    default: return "unknown";
    }
}

const char *yai_enforcement_outcome_class_name(yai_enforcement_outcome_class_t outcome_class)
{
    switch (outcome_class) {
    case YAI_ENFORCE_OUTCOME_ALLOW: return "allow";
    case YAI_ENFORCE_OUTCOME_REVIEW_REQUIRED: return "review_required";
    case YAI_ENFORCE_OUTCOME_DENY: return "deny";
    case YAI_ENFORCE_OUTCOME_BLOCKED: return "blocked";
    case YAI_ENFORCE_OUTCOME_QUARANTINED: return "quarantined";
    default: return "unknown";
    }
}

yai_enforcement_outcome_class_t yai_enforcement_outcome_from_decision(const yai_enforcement_decision_t *decision)
{
    if (!decision) return YAI_ENFORCE_OUTCOME_DENY;
    if (strcmp(decision->code, "OK") == 0) return YAI_ENFORCE_OUTCOME_ALLOW;
    if (strcmp(decision->code, "REVIEW_REQUIRED") == 0) return YAI_ENFORCE_OUTCOME_REVIEW_REQUIRED;
    if (strcmp(decision->code, "POLICY_BLOCK") == 0 || strcmp(decision->code, "AUTHORITY_BLOCK") == 0) {
        return YAI_ENFORCE_OUTCOME_BLOCKED;
    }
    if (strcmp(decision->code, "CONTAINMENT_QUARANTINE") == 0) return YAI_ENFORCE_OUTCOME_QUARANTINED;
    return YAI_ENFORCE_OUTCOME_DENY;
}
