#define _POSIX_C_SOURCE 200809L

#include <yai/core/session.h>
#include "yai_session_internal.h"
#include <yai/core/workspace.h>
#include <yai/law/policy_effects.h>
#include "cJSON.h"

#include <dirent.h>
#include <errno.h>
#include <ctype.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

#define YAI_WS_JSON_IO_CAP 262144

#define YAI_MANAGED_BEGIN "# BEGIN YAI MANAGED SHELL INTEGRATION"
#define YAI_MANAGED_END   "# END YAI MANAGED SHELL INTEGRATION"
#define YAI_POLICY_ATTACHMENTS_MAX 16

static int yai_embedded_law_path(char *out, size_t out_cap, const char *rel);
static int yai_read_text(const char *path, char *out, size_t out_cap);

typedef struct {
    int found;
    char id[192];
    char kind[96];
    char status[48];
    char review_state[48];
    int runtime_consumable;
    int experimental;
    int deprecated;
    char attachment_modes_csv[192];
    char precedence_class[192];
    char workspace_targets_csv[256];
    char family_targets_csv[192];
    char specialization_targets_csv[192];
    char manifest_ref[MAX_PATH_LEN];
} yai_governable_object_meta_t;

static int yai_policy_attachment_id_valid(const char *id)
{
    size_t i;
    if (!id || !id[0]) return 0;
    if (strlen(id) > 160u) return 0;
    for (i = 0; id[i]; i++)
    {
        const unsigned char c = (unsigned char)id[i];
        if (!(isalnum(c) || c == '.' || c == '-' || c == '_' || c == '/'))
            return 0;
    }
    return 1;
}

static int yai_policy_attachment_csv_count(const char *csv)
{
    int count = 0;
    char buf[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char *tok;
    char *save = NULL;
    if (!csv || !csv[0]) return 0;
    snprintf(buf, sizeof(buf), "%s", csv);
    tok = strtok_r(buf, ",", &save);
    while (tok)
    {
        if (tok[0]) count++;
        tok = strtok_r(NULL, ",", &save);
    }
    return count;
}

static int yai_policy_attachment_csv_contains(const char *csv, const char *id)
{
    char buf[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char *tok;
    char *save = NULL;
    if (!csv || !csv[0] || !id || !id[0]) return 0;
    snprintf(buf, sizeof(buf), "%s", csv);
    tok = strtok_r(buf, ",", &save);
    while (tok)
    {
        if (strcmp(tok, id) == 0) return 1;
        tok = strtok_r(NULL, ",", &save);
    }
    return 0;
}

static int yai_policy_attachment_csv_add(char *csv, size_t csv_cap, const char *id)
{
    size_t len;
    if (!csv || csv_cap == 0 || !id || !id[0]) return -1;
    if (yai_policy_attachment_csv_contains(csv, id)) return 0;
    if (csv[0])
    {
        len = strlen(csv);
        if (len + 1 + strlen(id) + 1 >= csv_cap) return -1;
        csv[len] = ',';
        csv[len + 1] = '\0';
    }
    if (strlen(csv) + strlen(id) + 1 >= csv_cap) return -1;
    strcat(csv, id);
    return 0;
}

static int yai_policy_attachment_csv_remove(char *csv, size_t csv_cap, const char *id)
{
    char src[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char out[sizeof(((yai_workspace_runtime_info_t *)0)->policy_attachments_csv)];
    char *tok;
    char *save = NULL;
    int removed = 0;
    if (!csv || csv_cap == 0 || !id || !id[0]) return -1;
    if (!csv[0]) return 0;
    snprintf(src, sizeof(src), "%s", csv);
    out[0] = '\0';
    tok = strtok_r(src, ",", &save);
    while (tok)
    {
        if (strcmp(tok, id) == 0)
        {
            removed = 1;
        }
        else if (tok[0])
        {
            if (out[0]) strncat(out, ",", sizeof(out) - strlen(out) - 1);
            strncat(out, tok, sizeof(out) - strlen(out) - 1);
        }
        tok = strtok_r(NULL, ",", &save);
    }
    snprintf(csv, csv_cap, "%s", out);
    return removed ? 0 : 1;
}

static void yai_governable_meta_defaults(yai_governable_object_meta_t *meta)
{
    if (!meta) return;
    memset(meta, 0, sizeof(*meta));
    snprintf(meta->kind, sizeof(meta->kind), "%s", "unknown");
    snprintf(meta->status, sizeof(meta->status), "%s", "unknown");
    snprintf(meta->review_state, sizeof(meta->review_state), "%s", "unknown");
    meta->runtime_consumable = 1;
}

static void yai_json_array_to_csv(cJSON *arr, char *out, size_t out_cap)
{
    int i;
    int n;
    if (!out || out_cap == 0) return;
    out[0] = '\0';
    if (!cJSON_IsArray(arr)) return;
    for (i = 0; i < cJSON_GetArraySize(arr); i++)
    {
        cJSON *it = cJSON_GetArrayItem(arr, i);
        const char *s;
        if (!cJSON_IsString(it) || !it->valuestring || !it->valuestring[0]) continue;
        s = it->valuestring;
        if (out[0]) strncat(out, ",", out_cap - strlen(out) - 1u);
        strncat(out, s, out_cap - strlen(out) - 1u);
    }
    n = (int)strlen(out);
    if (n < 0) out[0] = '\0';
}

static int yai_embedded_governable_object_lookup(const char *object_id, yai_governable_object_meta_t *meta)
{
    char path[MAX_PATH_LEN];
    char json[YAI_WS_JSON_IO_CAP];
    cJSON *doc = NULL;
    cJSON *objects = NULL;
    int i;
    int found = 0;
    yai_governable_meta_defaults(meta);
    if (!object_id || !object_id[0]) return 0;
    if (meta) snprintf(meta->id, sizeof(meta->id), "%s", object_id);
    if (yai_embedded_law_path(path, sizeof(path), "registry/governable-objects.v1.json") != 0)
        return 0;
    if (yai_read_text(path, json, sizeof(json)) != 0)
        return 0;

    doc = cJSON_Parse(json);
    if (!doc) return 0;
    objects = cJSON_GetObjectItemCaseSensitive(doc, "objects");
    if (!cJSON_IsArray(objects))
    {
        cJSON_Delete(doc);
        return 0;
    }

    for (i = 0; i < cJSON_GetArraySize(objects); i++)
    {
        cJSON *obj = cJSON_GetArrayItem(objects, i);
        cJSON *id = cJSON_GetObjectItemCaseSensitive(obj, "id");
        cJSON *kind = cJSON_GetObjectItemCaseSensitive(obj, "kind");
        cJSON *status = cJSON_GetObjectItemCaseSensitive(obj, "status");
        cJSON *review = cJSON_GetObjectItemCaseSensitive(obj, "review_state");
        cJSON *runtime = cJSON_GetObjectItemCaseSensitive(obj, "runtime_consumable");
        cJSON *experimental = cJSON_GetObjectItemCaseSensitive(obj, "experimental");
        cJSON *deprecated = cJSON_GetObjectItemCaseSensitive(obj, "deprecated");
        cJSON *modes = cJSON_GetObjectItemCaseSensitive(obj, "attachment_modes");
        cJSON *precedence = cJSON_GetObjectItemCaseSensitive(obj, "precedence_class");
        cJSON *scope = cJSON_GetObjectItemCaseSensitive(obj, "scope_targets");
        cJSON *manifest_ref = cJSON_GetObjectItemCaseSensitive(obj, "manifest_ref");
        if (!cJSON_IsString(id) || !id->valuestring) continue;
        if (strcmp(id->valuestring, object_id) != 0) continue;
        found = 1;
        if (meta)
        {
            if (cJSON_IsString(kind) && kind->valuestring)
                snprintf(meta->kind, sizeof(meta->kind), "%s", kind->valuestring);
            if (cJSON_IsString(status) && status->valuestring)
                snprintf(meta->status, sizeof(meta->status), "%s", status->valuestring);
            if (cJSON_IsString(review) && review->valuestring)
                snprintf(meta->review_state, sizeof(meta->review_state), "%s", review->valuestring);
            if (cJSON_IsBool(runtime))
                meta->runtime_consumable = cJSON_IsTrue(runtime) ? 1 : 0;
            if (cJSON_IsBool(experimental))
                meta->experimental = cJSON_IsTrue(experimental) ? 1 : 0;
            if (cJSON_IsBool(deprecated))
                meta->deprecated = cJSON_IsTrue(deprecated) ? 1 : 0;
            yai_json_array_to_csv(modes, meta->attachment_modes_csv, sizeof(meta->attachment_modes_csv));
            if (cJSON_IsString(precedence) && precedence->valuestring)
                snprintf(meta->precedence_class, sizeof(meta->precedence_class), "%s", precedence->valuestring);
            if (cJSON_IsObject(scope))
            {
                yai_json_array_to_csv(cJSON_GetObjectItemCaseSensitive(scope, "workspace_ids"),
                                      meta->workspace_targets_csv,
                                      sizeof(meta->workspace_targets_csv));
                yai_json_array_to_csv(cJSON_GetObjectItemCaseSensitive(scope, "family_targets"),
                                      meta->family_targets_csv,
                                      sizeof(meta->family_targets_csv));
                yai_json_array_to_csv(cJSON_GetObjectItemCaseSensitive(scope, "specialization_targets"),
                                      meta->specialization_targets_csv,
                                      sizeof(meta->specialization_targets_csv));
            }
            if (cJSON_IsString(manifest_ref) && manifest_ref->valuestring)
                snprintf(meta->manifest_ref, sizeof(meta->manifest_ref), "%s", manifest_ref->valuestring);
            meta->found = 1;
        }
        break;
    }

    cJSON_Delete(doc);
    return found;
}

static int yai_csv_has_token(const char *csv, const char *token)
{
    return yai_policy_attachment_csv_contains(csv, token);
}

static int yai_target_match(const char *target_csv, const char *value)
{
    if (!target_csv || !target_csv[0]) return 1;
    if (!value || !value[0]) return 0;
    return yai_csv_has_token(target_csv, value);
}

static void yai_workspace_policy_effective_context(const yai_workspace_runtime_info_t *info,
                                                   char *family,
                                                   size_t family_cap,
                                                   char *specialization,
                                                   size_t specialization_cap)
{
    const char *f;
    const char *s;
    f = info->inferred_family[0] ? info->inferred_family : info->declared_control_family;
    s = info->inferred_specialization[0] ? info->inferred_specialization : info->declared_specialization;
    snprintf(family, family_cap, "%s", f && f[0] ? f : "unset");
    snprintf(specialization, specialization_cap, "%s", s && s[0] ? s : "unset");
}

static const char *yai_workspace_attachment_state_for_meta(const yai_governable_object_meta_t *meta, int is_attached)
{
    if (!meta) return is_attached ? "attached_active" : "attached_inactive";
    if (strcmp(meta->status, "candidate") == 0) return "candidate_only";
    if (strcmp(meta->status, "approved") == 0 && !is_attached) return "approved_not_attached";
    if (is_attached) return "attached_active";
    return "attached_inactive";
}

static int yai_workspace_policy_evaluate(const yai_workspace_runtime_info_t *info,
                                         const yai_governable_object_meta_t *meta,
                                         char *eligibility_result,
                                         size_t eligibility_cap,
                                         char *compatibility_result,
                                         size_t compatibility_cap,
                                         char *conflict_summary,
                                         size_t conflict_cap,
                                         char *warnings,
                                         size_t warnings_cap,
                                         int *blocking)
{
    char family[96];
    char specialization[96];
    int blocker = 0;
    int warn = 0;
    if (!info || !meta) return -1;
    if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "eligible");
    if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "dry_run_passed");
    if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "none");
    if (warnings && warnings_cap) warnings[0] = '\0';
    yai_workspace_policy_effective_context(info, family, sizeof(family), specialization, sizeof(specialization));

    if (!yai_target_match(meta->workspace_targets_csv, info->ws_id))
    {
        blocker = 1;
        if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "ineligible");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "workspace_target_mismatch");
    }
    if (!blocker && !yai_target_match(meta->family_targets_csv, family))
    {
        blocker = 1;
        if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "ineligible");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "family_target_mismatch");
    }
    if (!blocker && !yai_target_match(meta->specialization_targets_csv, specialization))
    {
        blocker = 1;
        if (eligibility_result && eligibility_cap) snprintf(eligibility_result, eligibility_cap, "%s", "ineligible");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "specialization_target_mismatch");
    }
    if (!blocker &&
        meta->attachment_modes_csv[0] &&
        !(yai_csv_has_token(meta->attachment_modes_csv, "workspace_attach") ||
          yai_csv_has_token(meta->attachment_modes_csv, "workspace-attach")))
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "workspace_attach_not_allowed");
    }
    if (!blocker && (!meta->runtime_consumable || meta->deprecated))
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "object_not_runtime_consumable");
    }
    if (!blocker &&
        strcmp(meta->kind, "enterprise_custom_governance_object") == 0 &&
        strcmp(meta->status, "approved") != 0 &&
        strcmp(meta->status, "applied") != 0 &&
        strcmp(meta->status, "active") != 0)
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "enterprise_object_not_attachable_status");
    }
    if (!blocker &&
        strcmp(meta->kind, "enterprise_custom_governance_object") == 0 &&
        strcmp(meta->review_state, "approved") != 0)
    {
        blocker = 1;
        if (compatibility_result && compatibility_cap) snprintf(compatibility_result, compatibility_cap, "%s", "conflict_blocking");
        if (conflict_summary && conflict_cap) snprintf(conflict_summary, conflict_cap, "%s", "enterprise_object_not_review_approved");
    }
    if (meta->experimental)
    {
        warn = 1;
        if (warnings && warnings_cap) snprintf(warnings, warnings_cap, "%s", "experimental_object");
    }
    if (strcmp(meta->review_state, "in_review") == 0)
    {
        warn = 1;
        if (warnings && warnings_cap)
        {
            if (warnings[0]) strncat(warnings, ",", warnings_cap - strlen(warnings) - 1u);
            strncat(warnings, "review_in_progress", warnings_cap - strlen(warnings) - 1u);
        }
    }
    if (!blocker && warn && compatibility_result && compatibility_cap)
        snprintf(compatibility_result, compatibility_cap, "%s", "eligible_with_warnings");
    if (blocking) *blocking = blocker;
    return 0;
}

static const char *yai_get_home(void)
{
    const char *home = getenv("HOME");
    return (home && home[0]) ? home : NULL;
}

static int yai_workspace_is_scientific(const yai_workspace_runtime_info_t *info)
{
  const char *family;
  if (!info) return 0;
  family = info->inferred_family[0] ? info->inferred_family : info->declared_control_family;
  return (family[0] && strcmp(family, "scientific") == 0) ? 1 : 0;
}

static int yai_workspace_is_digital(const yai_workspace_runtime_info_t *info)
{
    const char *family;
    if (!info) return 0;
    family = info->inferred_family[0] ? info->inferred_family : info->declared_control_family;
    return (family[0] && strcmp(family, "digital") == 0) ? 1 : 0;
}

static void yai_workspace_build_scientific_summaries(const yai_workspace_runtime_info_t *info,
                                                     char *experiment_context,
                                                     size_t experiment_cap,
                                                     char *parameter_governance,
                                                     size_t parameter_cap,
                                                     char *reproducibility,
                                                     size_t reproducibility_cap,
                                                     char *dataset_integrity,
                                                     size_t dataset_cap,
                                                     char *publication_control,
                                                     size_t publication_cap)
{
    const char *spec;
    const char *effect;
    int scientific;
    if (!info) return;
    spec = info->inferred_specialization[0] ? info->inferred_specialization : info->declared_specialization;
    effect = info->last_effect_summary[0] ? info->last_effect_summary : "not resolved";
    scientific = yai_workspace_is_scientific(info);

    if (!scientific) {
        (void)snprintf(experiment_context, experiment_cap, "%s", "not scientific context");
        (void)snprintf(parameter_governance, parameter_cap, "%s", "not scientific context");
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "not scientific context");
        (void)snprintf(dataset_integrity, dataset_cap, "%s", "not scientific context");
        (void)snprintf(publication_control, publication_cap, "%s", "not scientific context");
        return;
    }

    if (strcmp(spec, "experiment-configuration") == 0) {
        (void)snprintf(experiment_context, experiment_cap, "%s", "experiment configuration governance active");
    } else if (strcmp(spec, "parameter-governance") == 0) {
        (void)snprintf(experiment_context, experiment_cap, "%s", "parameter-focused experiment path active");
    } else {
        (void)snprintf(experiment_context, experiment_cap, "%s", "scientific workspace execution path active");
    }

    if (strstr(info->last_evidence_summary, "parameter_lock_required") ||
        strstr(info->last_evidence_summary, "parameter_diff_trace_required")) {
        (void)snprintf(parameter_governance, parameter_cap, "%s", "parameter lock and diff trace required");
    } else if (strcmp(spec, "parameter-governance") == 0) {
        (void)snprintf(parameter_governance, parameter_cap, "%s", "parameter governance enforced");
    } else {
        (void)snprintf(parameter_governance, parameter_cap, "%s", "parameter governance not primary in last resolution");
    }

    if (strstr(info->last_evidence_summary, "reproducibility_proofpack_required")) {
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "reproducibility proofpack required");
    } else if (strcmp(spec, "reproducibility-control") == 0) {
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "reproducibility control active");
    } else {
        (void)snprintf(reproducibility, reproducibility_cap, "%s", "reproducibility checks partial");
    }

    if (strstr(info->last_evidence_summary, "dataset_integrity_attestation_required") ||
        strcmp(spec, "dataset-integrity") == 0) {
        (void)snprintf(dataset_integrity, dataset_cap, "%s", "dataset integrity attestation required");
    } else {
        (void)snprintf(dataset_integrity, dataset_cap, "%s", "dataset integrity checks not primary in last resolution");
    }

    if (strcmp(spec, "result-publication-control") == 0) {
        (void)snprintf(publication_control,
                       publication_cap,
                       "%s",
                       strcmp(effect, "deny") == 0 ? "publication blocked pending authority/repro checks" :
                       strcmp(effect, "quarantine") == 0 ? "publication quarantined pending review" :
                       strcmp(effect, "review_required") == 0 ? "publication requires final review" :
                       "publication path allowed with evidence obligations");
    } else {
        (void)snprintf(publication_control, publication_cap, "%s", "publication control not primary in last resolution");
    }
}

static void yai_workspace_build_digital_summaries(const yai_workspace_runtime_info_t *info,
                                                  char *outbound_context,
                                                  size_t outbound_cap,
                                                  char *sink_target,
                                                  size_t sink_cap,
                                                  char *publication_control,
                                                  size_t publication_cap,
                                                  char *retrieval_control,
                                                  size_t retrieval_cap,
                                                  char *distribution_control,
                                                  size_t distribution_cap)
{
    const char *spec;
    const char *effect;
    int digital;
    if (!info) return;
    spec = info->inferred_specialization[0] ? info->inferred_specialization : info->declared_specialization;
    effect = info->last_effect_summary[0] ? info->last_effect_summary : "not resolved";
    digital = yai_workspace_is_digital(info);

    if (!digital) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "not digital context");
        (void)snprintf(sink_target, sink_cap, "%s", "not digital context");
        (void)snprintf(publication_control, publication_cap, "%s", "not digital context");
        (void)snprintf(retrieval_control, retrieval_cap, "%s", "not digital context");
        (void)snprintf(distribution_control, distribution_cap, "%s", "not digital context");
        return;
    }

    if (strcmp(spec, "remote-retrieval") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "remote retrieval governed path active");
    } else if (strcmp(spec, "remote-publication") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "remote publication governed path active");
    } else if (strcmp(spec, "external-commentary") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "external commentary governed path active");
    } else if (strcmp(spec, "artifact-distribution") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "artifact distribution governed path active");
    } else if (strcmp(spec, "digital-sink-control") == 0) {
        (void)snprintf(outbound_context, outbound_cap, "%s", "digital sink governance active");
    } else {
        (void)snprintf(outbound_context, outbound_cap, "%s", "digital outbound governance active");
    }

    if (strstr(info->last_evidence_summary, "sink_policy_attestation_required")) {
        (void)snprintf(sink_target, sink_cap, "%s", "sink policy attestation required");
    } else if (strcmp(spec, "digital-sink-control") == 0) {
        (void)snprintf(sink_target, sink_cap, "%s", "sink target control active");
    } else {
        (void)snprintf(sink_target, sink_cap, "%s", "sink target checks not primary in last resolution");
    }

    if (strcmp(spec, "remote-publication") == 0 || strstr(info->last_evidence_summary, "publication_review_record_required")) {
        (void)snprintf(publication_control,
                       publication_cap,
                       "%s",
                       strcmp(effect, "deny") == 0 ? "publication denied pending authority/sink checks" :
                       strcmp(effect, "quarantine") == 0 ? "publication quarantined pending sink review" :
                       strcmp(effect, "review_required") == 0 ? "publication requires review record" :
                       "publication path allowed with destination trace");
    } else {
        (void)snprintf(publication_control, publication_cap, "%s", "publication control not primary in last resolution");
    }

    if (strcmp(spec, "remote-retrieval") == 0 || strstr(info->last_evidence_summary, "retrieval_source_attestation_required")) {
        (void)snprintf(retrieval_control, retrieval_cap, "%s", "retrieval source attestation required");
    } else {
        (void)snprintf(retrieval_control, retrieval_cap, "%s", "retrieval control not primary in last resolution");
    }

    if (strcmp(spec, "artifact-distribution") == 0 || strstr(info->last_evidence_summary, "distribution_manifest_required")) {
        (void)snprintf(distribution_control, distribution_cap, "%s", "distribution manifest and destination trace required");
    } else {
        (void)snprintf(distribution_control, distribution_cap, "%s", "artifact distribution not primary in last resolution");
    }
}

void yai_session_workspace_event_semantics(const yai_workspace_runtime_info_t *info,
                                           char *declared_scenario_spec,
                                           size_t declared_cap,
                                           char *business_spec,
                                           size_t business_cap,
                                           char *enforcement_spec,
                                           size_t enforcement_cap,
                                           char *flow_stage,
                                           size_t flow_cap,
                                           int *external_boundary)
{
    const char *declared = "";
    const char *inferred = "";
    const char *business = "";
    const char *enforcement = "";
    const char *stage = "unknown";
    int boundary = 0;

    if (!info) return;
    declared = info->declared_specialization[0] ? info->declared_specialization : "";
    inferred = info->inferred_specialization[0] ? info->inferred_specialization : "";
    enforcement = inferred[0] ? inferred : declared;

    if (declared[0] && inferred[0] &&
        strcmp(inferred, "network-egress") == 0 &&
        strcmp(declared, "network-egress") != 0)
        business = declared;
    else
        business = inferred[0] ? inferred : declared;

    if (strcmp(business, "remote-retrieval") == 0) stage = "retrieve";
    else if (strcmp(business, "experiment-configuration") == 0) stage = "ingress";
    else if (strcmp(business, "parameter-governance") == 0) stage = "transform";
    else if (strcmp(business, "black-box-evaluation") == 0) stage = "transform";
    else if (strcmp(business, "result-publication-control") == 0) stage = "publish";
    else if (strcmp(business, "remote-publication") == 0) stage = "publish";
    else if (strcmp(business, "external-commentary") == 0) stage = "publish";
    else if (strcmp(business, "artifact-distribution") == 0) stage = "distribute";
    else if (strcmp(business, "digital-sink-control") == 0) stage = "egress";
    else if (strcmp(enforcement, "network-egress") == 0) stage = "egress";
    else if (strcmp(info->last_effect_summary, "review_required") == 0) stage = "approve";
    else if (strcmp(info->last_effect_summary, "allow") == 0) stage = "actuate";

    if (strcmp(enforcement, "network-egress") == 0 ||
        strcmp(business, "remote-retrieval") == 0 ||
        strcmp(business, "remote-publication") == 0 ||
        strcmp(business, "external-commentary") == 0 ||
        strcmp(business, "artifact-distribution") == 0 ||
        strcmp(business, "digital-sink-control") == 0 ||
        strcmp(business, "result-publication-control") == 0)
        boundary = 1;

    if (declared_scenario_spec && declared_cap) snprintf(declared_scenario_spec, declared_cap, "%s", declared[0] ? declared : "unset");
    if (business_spec && business_cap) snprintf(business_spec, business_cap, "%s", business[0] ? business : "not_resolved");
    if (enforcement_spec && enforcement_cap) snprintf(enforcement_spec, enforcement_cap, "%s", enforcement[0] ? enforcement : "not_resolved");
    if (flow_stage && flow_cap) snprintf(flow_stage, flow_cap, "%s", stage);
    if (external_boundary) *external_boundary = boundary;
}

static const char *yai_workspace_review_state_from_effect(const char *effect)
{
    if (!effect || !effect[0]) return "unresolved";
    if (strcmp(effect, "review_required") == 0) return "pending_review";
    if (strcmp(effect, "quarantine") == 0) return "quarantined";
    if (strcmp(effect, "deny") == 0) return "blocked";
    if (strcmp(effect, "allow") == 0) return "clear";
    return "unresolved";
}

static void yai_workspace_operational_summary(const char *flow_stage,
                                              const char *business_spec,
                                              const char *effect,
                                              char *out,
                                              size_t out_cap)
{
    if (!out || out_cap == 0) return;
    (void)snprintf(out,
                   out_cap,
                   "%s/%s => %s",
                   flow_stage && flow_stage[0] ? flow_stage : "unknown",
                   business_spec && business_spec[0] ? business_spec : "not_resolved",
                   effect && effect[0] ? effect : "not_resolved");
}

static void trim_trailing_slashes(char *path);

static int yai_workspace_store_root_path(char *out, size_t out_cap)
{
    const char *home = yai_get_home();
    const char *env_root = getenv("YAI_WORKSPACE_ROOT");
    if (!out || out_cap == 0 || !home)
        return -1;
    if (env_root && env_root[0])
    {
        if (snprintf(out, out_cap, "%s", env_root) <= 0)
            return -1;
        trim_trailing_slashes(out);
        return 0;
    }
    if (snprintf(out, out_cap, "%s/.yai/workspaces", home) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_runtime_state_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_metadata_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/metadata", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_state_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/state", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_traces_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/traces", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_artifacts_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/artifacts", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_runtime_local_root_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(out, out_cap, "%s/runtime", run_dir) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_containment_surface_paths(yai_workspace_runtime_info_t *info)
{
    if (!info || !info->ws_id[0])
        return -1;

    if (yai_workspace_state_root_path(info->ws_id, info->state_root, sizeof(info->state_root)) != 0)
        return -1;
    if (yai_workspace_traces_root_path(info->ws_id, info->traces_root, sizeof(info->traces_root)) != 0)
        return -1;
    if (yai_workspace_artifacts_root_path(info->ws_id, info->artifacts_root, sizeof(info->artifacts_root)) != 0)
        return -1;
    if (yai_workspace_runtime_local_root_path(info->ws_id, info->runtime_local_root, sizeof(info->runtime_local_root)) != 0)
        return -1;
    if (snprintf(info->state_surface_path, sizeof(info->state_surface_path), "%s/workspace-state.json", info->state_root) <= 0)
        return -1;
    if (snprintf(info->traces_index_path, sizeof(info->traces_index_path), "%s/index.json", info->traces_root) <= 0)
        return -1;
    if (snprintf(info->artifacts_index_path, sizeof(info->artifacts_index_path), "%s/index.json", info->artifacts_root) <= 0)
        return -1;
    if (snprintf(info->runtime_surface_path, sizeof(info->runtime_surface_path), "%s/runtime-state.json", info->runtime_local_root) <= 0)
        return -1;
    if (snprintf(info->binding_state_path, sizeof(info->binding_state_path), "%s/binding.json", info->metadata_root) <= 0)
        return -1;
    if (snprintf(info->attach_descriptor_ref, sizeof(info->attach_descriptor_ref), "%s/attach-descriptor.json", info->runtime_local_root) <= 0)
        return -1;
    if (snprintf(info->execution_profile_ref, sizeof(info->execution_profile_ref), "%s/execution-profile.json", info->runtime_local_root) <= 0)
        return -1;
    return 0;
}

static void yai_workspace_security_defaults(yai_workspace_runtime_info_t *info)
{
    if (!info)
        return;
    snprintf(info->security_envelope_version, sizeof(info->security_envelope_version), "%s", "v1");
    snprintf(info->security_level_declared, sizeof(info->security_level_declared), "%s", "scoped");
    snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "logical");
    snprintf(info->security_enforcement_mode, sizeof(info->security_enforcement_mode), "%s", "runtime_scoped");
    snprintf(info->security_backend_mode, sizeof(info->security_backend_mode), "%s", "none");
    info->scope_process = 0;
    info->scope_filesystem = 1;
    info->scope_socket = 0;
    info->scope_network = 0;
    info->scope_resource = 0;
    info->scope_privilege = 0;
    info->scope_runtime_route = 1;
    info->scope_binding = 1;
    info->capability_sandbox_ready = 1;
    info->capability_hardened_fs = 1;
    info->capability_process_isolation = 0;
    info->capability_network_policy = 0;
    snprintf(info->execution_mode_requested, sizeof(info->execution_mode_requested), "%s", "scoped");
    snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
    info->execution_mode_degraded = 0;
    snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
    snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
    snprintf(info->execution_advisory_scopes, sizeof(info->execution_advisory_scopes), "%s", "process,socket,network,resource,privilege");
    snprintf(info->process_intent, sizeof(info->process_intent), "%s", "shared_runtime");
    snprintf(info->channel_mode, sizeof(info->channel_mode), "%s", "global_control_scoped_route");
    snprintf(info->artifact_policy_mode, sizeof(info->artifact_policy_mode), "%s", "workspace_owned");
    snprintf(info->network_intent, sizeof(info->network_intent), "%s", "advisory_none");
    snprintf(info->resource_intent, sizeof(info->resource_intent), "%s", "advisory_none");
    snprintf(info->privilege_intent, sizeof(info->privilege_intent), "%s", "inherited_host");
    info->attach_descriptor_ref[0] = '\0';
    info->execution_profile_ref[0] = '\0';
}

static int yai_workspace_security_level_is_valid(const char *level)
{
    if (!level || !level[0])
        return 0;
    return strcmp(level, "logical") == 0 ||
           strcmp(level, "scoped") == 0 ||
           strcmp(level, "isolated") == 0 ||
           strcmp(level, "sandboxed") == 0;
}

static void yai_workspace_security_recompute_effective(yai_workspace_runtime_info_t *info)
{
    const char *requested = NULL;
    if (!info)
        return;
    requested = yai_workspace_security_level_is_valid(info->security_level_declared)
                    ? info->security_level_declared
                    : "scoped";
    snprintf(info->execution_mode_requested, sizeof(info->execution_mode_requested), "%s", requested);

    if (!info->containment_ready || !info->namespace_valid)
    {
        snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "logical");
        snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "logical");
        info->execution_mode_degraded = strcmp(requested, "logical") != 0;
        snprintf(info->execution_degraded_reason,
                 sizeof(info->execution_degraded_reason),
                 "%s",
                 info->execution_mode_degraded ? "containment_not_ready_or_namespace_invalid" : "none");
        snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
        return;
    }

    if (strcmp(requested, "sandboxed") == 0)
    {
        snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "scoped");
        snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
        info->execution_mode_degraded = 1;
        snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "sandbox_backend_unavailable");
        snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "process,socket,network,resource,privilege");
        return;
    }

    if (strcmp(requested, "isolated") == 0)
    {
        if (info->scope_process || info->scope_socket || info->scope_network || info->scope_resource || info->scope_privilege)
        {
            snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "isolated");
            snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "isolated");
            info->execution_mode_degraded = 0;
            snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
            snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
        }
        else
        {
            snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "scoped");
            snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
            info->execution_mode_degraded = 1;
            snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "isolated_scopes_not_enforced");
            snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "process,socket,network,resource,privilege");
        }
        return;
    }

    if (strcmp(requested, "logical") == 0)
    {
        snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "logical");
        snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "logical");
        info->execution_mode_degraded = 0;
        snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
        snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
        return;
    }

    snprintf(info->security_level_effective, sizeof(info->security_level_effective), "%s", "scoped");
    snprintf(info->execution_mode_effective, sizeof(info->execution_mode_effective), "%s", "scoped");
    info->execution_mode_degraded = 0;
    snprintf(info->execution_degraded_reason, sizeof(info->execution_degraded_reason), "%s", "none");
    snprintf(info->execution_unsupported_scopes, sizeof(info->execution_unsupported_scopes), "%s", "none");
}

static int yai_path_is_under(const char *root, const char *path)
{
    size_t n;
    if (!root || !root[0] || !path || !path[0])
        return 0;
    n = strlen(root);
    if (strncmp(root, path, n) != 0)
        return 0;
    return path[n] == '\0' || path[n] == '/';
}

static int yai_is_ws_runtime_path_valid(const char *ws_id,
                                        const char *actual_path,
                                        const char *expected_path)
{
    if (!ws_id || !ws_id[0] || !actual_path || !actual_path[0] || !expected_path || !expected_path[0])
        return 0;
    return strcmp(actual_path, expected_path) == 0;
}

static void yai_workspace_fill_shell_relation(yai_workspace_runtime_info_t *info)
{
    char cwd[MAX_PATH_LEN];
    if (!info)
        return;
    info->shell_cwd[0] = '\0';
    snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "unknown");
    if (!getcwd(cwd, sizeof(cwd)))
    {
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "cwd_unavailable");
        return;
    }
    snprintf(info->shell_cwd, sizeof(info->shell_cwd), "%s", cwd);
    if (!info->root_path[0])
    {
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "workspace_root_unset");
        return;
    }
    if (yai_path_is_under(info->root_path, cwd))
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "inside_workspace_root");
    else
        snprintf(info->shell_path_relation, sizeof(info->shell_path_relation), "%s", "outside_workspace_root");
}

static int mkdir_if_missing(const char *path, mode_t mode)
{
    struct stat st;
    if (stat(path, &st) == 0)
        return S_ISDIR(st.st_mode) ? 0 : -1;
    return mkdir(path, mode);
}

static int yai_file_read_all(const char *path, char *out, size_t out_cap)
{
    FILE *f;
    size_t n;
    if (!path || !out || out_cap < 2)
        return -1;
    f = fopen(path, "rb");
    if (!f)
        return -1;
    n = fread(out, 1, out_cap - 1, f);
    fclose(f);
    out[n] = '\0';
    return 0;
}

static int yai_file_write_all(const char *path, const char *content)
{
    FILE *f;
    if (!path || !content)
        return -1;
    f = fopen(path, "wb");
    if (!f)
        return -1;
    if (fwrite(content, 1, strlen(content), f) != strlen(content))
    {
        fclose(f);
        return -1;
    }
    fclose(f);
    return 0;
}

static void yai_remove_range(char *buf, char *from, char *to_after)
{
    size_t tail_len;
    if (!buf || !from || !to_after || to_after < from)
        return;
    tail_len = strlen(to_after);
    memmove(from, to_after, tail_len + 1);
}

static int yai_remove_block_in_file(const char *path, const char *begin, const char *end)
{
    char buf[YAI_WS_JSON_IO_CAP];
    char *p, *q, *q_end;
    int changed = 0;
    if (!path || !begin || !end)
        return -1;
    if (yai_file_read_all(path, buf, sizeof(buf)) != 0)
        return 0;
    while ((p = strstr(buf, begin)) != NULL)
    {
        q = strstr(p, end);
        if (!q)
            break;
        q_end = q + strlen(end);
        if (*q_end == '\n')
            q_end++;
        yai_remove_range(buf, p, q_end);
        changed = 1;
    }
    if (changed)
        return yai_file_write_all(path, buf);
    return 0;
}

static int yai_replace_managed_block(const char *path, const char *begin, const char *end, const char *block)
{
    char buf[YAI_WS_JSON_IO_CAP];
    char out[YAI_WS_JSON_IO_CAP];
    char *p, *q;
    int n;
    if (!path || !begin || !end || !block)
        return -1;

    if (yai_file_read_all(path, buf, sizeof(buf)) != 0)
    {
        n = snprintf(out, sizeof(out), "%s", block);
        if (n <= 0 || (size_t)n >= sizeof(out))
            return -1;
        return yai_file_write_all(path, out);
    }

    p = strstr(buf, begin);
    if (!p)
    {
        n = snprintf(out, sizeof(out), "%s%s%s",
                     buf,
                     (buf[0] && buf[strlen(buf) - 1] != '\n') ? "\n" : "",
                     block);
        if (n <= 0 || (size_t)n >= sizeof(out))
            return -1;
        return yai_file_write_all(path, out);
    }

    q = strstr(p, end);
    if (!q)
    {
        n = snprintf(out, sizeof(out), "%s%s%s",
                     buf,
                     (buf[0] && buf[strlen(buf) - 1] != '\n') ? "\n" : "",
                     block);
        if (n <= 0 || (size_t)n >= sizeof(out))
            return -1;
        return yai_file_write_all(path, out);
    }

    q += strlen(end);
    if (*q == '\n')
        q++;

    n = snprintf(out, sizeof(out), "%.*s%s%s", (int)(p - buf), buf, block, q);
    if (n <= 0 || (size_t)n >= sizeof(out))
        return -1;
    return yai_file_write_all(path, out);
}

static int yai_session_ensure_shell_integration(void)
{
    const char *home = yai_get_home();
    char config_dir[MAX_PATH_LEN];
    char yai_cfg_dir[MAX_PATH_LEN];
    char shell_dir[MAX_PATH_LEN];
    char prompt_script[MAX_PATH_LEN];
    char zshrc[MAX_PATH_LEN];
    const char *script_content =
        "# yai managed prompt integration (do not edit manually)\n"
        "function prompt_yai_ws() {\n"
        "  emulate -L zsh\n"
        "  local tok cmd\n"
        "  cmd=\"${commands[yai-ws-token]}\"\n"
        "  [[ -n \"$cmd\" ]] || cmd=\"$HOME/Developer/YAI/yai/tools/bin/yai-ws-token\"\n"
        "  [[ -x \"$cmd\" ]] || return\n"
        "  tok=\"$($cmd 2>/dev/null)\"\n"
        "  [[ -n \"$tok\" ]] || return\n"
        "  p10k segment -f 255 -b 35 -t \"$tok\"\n"
        "}\n"
        "typeset -ga POWERLEVEL9K_LEFT_PROMPT_ELEMENTS\n"
        "typeset -ga POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS\n"
        "POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS:#yai_ws})\n"
        "POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS:#context})\n"
        "POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS=(${POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS:#context})\n"
        "if (( ${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[(I)vcs]} <= ${#POWERLEVEL9K_LEFT_PROMPT_ELEMENTS} )); then\n"
        "  integer _yai_i=${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[(I)vcs]}\n"
        "  POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(\n"
        "    ${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[1,$((_yai_i-1))]}\n"
        "    yai_ws\n"
        "    ${POWERLEVEL9K_LEFT_PROMPT_ELEMENTS[_yai_i,-1]}\n"
        "  )\n"
        "  unset _yai_i\n"
        "else\n"
        "  POWERLEVEL9K_LEFT_PROMPT_ELEMENTS+=(yai_ws)\n"
        "fi\n";
    const char *zshrc_block =
        YAI_MANAGED_BEGIN "\n"
        "if [[ -f \"$HOME/.config/yai/shell/yai-prompt.zsh\" ]]; then\n"
        "  source \"$HOME/.config/yai/shell/yai-prompt.zsh\"\n"
        "fi\n"
        YAI_MANAGED_END "\n";

    if (!home || !home[0])
        return -1;
    if (snprintf(config_dir, sizeof(config_dir), "%s/.config", home) <= 0)
        return -1;
    if (snprintf(yai_cfg_dir, sizeof(yai_cfg_dir), "%s/.config/yai", home) <= 0)
        return -1;
    if (snprintf(shell_dir, sizeof(shell_dir), "%s/.config/yai/shell", home) <= 0)
        return -1;
    if (snprintf(prompt_script, sizeof(prompt_script), "%s/yai-prompt.zsh", shell_dir) <= 0)
        return -1;
    if (snprintf(zshrc, sizeof(zshrc), "%s/.zshrc", home) <= 0)
        return -1;

    (void)mkdir_if_missing(config_dir, 0755);
    (void)mkdir_if_missing(yai_cfg_dir, 0755);
    (void)mkdir_if_missing(shell_dir, 0755);

    if (yai_file_write_all(prompt_script, script_content) != 0)
        return -1;

    /* Cleanup legacy manual patches injected during migration/debug rounds. */
    (void)yai_remove_block_in_file(zshrc,
                                   "# --- YAI workspace token (left prompt, robust) ---",
                                   "# --- end ---");
    (void)yai_remove_block_in_file(zshrc,
                                   "# ==== YAI prompt patch (path -> workspace -> git) ====",
                                   "# ==== end patch ====");
    {
        char p10k[MAX_PATH_LEN];
        if (snprintf(p10k, sizeof(p10k), "%s/.p10k.zsh", home) > 0)
        {
            (void)yai_remove_block_in_file(p10k,
                                           "# === YAI workspace segment ===",
                                           "# === end YAI workspace segment ===");
        }
    }

    return yai_replace_managed_block(zshrc, YAI_MANAGED_BEGIN, YAI_MANAGED_END, zshrc_block);
}

static int mkdir_parents(const char *path, mode_t mode)
{
    char tmp[MAX_PATH_LEN];
    size_t len;
    char *p;

    if (!path || !path[0])
        return -1;

    snprintf(tmp, sizeof(tmp), "%s", path);
    len = strlen(tmp);
    if (len == 0 || len >= sizeof(tmp))
        return -1;

    for (p = tmp + 1; *p; ++p)
    {
        if (*p != '/')
            continue;
        *p = '\0';
        if (mkdir_if_missing(tmp, mode) != 0)
            return -1;
        *p = '/';
    }

    return mkdir_if_missing(tmp, mode);
}

static int remove_tree(const char *path)
{
    DIR *d = opendir(path);
    if (!d)
        return (errno == ENOENT) ? 0 : -1;

    struct dirent *ent;
    while ((ent = readdir(d)) != NULL)
    {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0)
            continue;

        char child[MAX_PATH_LEN];
        int n = snprintf(child, sizeof(child), "%s/%s", path, ent->d_name);
        if (n <= 0 || (size_t)n >= sizeof(child))
        {
            closedir(d);
            return -1;
        }

        struct stat st;
        if (stat(child, &st) != 0)
        {
            if (errno == ENOENT)
                continue;
            closedir(d);
            return -1;
        }

        if (S_ISDIR(st.st_mode))
        {
            if (remove_tree(child) != 0)
            {
                closedir(d);
                return -1;
            }
        }
        else if (unlink(child) != 0)
        {
            closedir(d);
            return -1;
        }
    }

    closedir(d);
    return rmdir(path);
}

static void trim_trailing_slashes(char *path)
{
    size_t len;
    if (!path)
        return;
    len = strlen(path);
    while (len > 1 && path[len - 1] == '/')
    {
        path[len - 1] = '\0';
        len--;
    }
}

static int yai_session_extract_json_long(const char *json, const char *key, long *out)
{
    char needle[64];
    const char *p;
    char *end = NULL;

    if (!json || !key || !out)
        return -1;

    if (snprintf(needle, sizeof(needle), "\"%s\"", key) <= 0)
        return -1;

    p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;

    *out = strtol(p, &end, 10);
    if (end == p)
        return -1;
    return 0;
}

static int yai_session_extract_json_bool(const char *json, const char *key, int *out)
{
    char needle[64];
    const char *p;

    if (!json || !key || !out)
        return -1;

    if (snprintf(needle, sizeof(needle), "\"%s\"", key) <= 0)
        return -1;

    p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;

    if (strncmp(p, "true", 4) == 0)
    {
        *out = 1;
        return 0;
    }
    if (strncmp(p, "false", 5) == 0)
    {
        *out = 0;
        return 0;
    }
    return -1;
}

static int yai_session_extract_json_double(const char *json, const char *key, double *out)
{
    char needle[64];
    const char *p;
    char *end = NULL;

    if (!json || !key || !out)
        return -1;
    if (snprintf(needle, sizeof(needle), "\"%s\"", key) <= 0)
        return -1;

    p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;

    *out = strtod(p, &end);
    if (end == p)
        return -1;
    return 0;
}

static int yai_workspace_manifest_path(const char *ws_id, char *out, size_t out_cap)
{
    char run_dir[MAX_PATH_LEN];
    if (!ws_id || !out || out_cap == 0)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), "") != 0)
        return -1;
    if (snprintf(out, out_cap, "%s%s/manifest.json", run_dir, ws_id) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_binding_path(char *out, size_t out_cap)
{
    const char *scope = getenv("YAI_WS_BIND_SCOPE");
    const char *home = yai_get_home();
    const char *tty = NULL;
    char tty_tag[128];
    size_t j = 0;
    size_t i = 0;

    if (!home || !out || out_cap == 0)
        return -1;

    if (!scope || scope[0] == '\0' || strcmp(scope, "tty") == 0 || strcmp(scope, "terminal") == 0)
    {
        tty = ttyname(STDIN_FILENO);
        if (!tty || !tty[0])
            tty = ttyname(STDOUT_FILENO);
        if (!tty || !tty[0])
            tty = ttyname(STDERR_FILENO);

        if (tty && tty[0])
        {
            for (i = 0; tty[i] != '\0' && j + 1 < sizeof(tty_tag); i++)
            {
                unsigned char c = (unsigned char)tty[i];
                if (isalnum(c))
                    tty_tag[j++] = (char)c;
                else
                    tty_tag[j++] = '_';
            }
            tty_tag[j] = '\0';
            if (tty_tag[0] == '\0')
                snprintf(tty_tag, sizeof(tty_tag), "%s", "unknown_tty");

            if (snprintf(out, out_cap, "%s/.yai/session/by-tty/%s.json", home, tty_tag) <= 0)
                return -1;
            return 0;
        }
    }

    if (snprintf(out, out_cap, "%s/.yai/session/active_workspace.json", home) <= 0)
        return -1;
    return 0;
}

static int yai_workspace_binding_write(const char *ws_id, const char *ws_alias)
{
    char path[MAX_PATH_LEN];
    char session_dir[MAX_PATH_LEN];
    FILE *f;
    time_t now = time(NULL);
    const char *home = yai_get_home();

    if (!home || !ws_id)
        return -1;
    if (snprintf(session_dir, sizeof(session_dir), "%s/.yai/session", home) <= 0)
        return -1;
    if (mkdir_parents(session_dir, 0755) != 0)
        return -1;
    if (yai_workspace_binding_path(path, sizeof(path)) != 0)
        return -1;
    {
        char *slash = strrchr(path, '/');
        if (slash)
        {
            char parent[MAX_PATH_LEN];
            size_t n = (size_t)(slash - path);
            if (n >= sizeof(parent))
                return -1;
            memcpy(parent, path, n);
            parent[n] = '\0';
            if (mkdir_parents(parent, 0755) != 0)
                return -1;
        }
    }

    f = fopen(path, "w");
    if (!f)
        return -1;

    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.binding.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"workspace_alias\": \"%s\",\n"
            "  \"bound_at\": %ld,\n"
            "  \"source\": \"explicit\"\n"
            "}\n",
            ws_id,
            (ws_alias && ws_alias[0]) ? ws_alias : ws_id,
            (long)now);
    fclose(f);
    return 0;
}

static int yai_workspace_binding_read(char *ws_id, size_t ws_id_cap, char *ws_alias, size_t ws_alias_cap)
{
    char path[MAX_PATH_LEN];
    FILE *f;
    char buf[1024];
    size_t r;

    if (!ws_id || ws_id_cap == 0 || !ws_alias || ws_alias_cap == 0)
        return -1;
    ws_id[0] = '\0';
    ws_alias[0] = '\0';

    if (yai_workspace_binding_path(path, sizeof(path)) != 0)
        return -1;
    f = fopen(path, "r");
    if (!f)
        return -1;
    r = fread(buf, 1, sizeof(buf) - 1, f);
    fclose(f);
    buf[r] = '\0';

    if (yai_session_extract_json_string(buf, "workspace_id", ws_id, ws_id_cap) != 0)
        return -1;
    (void)yai_session_extract_json_string(buf, "workspace_alias", ws_alias, ws_alias_cap);
    if (ws_alias[0] == '\0')
        snprintf(ws_alias, ws_alias_cap, "%s", ws_id);
    return 0;
}

static int yai_workspace_resolve_root_path(
    const char *ws_id,
    const char *root_path_opt,
    char *anchor_mode_out,
    size_t anchor_mode_cap,
    char *out,
    size_t out_cap)
{
    char abs_path[MAX_PATH_LEN];
    char store_root[MAX_PATH_LEN];
    const char *home = yai_get_home();

    if (!ws_id || !out || out_cap == 0 || !home)
        return -1;

    if (anchor_mode_out && anchor_mode_cap > 0)
        anchor_mode_out[0] = '\0';

    if (!root_path_opt || !root_path_opt[0])
    {
        if (yai_workspace_store_root_path(store_root, sizeof(store_root)) != 0)
            return -1;
        if (snprintf(out, out_cap, "%s/%s", store_root, ws_id) <= 0)
            return -1;
        trim_trailing_slashes(out);
        if (anchor_mode_out && anchor_mode_cap > 0)
            snprintf(anchor_mode_out, anchor_mode_cap, "%s",
                     getenv("YAI_WORKSPACE_ROOT") ? "managed_custom_root" : "managed_default_root");
        return 0;
    }

    if (strstr(root_path_opt, "..") != NULL)
        return -1;

    if (root_path_opt[0] == '/')
    {
        if (snprintf(abs_path, sizeof(abs_path), "%s", root_path_opt) <= 0)
            return -1;
        if (anchor_mode_out && anchor_mode_cap > 0)
            snprintf(anchor_mode_out, anchor_mode_cap, "%s", "explicit_absolute");
    }
    else
    {
        char cwd[MAX_PATH_LEN];
        if (!getcwd(cwd, sizeof(cwd)))
            return -1;
        if (snprintf(abs_path, sizeof(abs_path), "%s/%s", cwd, root_path_opt) <= 0)
            return -1;
        if (anchor_mode_out && anchor_mode_cap > 0)
            snprintf(anchor_mode_out, anchor_mode_cap, "%s", "explicit_relative");
    }

    trim_trailing_slashes(abs_path);

    snprintf(out, out_cap, "%s", abs_path);
    return 0;
}

int yai_session_extract_json_string(const char *json, const char *key, char *out, size_t out_cap)
{
    if (!json || !key || !out || out_cap == 0)
        return -1;

    char needle[64];
    int nn = snprintf(needle, sizeof(needle), "\"%s\"", key);
    if (nn <= 0 || (size_t)nn >= sizeof(needle))
        return -1;

    const char *p = strstr(json, needle);
    if (!p)
        return -1;
    p = strchr(p, ':');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;
    if (*p != '"')
        return -1;
    p++;
    const char *q = strchr(p, '"');
    if (!q)
        return -1;

    size_t len = (size_t)(q - p);
    if (len >= out_cap)
        len = out_cap - 1;
    memcpy(out, p, len);
    out[len] = '\0';
    return 0;
}

int yai_session_extract_argv_first(const char *json, char *out, size_t out_cap)
{
    const char *p = strstr(json ? json : "", "\"argv\"");
    if (!p)
        return -1;
    p = strchr(p, '[');
    if (!p)
        return -1;
    p++;
    while (*p == ' ' || *p == '\t')
        p++;
    if (*p != '"')
        return -1;
    p++;
    const char *q = strchr(p, '"');
    if (!q)
        return -1;
    size_t len = (size_t)(q - p);
    if (len >= out_cap)
        len = out_cap - 1;
    memcpy(out, p, len);
    out[len] = '\0';
    return 0;
}

int yai_session_extract_argv_flag_value(
    const char *json,
    const char *flag_a,
    const char *flag_b,
    char *out,
    size_t out_cap)
{
    if (!json || !out || out_cap == 0)
        return -1;

    const char *p = strstr(json, "\"argv\"");
    if (!p)
        return -1;
    p = strchr(p, '[');
    if (!p)
        return -1;
    p++;

    char prev[128] = {0};
    while (*p && *p != ']')
    {
        while (*p && *p != '"' && *p != ']')
            p++;
        if (*p != '"')
            break;
        p++;
        const char *q = strchr(p, '"');
        if (!q)
            break;

        size_t len = (size_t)(q - p);
        char cur[128];
        if (len >= sizeof(cur))
            len = sizeof(cur) - 1;
        memcpy(cur, p, len);
        cur[len] = '\0';

        if ((flag_a && strcmp(prev, flag_a) == 0) ||
            (flag_b && strcmp(prev, flag_b) == 0))
        {
            size_t out_len = strlen(cur);
            if (out_len >= out_cap)
                out_len = out_cap - 1;
            memcpy(out, cur, out_len);
            out[out_len] = '\0';
            return 0;
        }

        snprintf(prev, sizeof(prev), "%s", cur);
        p = q + 1;
    }

    return -1;
}

int yai_session_path_exists(const char *path)
{
    struct stat st;
    return (path && stat(path, &st) == 0) ? 1 : 0;
}

int yai_session_build_run_path(char *out, size_t out_cap, const char *suffix)
{
    const char *home = yai_get_home();
    if (!home || !out || out_cap == 0)
        return -1;

    int n = snprintf(out, out_cap, "%s/.yai/run/%s", home, suffix ? suffix : "");
    if (n <= 0 || (size_t)n >= out_cap)
        return -1;
    return 0;
}

static int yai_workspace_write_manifest_path(
    const char *manifest_path,
    const yai_workspace_runtime_info_t *info)
{
    FILE *f;

    if (!manifest_path || !info)
        return -1;

    f = fopen(manifest_path, "w");
    if (!f)
        return -1;
    fprintf(f, "{\n");
    fprintf(f, "  \"type\": \"yai.workspace.manifest.v1\",\n");
    fprintf(f, "  \"schema\": \"workspace-runtime.v1\",\n");
    fprintf(f, "  \"ws_id\": \"%s\",\n", info->ws_id);
    fprintf(f, "  \"state\": \"%s\",\n", info->state[0] ? info->state : "created");
    fprintf(f, "  \"created_at\": %ld,\n", info->created_at);
    fprintf(f, "  \"updated_at\": %ld,\n", info->updated_at);
    fprintf(f, "  \"layout\": \"v3\",\n");
    fprintf(f, "  \"containment_layout\": \"%s\",\n", info->containment_layout[0] ? info->containment_layout : "v1");
    fprintf(f, "  \"root_path\": \"%s\",\n", info->root_path);
    fprintf(f, "  \"identity\": {\"workspace_id\": \"%s\", \"workspace_alias\": \"%s\", \"workspace_root\": \"%s\"},\n",
            info->ws_id, info->workspace_alias[0] ? info->workspace_alias : info->ws_id, info->root_path);
    fprintf(f, "  \"lifecycle\": {\"workspace_state\": \"%s\", \"created_at\": %ld, \"activated_at\": %ld, \"last_attached_at\": %ld, \"last_updated_at\": %ld},\n",
            info->state[0] ? info->state : "created", info->created_at, info->activated_at, info->last_attached_at, info->updated_at);
    fprintf(f, "  \"root_model\": {\"workspace_store_root\": \"%s\", \"workspace_root\": \"%s\", \"runtime_state_root\": \"%s\", \"metadata_root\": \"%s\", \"state_root\": \"%s\", \"traces_root\": \"%s\", \"artifacts_root\": \"%s\", \"runtime_local_root\": \"%s\", \"root_anchor_mode\": \"%s\"},\n",
            info->workspace_store_root, info->root_path, info->runtime_state_root, info->metadata_root, info->state_root, info->traces_root, info->artifacts_root, info->runtime_local_root, info->root_anchor_mode[0] ? info->root_anchor_mode : "managed_default_root");
    fprintf(f, "  \"containment\": {\"ready\": %s, \"state_surface\": \"%s\", \"traces_index\": \"%s\", \"artifacts_index\": \"%s\", \"runtime_surface\": \"%s\", \"binding_surface\": \"%s\"},\n",
            info->containment_ready ? "true" : "false", info->state_surface_path, info->traces_index_path, info->artifacts_index_path, info->runtime_surface_path, info->binding_state_path);
    fprintf(f, "  \"security_envelope\": {\n");
    fprintf(f, "    \"security_envelope_version\": \"%s\",\n", info->security_envelope_version[0] ? info->security_envelope_version : "v1");
    fprintf(f, "    \"security_level_declared\": \"%s\",\n", info->security_level_declared[0] ? info->security_level_declared : "scoped");
    fprintf(f, "    \"security_level_effective\": \"%s\",\n", info->security_level_effective[0] ? info->security_level_effective : "logical");
    fprintf(f, "    \"security_enforcement_mode\": \"%s\",\n", info->security_enforcement_mode[0] ? info->security_enforcement_mode : "runtime_scoped");
    fprintf(f, "    \"security_backend_mode\": \"%s\",\n", info->security_backend_mode[0] ? info->security_backend_mode : "none");
    fprintf(f, "    \"scopes\": {\"process\": %s, \"filesystem\": %s, \"socket\": %s, \"network\": %s, \"resource\": %s, \"privilege\": %s, \"runtime_route\": %s, \"binding\": %s},\n",
            info->scope_process ? "true" : "false",
            info->scope_filesystem ? "true" : "false",
            info->scope_socket ? "true" : "false",
            info->scope_network ? "true" : "false",
            info->scope_resource ? "true" : "false",
            info->scope_privilege ? "true" : "false",
            info->scope_runtime_route ? "true" : "false",
            info->scope_binding ? "true" : "false");
    fprintf(f, "    \"capabilities\": {\"sandbox_ready\": %s, \"hardened_fs\": %s, \"process_isolation\": %s, \"network_policy\": %s}\n",
            info->capability_sandbox_ready ? "true" : "false",
            info->capability_hardened_fs ? "true" : "false",
            info->capability_process_isolation ? "true" : "false",
            info->capability_network_policy ? "true" : "false");
    fprintf(f, "  },\n");
    fprintf(f, "  \"execution_profile\": {\n");
    fprintf(f, "    \"execution_mode_requested\": \"%s\",\n", info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped");
    fprintf(f, "    \"execution_mode_effective\": \"%s\",\n", info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped");
    fprintf(f, "    \"execution_mode_degraded\": %s,\n", info->execution_mode_degraded ? "true" : "false");
    fprintf(f, "    \"execution_degraded_reason\": \"%s\",\n", info->execution_degraded_reason[0] ? info->execution_degraded_reason : "none");
    fprintf(f, "    \"execution_unsupported_scopes\": \"%s\",\n", info->execution_unsupported_scopes[0] ? info->execution_unsupported_scopes : "none");
    fprintf(f, "    \"execution_advisory_scopes\": \"%s\",\n", info->execution_advisory_scopes[0] ? info->execution_advisory_scopes : "none");
    fprintf(f, "    \"process_intent\": \"%s\",\n", info->process_intent[0] ? info->process_intent : "shared_runtime");
    fprintf(f, "    \"channel_mode\": \"%s\",\n", info->channel_mode[0] ? info->channel_mode : "global_control_scoped_route");
    fprintf(f, "    \"artifact_policy_mode\": \"%s\",\n", info->artifact_policy_mode[0] ? info->artifact_policy_mode : "workspace_owned");
    fprintf(f, "    \"network_intent\": \"%s\",\n", info->network_intent[0] ? info->network_intent : "advisory_none");
    fprintf(f, "    \"resource_intent\": \"%s\",\n", info->resource_intent[0] ? info->resource_intent : "advisory_none");
    fprintf(f, "    \"privilege_intent\": \"%s\",\n", info->privilege_intent[0] ? info->privilege_intent : "inherited_host");
    fprintf(f, "    \"attach_descriptor_ref\": \"%s\",\n", info->attach_descriptor_ref);
    fprintf(f, "    \"execution_profile_ref\": \"%s\"\n", info->execution_profile_ref);
    fprintf(f, "  },\n");
    fprintf(f, "  \"boundaries\": {\"execution_boundary\": true, \"context_boundary\": true, \"policy_boundary\": true, \"runtime_binding_boundary\": true, \"shell_binding_scope\": \"session\"},\n");
    fprintf(f, "  \"binding\": {\"session_binding\": \"%s\", \"runtime_attached\": %s, \"runtime_endpoint\": \"\", \"control_plane_attached\": %s},\n",
            info->session_binding, info->runtime_attached ? "true" : "false", info->control_plane_attached ? "true" : "false");
    fprintf(f, "  \"declared_context\": {\"declared_control_family\": \"%s\", \"declared_specialization\": \"%s\", \"declared_profile\": \"\", \"declared_context_source\": \"%s\"},\n",
            info->declared_control_family, info->declared_specialization, info->declared_context_source[0] ? info->declared_context_source : "unset");
    fprintf(f, "  \"inferred_context\": {\"last_inferred_family\": \"%s\", \"last_inferred_specialization\": \"%s\", \"last_inference_confidence\": %.3f},\n",
            info->inferred_family, info->inferred_specialization, info->inferred_confidence);
    {
        char evt_declared[96];
        char evt_business[96];
        char evt_enforcement[96];
        char evt_stage[48];
        char op_summary[192];
        const char *review_state;
        int evt_external = 0;
        yai_session_workspace_event_semantics(info,
                                              evt_declared, sizeof(evt_declared),
                                              evt_business, sizeof(evt_business),
                                              evt_enforcement, sizeof(evt_enforcement),
                                              evt_stage, sizeof(evt_stage),
                                              &evt_external);
        review_state = yai_workspace_review_state_from_effect(info->last_effect_summary);
        yai_workspace_operational_summary(evt_stage, evt_business, info->last_effect_summary, op_summary, sizeof(op_summary));
        fprintf(f, "  \"effective_state\": {\"effective_stack_ref\": \"%s\", \"effective_overlays_ref\": \"%s\", \"policy_attachments\": \"%s\", \"last_effect_summary\": \"%s\", \"last_authority_summary\": \"%s\", \"last_evidence_summary\": \"%s\", \"last_event_ref\": \"%s\", \"business_specialization\": \"%s\", \"enforcement_specialization\": \"%s\", \"flow_stage\": \"%s\", \"external_effect_boundary\": %s, \"review_state\": \"%s\", \"operational_summary\": \"%s\"},\n",
                info->effective_stack_ref,
                info->effective_overlays_ref,
                info->policy_attachments_csv,
                info->last_effect_summary,
                info->last_authority_summary,
                info->last_evidence_summary,
                info->last_resolution_trace_ref,
                evt_business,
                evt_enforcement,
                evt_stage,
                evt_external ? "true" : "false",
                review_state,
                op_summary);
    }
    fprintf(f, "  \"runtime\": {\"isolation_mode\": \"%s\", \"debug_mode\": %s, \"last_resolution_trace_ref\": \"%s\"},\n",
            info->isolation_mode[0] ? info->isolation_mode : "process", info->debug_mode ? "true" : "false", info->last_resolution_trace_ref);
    fprintf(f, "  \"inspect\": {\"last_resolution_summary\": \"%s\"},\n", info->last_resolution_summary);
    fprintf(f, "  \"runtime_owner\": \"yai\",\n");
    fprintf(f, "  \"attachments\": [");
    if (info->policy_attachments_csv[0])
    {
        char refs[sizeof(info->policy_attachments_csv)];
        char *tok;
        char *save = NULL;
        int first = 1;
        snprintf(refs, sizeof(refs), "%s", info->policy_attachments_csv);
        tok = strtok_r(refs, ",", &save);
        while (tok)
        {
            if (tok[0])
            {
                fprintf(f, "%s{\"kind\":\"policy_object\",\"id\":\"%s\",\"active\":true}", first ? "" : ",", tok);
                first = 0;
            }
            tok = strtok_r(NULL, ",", &save);
        }
    }
    fprintf(f, "],\n");
    fprintf(f, "  \"capabilities\": {\"workspace_scope\": true, \"attachment_ready\": true}\n");
    fprintf(f, "}\n");
    fclose(f);
    return 0;
}

static int yai_workspace_write_manifest_ws_id(const char *ws_id, const yai_workspace_runtime_info_t *info)
{
    char manifest[MAX_PATH_LEN];
    if (!ws_id || !info)
        return -1;
    if (yai_workspace_manifest_path(ws_id, manifest, sizeof(manifest)) != 0)
        return -1;
    return yai_workspace_write_manifest_path(manifest, info);
}

static int yai_workspace_write_containment_surfaces(const yai_workspace_runtime_info_t *info)
{
    FILE *f;
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char op_summary[192];
    const char *review_state;
    int evt_external = 0;
    if (!info || !info->ws_id[0])
        return -1;
    yai_session_workspace_event_semantics(info,
                                          evt_declared, sizeof(evt_declared),
                                          evt_business, sizeof(evt_business),
                                          evt_enforcement, sizeof(evt_enforcement),
                                          evt_stage, sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info->last_resolution_trace_ref[0] ? "evt-" : "none",
             info->last_resolution_trace_ref[0] ? info->last_resolution_trace_ref : "");
    review_state = yai_workspace_review_state_from_effect(info->last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info->last_effect_summary, op_summary, sizeof(op_summary));

    f = fopen(info->state_surface_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.state.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"declared\": {\"family\": \"%s\", \"specialization\": \"%s\", \"source\": \"%s\"},\n"
            "  \"inferred\": {\"family\": \"%s\", \"specialization\": \"%s\", \"confidence\": %.3f},\n"
            "  \"effective\": {\"stack_ref\": \"%s\", \"overlays_ref\": \"%s\", \"effect\": \"%s\", \"authority\": \"%s\", \"evidence\": \"%s\"},\n"
            "  \"governance\": {\"policy_attachments\": \"%s\", \"policy_attachment_count\": %d},\n"
            "  \"event_surface\": {\"event_id\": \"%s\", \"flow_stage\": \"%s\", \"declared_scenario_specialization\": \"%s\", \"business_specialization\": \"%s\", \"enforcement_specialization\": \"%s\", \"external_effect_boundary\": %s},\n"
            "  \"operational_state\": {\"last_event_ref\": \"%s\", \"last_flow_stage\": \"%s\", \"last_business_specialization\": \"%s\", \"last_enforcement_specialization\": \"%s\", \"last_effect\": \"%s\", \"last_authority\": \"%s\", \"last_evidence\": \"%s\", \"last_trace_ref\": \"%s\", \"review_state\": \"%s\", \"operational_summary\": \"%s\"},\n"
            "  \"inspect\": {\"last_summary\": \"%s\", \"last_trace_ref\": \"%s\"},\n"
            "  \"refs\": {\"trace_index\": \"%s\", \"artifact_index\": \"%s\", \"runtime_state\": \"%s\"}\n"
            "}\n",
            info->ws_id,
            info->declared_control_family,
            info->declared_specialization,
            info->declared_context_source[0] ? info->declared_context_source : "unset",
            info->inferred_family,
            info->inferred_specialization,
            info->inferred_confidence,
            info->effective_stack_ref,
            info->effective_overlays_ref,
            info->last_effect_summary,
            info->last_authority_summary,
            info->last_evidence_summary,
            info->policy_attachments_csv,
            info->policy_attachment_count,
            evt_id,
            evt_stage,
            evt_declared,
            evt_business,
            evt_enforcement,
            evt_external ? "true" : "false",
            evt_id,
            evt_stage,
            evt_business,
            evt_enforcement,
            info->last_effect_summary,
            info->last_authority_summary,
            info->last_evidence_summary,
            info->last_resolution_trace_ref,
            review_state,
            op_summary,
            info->last_resolution_summary,
            info->last_resolution_trace_ref,
            info->traces_index_path,
            info->artifacts_index_path,
            info->runtime_surface_path);
    fclose(f);

    f = fopen(info->traces_index_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.traces.index.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"ownership\": \"workspace-owned\",\n"
            "  \"entries\": [\n"
            "    {\"trace_ref\": \"%s\", \"summary\": \"%s\"}\n"
            "  ]\n"
            "}\n",
            info->ws_id,
            info->last_resolution_trace_ref,
            info->last_resolution_summary);
    fclose(f);

    f = fopen(info->artifacts_index_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.artifacts.index.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"ownership\": \"workspace-owned\",\n"
            "  \"entries\": []\n"
            "}\n",
            info->ws_id);
    fclose(f);

    f = fopen(info->runtime_surface_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.runtime.state.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"routing\": {\"scope\": \"workspace\", \"namespace\": \"%s\"},\n"
            "  \"attachments\": {\"runtime_attached\": %s, \"control_plane_attached\": %s},\n"
            "  \"security\": {\"level_effective\": \"%s\", \"enforcement_mode\": \"%s\", \"backend_mode\": \"%s\"},\n"
            "  \"execution\": {\"mode_requested\": \"%s\", \"mode_effective\": \"%s\", \"degraded\": %s, \"degraded_reason\": \"%s\", \"unsupported_scopes\": \"%s\"},\n"
            "  \"isolation_mode\": \"%s\",\n"
            "  \"debug_mode\": %s\n"
            "}\n",
            info->ws_id,
            info->workspace_namespace,
            info->runtime_attached ? "true" : "false",
            info->control_plane_attached ? "true" : "false",
            info->security_level_effective[0] ? info->security_level_effective : "logical",
            info->security_enforcement_mode[0] ? info->security_enforcement_mode : "runtime_scoped",
            info->security_backend_mode[0] ? info->security_backend_mode : "none",
            info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped",
            info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped",
            info->execution_mode_degraded ? "true" : "false",
            info->execution_degraded_reason[0] ? info->execution_degraded_reason : "none",
            info->execution_unsupported_scopes[0] ? info->execution_unsupported_scopes : "none",
            info->isolation_mode[0] ? info->isolation_mode : "process",
            info->debug_mode ? "true" : "false");
    fclose(f);

    f = fopen(info->attach_descriptor_ref, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.attach.descriptor.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"binding_scope\": \"session\",\n"
            "  \"runtime_attached\": %s,\n"
            "  \"control_plane_attached\": %s,\n"
            "  \"channel_mode\": \"%s\",\n"
            "  \"artifact_policy_mode\": \"%s\",\n"
            "  \"process_intent\": \"%s\",\n"
            "  \"mode_requested\": \"%s\",\n"
            "  \"mode_effective\": \"%s\"\n"
            "}\n",
            info->ws_id,
            info->runtime_attached ? "true" : "false",
            info->control_plane_attached ? "true" : "false",
            info->channel_mode[0] ? info->channel_mode : "global_control_scoped_route",
            info->artifact_policy_mode[0] ? info->artifact_policy_mode : "workspace_owned",
            info->process_intent[0] ? info->process_intent : "shared_runtime",
            info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped",
            info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped");
    fclose(f);

    f = fopen(info->execution_profile_ref, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.execution.profile.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"mode_requested\": \"%s\",\n"
            "  \"mode_effective\": \"%s\",\n"
            "  \"degraded\": %s,\n"
            "  \"degraded_reason\": \"%s\",\n"
            "  \"unsupported_scopes\": \"%s\",\n"
            "  \"advisory_scopes\": \"%s\",\n"
            "  \"backend_mode\": \"%s\",\n"
            "  \"enforcement_mode\": \"%s\",\n"
            "  \"intents\": {\"network\": \"%s\", \"resource\": \"%s\", \"privilege\": \"%s\"}\n"
            "}\n",
            info->ws_id,
            info->execution_mode_requested[0] ? info->execution_mode_requested : "scoped",
            info->execution_mode_effective[0] ? info->execution_mode_effective : "scoped",
            info->execution_mode_degraded ? "true" : "false",
            info->execution_degraded_reason[0] ? info->execution_degraded_reason : "none",
            info->execution_unsupported_scopes[0] ? info->execution_unsupported_scopes : "none",
            info->execution_advisory_scopes[0] ? info->execution_advisory_scopes : "none",
            info->security_backend_mode[0] ? info->security_backend_mode : "none",
            info->security_enforcement_mode[0] ? info->security_enforcement_mode : "runtime_scoped",
            info->network_intent[0] ? info->network_intent : "advisory_none",
            info->resource_intent[0] ? info->resource_intent : "advisory_none",
            info->privilege_intent[0] ? info->privilege_intent : "inherited_host");
    fclose(f);

    f = fopen(info->binding_state_path, "w");
    if (!f)
        return -1;
    fprintf(f,
            "{\n"
            "  \"type\": \"yai.workspace.binding.state.v1\",\n"
            "  \"workspace_id\": \"%s\",\n"
            "  \"session_binding\": \"%s\",\n"
            "  \"binding_scope\": \"session\",\n"
            "  \"binding_valid\": %s,\n"
            "  \"boundary_reason\": \"%s\"\n"
            "}\n",
            info->ws_id,
            info->session_binding,
            info->namespace_valid ? "true" : "false",
            info->boundary_reason[0] ? info->boundary_reason : "none");
    fclose(f);
    return 0;
}

int yai_session_read_workspace_info(const char *ws_id, yai_workspace_runtime_info_t *out)
{
    char manifest[MAX_PATH_LEN];
    FILE *f;
    char buf[YAI_WS_JSON_IO_CAP];
    size_t r;

    if (!ws_id || !out)
        return -1;

    memset(out, 0, sizeof(*out));
    snprintf(out->ws_id, sizeof(out->ws_id), "%s", ws_id);
    snprintf(out->workspace_namespace, sizeof(out->workspace_namespace), "ws/%s", ws_id);
    out->namespace_valid = 1;
    snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "none");
    snprintf(out->state, sizeof(out->state), "missing");
    snprintf(out->layout, sizeof(out->layout), "v3");
    snprintf(out->workspace_alias, sizeof(out->workspace_alias), "%s", ws_id);
    snprintf(out->isolation_mode, sizeof(out->isolation_mode), "process");
    snprintf(out->root_anchor_mode, sizeof(out->root_anchor_mode), "%s", "managed_default_root");
    snprintf(out->containment_layout, sizeof(out->containment_layout), "%s", "v1");
    out->containment_ready = 0;
    yai_workspace_security_defaults(out);
    if (yai_workspace_store_root_path(out->workspace_store_root, sizeof(out->workspace_store_root)) != 0)
        out->workspace_store_root[0] = '\0';
    if (yai_workspace_runtime_state_root_path(ws_id, out->runtime_state_root, sizeof(out->runtime_state_root)) != 0)
        out->runtime_state_root[0] = '\0';
    if (yai_workspace_metadata_root_path(ws_id, out->metadata_root, sizeof(out->metadata_root)) != 0)
        out->metadata_root[0] = '\0';
    (void)yai_workspace_containment_surface_paths(out);

    if (yai_workspace_manifest_path(ws_id, manifest, sizeof(manifest)) != 0)
        return -1;

    f = fopen(manifest, "r");
    if (!f)
        return -1;

    r = fread(buf, 1, sizeof(buf) - 1, f);
    fclose(f);
    buf[r] = '\0';

    out->exists = 1;
    (void)yai_session_extract_json_string(buf, "state", out->state, sizeof(out->state));
    (void)yai_session_extract_json_string(buf, "layout", out->layout, sizeof(out->layout));
    (void)yai_session_extract_json_string(buf, "containment_layout", out->containment_layout, sizeof(out->containment_layout));
    (void)yai_session_extract_json_string(buf, "security_envelope_version", out->security_envelope_version, sizeof(out->security_envelope_version));
    (void)yai_session_extract_json_string(buf, "security_level_declared", out->security_level_declared, sizeof(out->security_level_declared));
    (void)yai_session_extract_json_string(buf, "security_level_effective", out->security_level_effective, sizeof(out->security_level_effective));
    (void)yai_session_extract_json_string(buf, "security_enforcement_mode", out->security_enforcement_mode, sizeof(out->security_enforcement_mode));
    (void)yai_session_extract_json_string(buf, "security_backend_mode", out->security_backend_mode, sizeof(out->security_backend_mode));
    (void)yai_session_extract_json_string(buf, "execution_mode_requested", out->execution_mode_requested, sizeof(out->execution_mode_requested));
    (void)yai_session_extract_json_string(buf, "execution_mode_effective", out->execution_mode_effective, sizeof(out->execution_mode_effective));
    (void)yai_session_extract_json_string(buf, "execution_degraded_reason", out->execution_degraded_reason, sizeof(out->execution_degraded_reason));
    (void)yai_session_extract_json_string(buf, "execution_unsupported_scopes", out->execution_unsupported_scopes, sizeof(out->execution_unsupported_scopes));
    (void)yai_session_extract_json_string(buf, "execution_advisory_scopes", out->execution_advisory_scopes, sizeof(out->execution_advisory_scopes));
    (void)yai_session_extract_json_string(buf, "process_intent", out->process_intent, sizeof(out->process_intent));
    (void)yai_session_extract_json_string(buf, "channel_mode", out->channel_mode, sizeof(out->channel_mode));
    (void)yai_session_extract_json_string(buf, "artifact_policy_mode", out->artifact_policy_mode, sizeof(out->artifact_policy_mode));
    (void)yai_session_extract_json_string(buf, "network_intent", out->network_intent, sizeof(out->network_intent));
    (void)yai_session_extract_json_string(buf, "resource_intent", out->resource_intent, sizeof(out->resource_intent));
    (void)yai_session_extract_json_string(buf, "privilege_intent", out->privilege_intent, sizeof(out->privilege_intent));
    (void)yai_session_extract_json_string(buf, "attach_descriptor_ref", out->attach_descriptor_ref, sizeof(out->attach_descriptor_ref));
    (void)yai_session_extract_json_string(buf, "execution_profile_ref", out->execution_profile_ref, sizeof(out->execution_profile_ref));
    (void)yai_session_extract_json_string(buf, "root_path", out->root_path, sizeof(out->root_path));
    (void)yai_session_extract_json_string(buf, "workspace_store_root", out->workspace_store_root, sizeof(out->workspace_store_root));
    (void)yai_session_extract_json_string(buf, "runtime_state_root", out->runtime_state_root, sizeof(out->runtime_state_root));
    (void)yai_session_extract_json_string(buf, "metadata_root", out->metadata_root, sizeof(out->metadata_root));
    (void)yai_session_extract_json_string(buf, "state_root", out->state_root, sizeof(out->state_root));
    (void)yai_session_extract_json_string(buf, "traces_root", out->traces_root, sizeof(out->traces_root));
    (void)yai_session_extract_json_string(buf, "artifacts_root", out->artifacts_root, sizeof(out->artifacts_root));
    (void)yai_session_extract_json_string(buf, "runtime_local_root", out->runtime_local_root, sizeof(out->runtime_local_root));
    (void)yai_session_extract_json_string(buf, "state_surface", out->state_surface_path, sizeof(out->state_surface_path));
    (void)yai_session_extract_json_string(buf, "traces_index", out->traces_index_path, sizeof(out->traces_index_path));
    (void)yai_session_extract_json_string(buf, "artifacts_index", out->artifacts_index_path, sizeof(out->artifacts_index_path));
    (void)yai_session_extract_json_string(buf, "runtime_surface", out->runtime_surface_path, sizeof(out->runtime_surface_path));
    (void)yai_session_extract_json_string(buf, "binding_surface", out->binding_state_path, sizeof(out->binding_state_path));
    (void)yai_session_extract_json_string(buf, "root_anchor_mode", out->root_anchor_mode, sizeof(out->root_anchor_mode));
    (void)yai_session_extract_json_string(buf, "workspace_alias", out->workspace_alias, sizeof(out->workspace_alias));
    (void)yai_session_extract_json_string(buf, "session_binding", out->session_binding, sizeof(out->session_binding));
    (void)yai_session_extract_json_string(buf, "declared_control_family", out->declared_control_family, sizeof(out->declared_control_family));
    (void)yai_session_extract_json_string(buf, "declared_specialization", out->declared_specialization, sizeof(out->declared_specialization));
    (void)yai_session_extract_json_string(buf, "declared_context_source", out->declared_context_source, sizeof(out->declared_context_source));
    (void)yai_session_extract_json_string(buf, "last_inferred_family", out->inferred_family, sizeof(out->inferred_family));
    (void)yai_session_extract_json_string(buf, "last_inferred_specialization", out->inferred_specialization, sizeof(out->inferred_specialization));
    out->inferred_confidence = 0.0;
    (void)yai_session_extract_json_double(buf, "last_inference_confidence", &out->inferred_confidence);
    (void)yai_session_extract_json_string(buf, "effective_stack_ref", out->effective_stack_ref, sizeof(out->effective_stack_ref));
    (void)yai_session_extract_json_string(buf, "effective_overlays_ref", out->effective_overlays_ref, sizeof(out->effective_overlays_ref));
    (void)yai_session_extract_json_string(buf, "policy_attachments", out->policy_attachments_csv, sizeof(out->policy_attachments_csv));
    out->policy_attachment_count = yai_policy_attachment_csv_count(out->policy_attachments_csv);
    (void)yai_session_extract_json_string(buf, "last_effect_summary", out->last_effect_summary, sizeof(out->last_effect_summary));
    (void)yai_session_extract_json_string(buf, "last_authority_summary", out->last_authority_summary, sizeof(out->last_authority_summary));
    (void)yai_session_extract_json_string(buf, "last_evidence_summary", out->last_evidence_summary, sizeof(out->last_evidence_summary));
    (void)yai_session_extract_json_string(buf, "last_resolution_summary", out->last_resolution_summary, sizeof(out->last_resolution_summary));
    (void)yai_session_extract_json_string(buf, "isolation_mode", out->isolation_mode, sizeof(out->isolation_mode));
    (void)yai_session_extract_json_string(buf, "last_resolution_trace_ref", out->last_resolution_trace_ref, sizeof(out->last_resolution_trace_ref));
    (void)yai_session_extract_json_long(buf, "created_at", &out->created_at);
    (void)yai_session_extract_json_long(buf, "activated_at", &out->activated_at);
    (void)yai_session_extract_json_long(buf, "last_attached_at", &out->last_attached_at);
    if (yai_session_extract_json_long(buf, "updated_at", &out->updated_at) != 0)
        out->updated_at = out->created_at;
    (void)yai_session_extract_json_bool(buf, "runtime_attached", &out->runtime_attached);
    (void)yai_session_extract_json_bool(buf, "control_plane_attached", &out->control_plane_attached);
    (void)yai_session_extract_json_bool(buf, "debug_mode", &out->debug_mode);
    (void)yai_session_extract_json_bool(buf, "scope_process", &out->scope_process);
    (void)yai_session_extract_json_bool(buf, "scope_filesystem", &out->scope_filesystem);
    (void)yai_session_extract_json_bool(buf, "scope_socket", &out->scope_socket);
    (void)yai_session_extract_json_bool(buf, "scope_network", &out->scope_network);
    (void)yai_session_extract_json_bool(buf, "scope_resource", &out->scope_resource);
    (void)yai_session_extract_json_bool(buf, "scope_privilege", &out->scope_privilege);
    (void)yai_session_extract_json_bool(buf, "scope_runtime_route", &out->scope_runtime_route);
    (void)yai_session_extract_json_bool(buf, "scope_binding", &out->scope_binding);
    (void)yai_session_extract_json_bool(buf, "execution_mode_degraded", &out->execution_mode_degraded);
    (void)yai_session_extract_json_bool(buf, "capability_sandbox_ready", &out->capability_sandbox_ready);
    (void)yai_session_extract_json_bool(buf, "capability_hardened_fs", &out->capability_hardened_fs);
    (void)yai_session_extract_json_bool(buf, "capability_process_isolation", &out->capability_process_isolation);
    (void)yai_session_extract_json_bool(buf, "capability_network_policy", &out->capability_network_policy);

    if (out->root_path[0] == '\0')
    {
        if (out->workspace_store_root[0])
            snprintf(out->root_path, sizeof(out->root_path), "%s/%s", out->workspace_store_root, ws_id);
    }
    out->containment_ready = yai_session_path_exists(out->state_surface_path) &&
                             yai_session_path_exists(out->traces_index_path) &&
                             yai_session_path_exists(out->artifacts_index_path) &&
                             yai_session_path_exists(out->runtime_surface_path) &&
                             yai_session_path_exists(out->binding_state_path) &&
                             yai_session_path_exists(out->attach_descriptor_ref) &&
                             yai_session_path_exists(out->execution_profile_ref);
    if (!out->containment_ready)
    {
        out->namespace_valid = 0;
        if (strcmp(out->boundary_reason, "none") == 0)
            snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "containment_surface_missing");
    }
    if (!yai_workspace_security_level_is_valid(out->security_level_declared))
        snprintf(out->security_level_declared, sizeof(out->security_level_declared), "%s", "scoped");
    yai_workspace_security_recompute_effective(out);

    if (out->session_binding[0] && strcmp(out->session_binding, ws_id) != 0)
    {
        out->namespace_valid = 0;
        snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "session_binding_mismatch");
        out->session_binding[0] = '\0';
    }
    {
        char expected_runtime_root[MAX_PATH_LEN];
        char expected_metadata_root[MAX_PATH_LEN];
        if (yai_workspace_runtime_state_root_path(ws_id, expected_runtime_root, sizeof(expected_runtime_root)) == 0)
        {
            if (!yai_is_ws_runtime_path_valid(ws_id, out->runtime_state_root, expected_runtime_root))
            {
                out->namespace_valid = 0;
                if (strcmp(out->boundary_reason, "none") == 0)
                    snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "runtime_state_root_mismatch");
                snprintf(out->runtime_state_root, sizeof(out->runtime_state_root), "%s", expected_runtime_root);
            }
        }
        if (yai_workspace_metadata_root_path(ws_id, expected_metadata_root, sizeof(expected_metadata_root)) == 0)
        {
            if (!yai_is_ws_runtime_path_valid(ws_id, out->metadata_root, expected_metadata_root))
            {
                out->namespace_valid = 0;
                if (strcmp(out->boundary_reason, "none") == 0)
                    snprintf(out->boundary_reason, sizeof(out->boundary_reason), "%s", "metadata_root_mismatch");
                snprintf(out->metadata_root, sizeof(out->metadata_root), "%s", expected_metadata_root);
            }
        }
    }
    yai_workspace_fill_shell_relation(out);

    return 0;
}

int yai_session_build_workspace_list_json(char *out, size_t out_cap, int *count_out)
{
    char run_dir[MAX_PATH_LEN];
    DIR *d;
    size_t used = 0;
    int first = 1;
    int count = 0;
    struct dirent *ent;
    int n;

    if (!out || out_cap == 0)
        return -1;
    if (count_out)
        *count_out = 0;

    if (yai_session_build_run_path(run_dir, sizeof(run_dir), "") != 0)
        return -1;

    d = opendir(run_dir);
    if (!d)
        return -1;

    n = snprintf(out, out_cap, "[");
    if (n <= 0 || (size_t)n >= out_cap)
    {
        closedir(d);
        return -1;
    }
    used = (size_t)n;

    while ((ent = readdir(d)) != NULL)
    {
        yai_workspace_runtime_info_t info;
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0)
            continue;
        if (yai_session_read_workspace_info(ent->d_name, &info) != 0 || !info.exists)
            continue;

        n = snprintf(out + used,
                     out_cap - used,
                     "%s{\"ws_id\":\"%s\",\"workspace_alias\":\"%s\",\"state\":\"%s\",\"root_path\":\"%s\",\"runtime_attached\":%s}",
                     first ? "" : ",",
                     info.ws_id,
                     info.workspace_alias,
                     info.state,
                     info.root_path,
                     info.runtime_attached ? "true" : "false");
        if (n <= 0 || (size_t)n >= (out_cap - used))
        {
            closedir(d);
            return -1;
        }

        used += (size_t)n;
        first = 0;
        count++;
    }

    closedir(d);

    n = snprintf(out + used, out_cap - used, "]");
    if (n <= 0 || (size_t)n >= (out_cap - used))
        return -1;

    if (count_out)
        *count_out = count;
    return 0;
}

int yai_session_handle_workspace_action(
    const char *ws_id,
    const char *action,
    const char *root_path_opt,
    const char *security_level_opt,
    yai_workspace_runtime_info_t *info_out)
{
    const char *home = yai_get_home();
    char yai_dir[MAX_PATH_LEN];
    char run_dir[MAX_PATH_LEN];
    char ws_dir[MAX_PATH_LEN];
    char auth_dir[MAX_PATH_LEN];
    char events_dir[MAX_PATH_LEN];
    char exec_dir[MAX_PATH_LEN];
    char logs_dir[MAX_PATH_LEN];
    char metadata_dir[MAX_PATH_LEN];
    char state_dir[MAX_PATH_LEN];
    char traces_dir[MAX_PATH_LEN];
    char artifacts_dir[MAX_PATH_LEN];
    char runtime_dir[MAX_PATH_LEN];
    char root_path[MAX_PATH_LEN] = {0};
    char root_anchor_mode[32] = {0};
    char manifest_path[MAX_PATH_LEN];
    yai_workspace_runtime_info_t info;
    time_t now = time(NULL);

    if (!home || !ws_id || !action)
        return -1;
    if (!yai_ws_id_is_valid(ws_id))
        return -1;

    if (snprintf(yai_dir, sizeof(yai_dir), "%s/.yai", home) <= 0 ||
        snprintf(run_dir, sizeof(run_dir), "%s/.yai/run", home) <= 0 ||
        snprintf(ws_dir, sizeof(ws_dir), "%s/.yai/run/%s", home, ws_id) <= 0 ||
        snprintf(auth_dir, sizeof(auth_dir), "%s/authority", ws_dir) <= 0 ||
        snprintf(events_dir, sizeof(events_dir), "%s/events", ws_dir) <= 0 ||
        snprintf(exec_dir, sizeof(exec_dir), "%s/exec", ws_dir) <= 0 ||
        snprintf(logs_dir, sizeof(logs_dir), "%s/logs", ws_dir) <= 0 ||
        snprintf(metadata_dir, sizeof(metadata_dir), "%s/metadata", ws_dir) <= 0 ||
        snprintf(state_dir, sizeof(state_dir), "%s/state", ws_dir) <= 0 ||
        snprintf(traces_dir, sizeof(traces_dir), "%s/traces", ws_dir) <= 0 ||
        snprintf(artifacts_dir, sizeof(artifacts_dir), "%s/artifacts", ws_dir) <= 0 ||
        snprintf(runtime_dir, sizeof(runtime_dir), "%s/runtime", ws_dir) <= 0)
        return -1;

    if (strcmp(action, "destroy") == 0)
    {
        if (info_out)
        {
            (void)yai_session_read_workspace_info(ws_id, info_out);
            snprintf(info_out->ws_id, sizeof(info_out->ws_id), "%s", ws_id);
            info_out->exists = 0;
            snprintf(info_out->state, sizeof(info_out->state), "destroyed");
            info_out->updated_at = (long)now;
        }
        return remove_tree(ws_dir);
    }

    if (strcmp(action, "reset") == 0)
    {
        (void)remove_tree(ws_dir);
        action = "create";
    }

    if (strcmp(action, "create") != 0)
        return -2;

    if (yai_workspace_resolve_root_path(ws_id,
                                        root_path_opt,
                                        root_anchor_mode,
                                        sizeof(root_anchor_mode),
                                        root_path,
                                        sizeof(root_path)) != 0)
        return -1;

    if (mkdir_if_missing(yai_dir, 0755) != 0 ||
        mkdir_if_missing(run_dir, 0755) != 0 ||
        mkdir_if_missing(ws_dir, 0755) != 0 ||
        mkdir_if_missing(auth_dir, 0755) != 0 ||
        mkdir_if_missing(events_dir, 0755) != 0 ||
        mkdir_if_missing(exec_dir, 0755) != 0 ||
        mkdir_if_missing(logs_dir, 0755) != 0 ||
        mkdir_if_missing(metadata_dir, 0755) != 0 ||
        mkdir_if_missing(state_dir, 0755) != 0 ||
        mkdir_if_missing(traces_dir, 0755) != 0 ||
        mkdir_if_missing(artifacts_dir, 0755) != 0 ||
        mkdir_if_missing(runtime_dir, 0755) != 0 ||
        mkdir_parents(root_path, 0755) != 0)
        return -1;

    memset(&info, 0, sizeof(info));
    snprintf(info.ws_id, sizeof(info.ws_id), "%s", ws_id);
    snprintf(info.workspace_alias, sizeof(info.workspace_alias), "%s", ws_id);
    snprintf(info.state, sizeof(info.state), "%s", "created");
    snprintf(info.layout, sizeof(info.layout), "%s", "v3");
    snprintf(info.containment_layout, sizeof(info.containment_layout), "%s", "v1");
    yai_workspace_security_defaults(&info);
    if (security_level_opt && security_level_opt[0])
    {
        if (!yai_workspace_security_level_is_valid(security_level_opt))
            return -1;
        snprintf(info.security_level_declared, sizeof(info.security_level_declared), "%s", security_level_opt);
    }
    snprintf(info.root_path, sizeof(info.root_path), "%s", root_path);
    snprintf(info.root_anchor_mode, sizeof(info.root_anchor_mode), "%s", root_anchor_mode[0] ? root_anchor_mode : "managed_default_root");
    if (yai_workspace_store_root_path(info.workspace_store_root, sizeof(info.workspace_store_root)) != 0)
        return -1;
    if (yai_workspace_runtime_state_root_path(ws_id, info.runtime_state_root, sizeof(info.runtime_state_root)) != 0)
        return -1;
    if (yai_workspace_metadata_root_path(ws_id, info.metadata_root, sizeof(info.metadata_root)) != 0)
        return -1;
    if (yai_workspace_containment_surface_paths(&info) != 0)
        return -1;
    snprintf(info.isolation_mode, sizeof(info.isolation_mode), "%s", "process");
    info.created_at = (long)now;
    info.updated_at = (long)now;
    info.exists = 1;
    info.namespace_valid = 1;
    info.containment_ready = 1;
    snprintf(info.workspace_namespace, sizeof(info.workspace_namespace), "ws/%s", ws_id);
    snprintf(info.boundary_reason, sizeof(info.boundary_reason), "%s", "none");
    yai_workspace_security_recompute_effective(&info);

    if (snprintf(manifest_path, sizeof(manifest_path), "%s/manifest.json", ws_dir) <= 0)
        return -1;
    if (yai_workspace_write_manifest_path(manifest_path, &info) != 0)
        return -1;
    if (yai_workspace_write_containment_surfaces(&info) != 0)
        return -1;

    if (info_out)
    {
        *info_out = info;
        yai_workspace_fill_shell_relation(info_out);
    }

    return 0;
}

int yai_session_set_active_workspace(const char *ws_id, char *err, size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    const char *why = "ok";

    if (err && err_cap > 0)
        err[0] = '\0';

    if (!ws_id || !ws_id[0] || !yai_ws_id_is_valid(ws_id))
    {
        why = "invalid_workspace_id";
        goto fail;
    }

    if (yai_session_read_workspace_info(ws_id, &info) != 0 || !info.exists)
    {
        why = "workspace_not_found";
        goto fail;
    }

    if (yai_workspace_binding_write(ws_id, info.workspace_alias) != 0)
    {
        why = "binding_write_failed";
        goto fail;
    }
    snprintf(info.session_binding, sizeof(info.session_binding), "%s", ws_id);
    info.activated_at = (long)time(NULL);
    info.updated_at = info.activated_at;
    if (yai_workspace_write_manifest_ws_id(ws_id, &info) != 0)
    {
        why = "manifest_write_failed";
        goto fail;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        why = "containment_write_failed";
        goto fail;
    }

    /* Best-effort shell integration bootstrap.
     * Keep activation successful even if user shell files are read-only/missing. */
    (void)yai_session_ensure_shell_integration();

    return 0;

fail:
    if (err && err_cap > 0)
        snprintf(err, err_cap, "%s", why);
    return -1;
}

int yai_session_clear_active_workspace(void)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char path[MAX_PATH_LEN];

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) == 0 &&
        strcmp(status, "active") == 0)
    {
        info.session_binding[0] = '\0';
        info.updated_at = (long)time(NULL);
        (void)yai_workspace_write_manifest_ws_id(info.ws_id, &info);
        (void)yai_workspace_write_containment_surfaces(&info);
    }

    if (yai_workspace_binding_path(path, sizeof(path)) != 0)
        return -1;
    if (unlink(path) != 0 && errno != ENOENT)
        return -1;
    return 0;
}

int yai_session_clear_workspace_runtime_state(char *out_ws_id, size_t out_ws_id_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0 ||
        strcmp(status, "active") != 0)
        return -1;

    info.inferred_family[0] = '\0';
    info.inferred_specialization[0] = '\0';
    info.inferred_confidence = 0.0;
    info.effective_stack_ref[0] = '\0';
    info.effective_overlays_ref[0] = '\0';
    info.last_effect_summary[0] = '\0';
    info.last_authority_summary[0] = '\0';
    info.last_evidence_summary[0] = '\0';
    info.last_resolution_summary[0] = '\0';
    info.last_resolution_trace_ref[0] = '\0';
    info.runtime_attached = 0;
    info.control_plane_attached = 0;
    info.updated_at = (long)time(NULL);

    if (yai_workspace_write_manifest_ws_id(info.ws_id, &info) != 0)
        return -1;
    if (yai_workspace_write_containment_surfaces(&info) != 0)
        return -1;

    if (out_ws_id && out_ws_id_cap > 0)
        snprintf(out_ws_id, out_ws_id_cap, "%s", info.ws_id);
    return 0;
}

int yai_session_resolve_current_workspace(yai_workspace_runtime_info_t *info_out,
                                          char *status_out,
                                          size_t status_cap,
                                          char *err,
                                          size_t err_cap)
{
    char ws_id[MAX_WS_ID_LEN] = {0};
    char ws_alias[64] = {0};
    const char *env_ws = getenv("YAI_ACTIVE_WORKSPACE");

    if (!info_out || !status_out || status_cap == 0)
        return -1;
    memset(info_out, 0, sizeof(*info_out));
    status_out[0] = '\0';
    if (err && err_cap > 0)
        err[0] = '\0';

    if (env_ws && env_ws[0])
    {
        if (!yai_ws_id_is_valid(env_ws))
        {
            snprintf(status_out, status_cap, "%s", "invalid");
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "env_binding_invalid");
            return -1;
        }
        snprintf(ws_id, sizeof(ws_id), "%s", env_ws);
        snprintf(ws_alias, sizeof(ws_alias), "%s", env_ws);
    }
    else
    {
        if (yai_workspace_binding_read(ws_id, sizeof(ws_id), ws_alias, sizeof(ws_alias)) != 0)
        {
            snprintf(status_out, status_cap, "%s", "no_active");
            return 0;
        }
        if (!yai_ws_id_is_valid(ws_id))
        {
            snprintf(status_out, status_cap, "%s", "invalid");
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "binding_workspace_id_invalid");
            return -1;
        }
    }

    if (yai_session_read_workspace_info(ws_id, info_out) != 0 || !info_out->exists)
    {
        snprintf(status_out, status_cap, "%s", "stale");
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "binding_workspace_missing");
        return -1;
    }

    if (info_out->workspace_alias[0] == '\0')
        snprintf(info_out->workspace_alias, sizeof(info_out->workspace_alias), "%s", ws_alias);
    if (!info_out->namespace_valid)
    {
        snprintf(status_out, status_cap, "%s", "invalid");
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", info_out->boundary_reason[0] ? info_out->boundary_reason : "workspace_namespace_invalid");
        return -1;
    }
    snprintf(status_out, status_cap, "%s", "active");
    return 0;
}

int yai_session_enforce_workspace_scope(const char *target_ws_id,
                                        char *err,
                                        size_t err_cap)
{
    yai_workspace_runtime_info_t current;
    char status[24];
    char bind_err[96];

    if (err && err_cap > 0)
        err[0] = '\0';
    if (!target_ws_id || !target_ws_id[0] || !yai_ws_id_is_valid(target_ws_id))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "target_workspace_invalid");
        return -1;
    }

    if (yai_session_resolve_current_workspace(&current, status, sizeof(status), bind_err, sizeof(bind_err)) != 0)
    {
        if (strcmp(status, "no_active") == 0)
            return 0;
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_binding_invalid");
        return -1;
    }

    if (strcmp(current.ws_id, target_ws_id) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "cross_workspace_scope_denied");
        return -1;
    }
    return 0;
}

int yai_session_build_prompt_context_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    int rc;
    int n;

    if (!out || out_cap == 0)
        return -1;

    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (rc != 0)
    {
        if (strcmp(status, "no_active") == 0)
        {
            n = snprintf(out,
                         out_cap,
                         "{\"type\":\"yai.workspace.prompt_context.v1\",\"binding_status\":\"no_active\",\"binding_scope\":\"session\"}");
        }
        else
        {
            n = snprintf(out,
                         out_cap,
                         "{\"type\":\"yai.workspace.prompt_context.v1\",\"binding_status\":\"%s\",\"binding_scope\":\"session\",\"reason\":\"%s\"}",
                         status[0] ? status : "invalid",
                         err[0] ? err : "binding_error");
        }
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.prompt_context.v1\","
                 "\"binding_status\":\"active\","
                 "\"binding_scope\":\"session\","
                 "\"workspace_id\":\"%s\","
                 "\"workspace_alias\":\"%s\","
                 "\"state\":\"%s\","
                 "\"workspace_root\":\"%s\","
                 "\"root_anchor_mode\":\"%s\","
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\"},"
                 "\"execution\":{\"mode_requested\":\"%s\",\"mode_effective\":\"%s\",\"degraded\":%s}"
                 "}",
                 info.ws_id,
                 info.workspace_alias,
                 info.state,
                 info.root_path,
                 info.root_anchor_mode[0] ? info.root_anchor_mode : "managed_default_root",
                 info.declared_control_family,
                 info.declared_specialization,
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false");
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

static int yai_workspace_binding_validity(const char *binding_status)
{
    if (!binding_status)
        return 0;
    if (strcmp(binding_status, "active") == 0)
        return 1;
    if (strcmp(binding_status, "no_active") == 0)
        return 1;
    return 0;
}

static int yai_workspace_has_declared_context(const yai_workspace_runtime_info_t *info)
{
    if (!info)
        return 0;
    return (info->declared_control_family[0] || info->declared_specialization[0]) ? 1 : 0;
}

static int yai_workspace_has_effective_context(const yai_workspace_runtime_info_t *info)
{
    if (!info)
        return 0;
    return (info->effective_stack_ref[0] || info->last_effect_summary[0]) ? 1 : 0;
}

static int yai_embedded_law_path(char *out, size_t out_cap, const char *rel)
{
    const char *root = getenv("YAI_LAW_EMBED_ROOT");
    const char *base = NULL;
    const char *candidates[] = {"embedded/law", "../yai/embedded/law", "../../yai/embedded/law"};
    int i;
    FILE *probe = NULL;
    if (!out || out_cap == 0)
        return -1;
    if (root && root[0])
    {
        base = root;
    }
    else
    {
        for (i = 0; i < (int)(sizeof(candidates) / sizeof(candidates[0])); i++)
        {
            char p[MAX_PATH_LEN];
            if (snprintf(p, sizeof(p), "%s/classification/classification-map.json", candidates[i]) <= 0)
                continue;
            probe = fopen(p, "rb");
            if (probe)
            {
                fclose(probe);
                base = candidates[i];
                break;
            }
        }
    }
    if (!base)
        base = "embedded/law";
    if (snprintf(out, out_cap, "%s/%s", base, rel ? rel : "") <= 0)
        return -1;
    return 0;
}

static int yai_read_text(const char *path, char *out, size_t out_cap)
{
    FILE *f;
    size_t n;
    if (!path || !out || out_cap < 2)
        return -1;
    f = fopen(path, "rb");
    if (!f)
        return -1;
    n = fread(out, 1, out_cap - 1, f);
    fclose(f);
    out[n] = '\0';
    return 0;
}

static int yai_embedded_family_exists(const char *family)
{
    char path[MAX_PATH_LEN];
    char json[YAI_WS_JSON_IO_CAP];
    char needle[160];
    if (!family || !family[0])
        return 0;
    if (yai_embedded_law_path(path, sizeof(path), "control-families/index/families.index.json") != 0)
        return 0;
    if (yai_read_text(path, json, sizeof(json)) != 0)
        return 0;
    if (snprintf(needle, sizeof(needle), "\"canonical_name\": \"%s\"", family) <= 0)
        return 0;
    return strstr(json, needle) != NULL;
}

static int yai_embedded_resolve_specialization_family(const char *specialization, char *family_out, size_t family_cap)
{
    char path[MAX_PATH_LEN];
    char json[YAI_WS_JSON_IO_CAP];
    char needle[192];
    char *p;
    char *fkey;
    char *colon;
    char *quote;
    if (!specialization || !specialization[0] || !family_out || family_cap == 0)
        return -1;
    family_out[0] = '\0';
    if (yai_embedded_law_path(path, sizeof(path), "domain-specializations/index/specializations.index.json") != 0)
        return -1;
    if (yai_read_text(path, json, sizeof(json)) != 0)
        return -1;
    if (snprintf(needle, sizeof(needle), "\"specialization_id\": \"%s\"", specialization) <= 0)
        return -1;
    p = strstr(json, needle);
    if (!p)
        return -1;
    fkey = strstr(p, "\"family\":");
    if (!fkey)
        return -1;
    colon = strchr(fkey, ':');
    if (!colon)
        return -1;
    quote = strchr(colon, '"');
    if (!quote)
        return -1;
    quote++;
    {
        char *end = strchr(quote, '"');
        size_t n;
        if (!end)
            return -1;
        n = (size_t)(end - quote);
        if (n >= family_cap)
            n = family_cap - 1;
        memcpy(family_out, quote, n);
        family_out[n] = '\0';
    }
    return family_out[0] ? 0 : -1;
}

static int yai_embedded_specialization_matches_family(const char *family, const char *specialization)
{
    char inferred_family[96];
    if (!specialization || !specialization[0])
        return 1;
    if (yai_embedded_resolve_specialization_family(specialization, inferred_family, sizeof(inferred_family)) != 0)
        return 0;
    if (!family || !family[0])
        return 1;
    return strcmp(family, inferred_family) == 0;
}

int yai_session_build_workspace_status_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.status.v1\","
                 "\"active\":%s,"
                 "\"binding_status\":\"%s\","
                 "\"binding_valid\":%s,"
                 "\"namespace_scope\":\"workspace\","
                 "\"namespace_valid\":%s,"
                 "\"boundary_reason\":\"%s\","
                 "\"containment_layout\":\"%s\","
                 "\"containment_ready\":%s,"
                 "\"security_level_declared\":\"%s\","
                 "\"security_level_effective\":\"%s\","
                 "\"security_enforcement_mode\":\"%s\","
                 "\"security_backend_mode\":\"%s\","
                 "\"execution_mode_requested\":\"%s\","
                 "\"execution_mode_effective\":\"%s\","
                 "\"execution_mode_degraded\":%s,"
                 "\"execution_degraded_reason\":\"%s\","
                 "\"execution_unsupported_scopes\":\"%s\","
                 "\"runtime_attached\":%s,"
                 "\"isolation_mode\":\"%s\","
                 "\"debug_mode\":%s,"
                 "\"declared_context_present\":%s,"
                 "\"effective_context_present\":%s,"
                 "\"workspace_root\":\"%s\","
                 "\"workspace_store_root\":\"%s\","
                 "\"root_anchor_mode\":\"%s\","
                 "\"shell_path_relation\":\"%s\","
                 "\"reason\":\"%s\""
                 "}",
                 (rc == 0 && strcmp(status, "active") == 0) ? "true" : "false",
                 status[0] ? status : "invalid",
                 yai_workspace_binding_validity(status) ? "true" : "false",
                 (rc == 0 && info.namespace_valid) ? "true" : "false",
                 (rc == 0 && info.boundary_reason[0]) ? info.boundary_reason : "none",
                 (rc == 0 && info.containment_layout[0]) ? info.containment_layout : "v1",
                 (rc == 0 && info.containment_ready) ? "true" : "false",
                 (rc == 0 && info.security_level_declared[0]) ? info.security_level_declared : "scoped",
                 (rc == 0 && info.security_level_effective[0]) ? info.security_level_effective : "logical",
                 (rc == 0 && info.security_enforcement_mode[0]) ? info.security_enforcement_mode : "runtime_scoped",
                 (rc == 0 && info.security_backend_mode[0]) ? info.security_backend_mode : "none",
                 (rc == 0 && info.execution_mode_requested[0]) ? info.execution_mode_requested : "scoped",
                 (rc == 0 && info.execution_mode_effective[0]) ? info.execution_mode_effective : "scoped",
                 (rc == 0 && info.execution_mode_degraded) ? "true" : "false",
                 (rc == 0 && info.execution_degraded_reason[0]) ? info.execution_degraded_reason : "none",
                 (rc == 0 && info.execution_unsupported_scopes[0]) ? info.execution_unsupported_scopes : "none",
                 (rc == 0 && info.runtime_attached) ? "true" : "false",
                 (rc == 0 && info.isolation_mode[0]) ? info.isolation_mode : "process",
                 (rc == 0 && info.debug_mode) ? "true" : "false",
                 (rc == 0 && yai_workspace_has_declared_context(&info)) ? "true" : "false",
                 (rc == 0 && yai_workspace_has_effective_context(&info)) ? "true" : "false",
                 (rc == 0) ? info.root_path : "",
                 (rc == 0) ? info.workspace_store_root : "",
                 (rc == 0) ? info.root_anchor_mode : "",
                 (rc == 0) ? info.shell_path_relation : "unknown",
                 err[0] ? err : "none");
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_inspect_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char sci_experiment[192];
    char sci_parameter[192];
    char sci_repro[192];
    char sci_dataset[192];
    char sci_publication[192];
    char dig_outbound[192];
    char dig_sink[192];
    char dig_publication[192];
    char dig_retrieval[192];
    char dig_distribution[192];
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char op_summary[192];
    const char *review_state;
    int evt_external;
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (rc != 0 || strcmp(status, "active") != 0)
    {
        (void)snprintf(evt_declared, sizeof(evt_declared), "%s", "unset");
        (void)snprintf(evt_business, sizeof(evt_business), "%s", "not_resolved");
        (void)snprintf(evt_enforcement, sizeof(evt_enforcement), "%s", "not_resolved");
        (void)snprintf(evt_stage, sizeof(evt_stage), "%s", "unknown");
        (void)snprintf(evt_id, sizeof(evt_id), "%s", "none");
        evt_external = 0;
        (void)snprintf(op_summary, sizeof(op_summary), "%s", "unknown/not_resolved => not_resolved");
        review_state = "unresolved";
        (void)snprintf(sci_experiment, sizeof(sci_experiment), "%s", "not available");
        (void)snprintf(sci_parameter, sizeof(sci_parameter), "%s", "not available");
        (void)snprintf(sci_repro, sizeof(sci_repro), "%s", "not available");
        (void)snprintf(sci_dataset, sizeof(sci_dataset), "%s", "not available");
        (void)snprintf(sci_publication, sizeof(sci_publication), "%s", "not available");
        (void)snprintf(dig_outbound, sizeof(dig_outbound), "%s", "not available");
        (void)snprintf(dig_sink, sizeof(dig_sink), "%s", "not available");
        (void)snprintf(dig_publication, sizeof(dig_publication), "%s", "not available");
        (void)snprintf(dig_retrieval, sizeof(dig_retrieval), "%s", "not available");
        (void)snprintf(dig_distribution, sizeof(dig_distribution), "%s", "not available");
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.inspect.v1\","
                     "\"binding_status\":\"%s\","
                     "\"identity\":{\"workspace_id\":\"\",\"workspace_alias\":\"\",\"root_path\":\"\",\"state\":\"\"},"
                     "\"root_model\":{\"workspace_store_root\":\"\",\"runtime_state_root\":\"\",\"metadata_root\":\"\",\"root_anchor_mode\":\"\"},"
                     "\"containment\":{\"layout\":\"v1\",\"ready\":false,\"state_surface\":\"\",\"traces_index\":\"\",\"artifacts_index\":\"\",\"runtime_surface\":\"\",\"binding_surface\":\"\"},"
                     "\"security\":{\"level_declared\":\"scoped\",\"level_effective\":\"logical\",\"enforcement_mode\":\"runtime_scoped\",\"backend_mode\":\"none\",\"scopes\":{\"process\":false,\"filesystem\":true,\"socket\":false,\"network\":false,\"resource\":false,\"privilege\":false,\"runtime_route\":true,\"binding\":true},\"capabilities\":{\"sandbox_ready\":true,\"hardened_fs\":true,\"process_isolation\":false,\"network_policy\":false}},"
                     "\"execution\":{\"mode_requested\":\"scoped\",\"mode_effective\":\"logical\",\"degraded\":true,\"degraded_reason\":\"binding_unavailable\",\"unsupported_scopes\":\"process,socket,network,resource,privilege\",\"advisory_scopes\":\"process,socket,network,resource,privilege\",\"process_intent\":\"shared_runtime\",\"channel_mode\":\"global_control_scoped_route\",\"artifact_policy_mode\":\"workspace_owned\",\"network_intent\":\"advisory_none\",\"resource_intent\":\"advisory_none\",\"privilege_intent\":\"inherited_host\",\"attach_descriptor_ref\":\"\",\"execution_profile_ref\":\"\"},"
                     "\"boundary\":{\"namespace\":\"\",\"namespace_scope\":\"workspace\",\"namespace_valid\":false,\"state\":\"invalid\",\"reason\":\"binding_unavailable\"},"
                     "\"shell\":{\"cwd\":\"\",\"cwd_relation\":\"workspace_root_unset\"},"
                     "\"session\":{\"session_binding\":\"%s\",\"runtime_attached\":false,\"control_plane_attached\":false,\"isolation_mode\":\"process\",\"debug_mode\":false},"
                     "\"normative\":{\"declared\":{\"family\":\"\",\"specialization\":\"\",\"source\":\"unset\"},"
                     "\"inferred\":{\"family\":\"\",\"specialization\":\"\",\"confidence\":0.000},"
                     "\"effective\":{\"stack_ref\":\"\",\"overlays_ref\":\"\",\"effect_summary\":\"\",\"authority_summary\":\"\",\"evidence_summary\":\"\"}},"
                     "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                     "\"operational_state\":{\"binding_state\":\"%s\",\"attached_governance_objects\":\"\",\"active_effective_stack\":\"\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"not_resolved\",\"last_authority\":\"not_available\",\"last_evidence\":\"not_available\",\"last_trace_ref\":\"\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                     "\"governance\":{\"policy_attachments\":\"\",\"policy_attachment_count\":0},"
                     "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                     "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"},"
                     "\"inspect\":{\"last_resolution_summary\":\"\",\"last_resolution_trace_ref\":\"\"},"
                     "\"reason\":\"%s\""
                     "}",
                     status[0] ? status : "invalid",
                     status[0] ? status : "invalid",
                     evt_id,
                     evt_stage,
                     evt_declared,
                     evt_business,
                     evt_enforcement,
                     evt_external ? "true" : "false",
                     status[0] ? status : "invalid",
                     evt_id,
                     evt_stage,
                     evt_business,
                     evt_enforcement,
                     review_state,
                     op_summary,
                     sci_experiment,
                     sci_parameter,
                     sci_repro,
                     sci_dataset,
                     sci_publication,
                     dig_outbound,
                     dig_sink,
                     dig_publication,
                     dig_retrieval,
                     dig_distribution,
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    yai_workspace_build_scientific_summaries(&info,
                                             sci_experiment,
                                             sizeof(sci_experiment),
                                             sci_parameter,
                                             sizeof(sci_parameter),
                                             sci_repro,
                                             sizeof(sci_repro),
                                             sci_dataset,
                                             sizeof(sci_dataset),
                                             sci_publication,
                                             sizeof(sci_publication));
    yai_workspace_build_digital_summaries(&info,
                                          dig_outbound,
                                          sizeof(dig_outbound),
                                          dig_sink,
                                          sizeof(dig_sink),
                                          dig_publication,
                                          sizeof(dig_publication),
                                          dig_retrieval,
                                          sizeof(dig_retrieval),
                                          dig_distribution,
                                          sizeof(dig_distribution));
    yai_session_workspace_event_semantics(&info,
                                          evt_declared,
                                          sizeof(evt_declared),
                                          evt_business,
                                          sizeof(evt_business),
                                          evt_enforcement,
                                          sizeof(evt_enforcement),
                                          evt_stage,
                                          sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info.last_resolution_trace_ref[0] ? "evt-" : "none",
             info.last_resolution_trace_ref[0] ? info.last_resolution_trace_ref : "");
    review_state = yai_workspace_review_state_from_effect(info.last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info.last_effect_summary, op_summary, sizeof(op_summary));

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.inspect.v1\","
                 "\"binding_status\":\"active\","
                 "\"identity\":{\"workspace_id\":\"%s\",\"workspace_alias\":\"%s\",\"root_path\":\"%s\",\"state\":\"%s\"},"
                 "\"root_model\":{\"workspace_store_root\":\"%s\",\"runtime_state_root\":\"%s\",\"metadata_root\":\"%s\",\"root_anchor_mode\":\"%s\"},"
                 "\"containment\":{\"layout\":\"%s\",\"ready\":%s,\"state_surface\":\"%s\",\"traces_index\":\"%s\",\"artifacts_index\":\"%s\",\"runtime_surface\":\"%s\",\"binding_surface\":\"%s\"},"
                 "\"security\":{\"level_declared\":\"%s\",\"level_effective\":\"%s\",\"enforcement_mode\":\"%s\",\"backend_mode\":\"%s\",\"scopes\":{\"process\":%s,\"filesystem\":%s,\"socket\":%s,\"network\":%s,\"resource\":%s,\"privilege\":%s,\"runtime_route\":%s,\"binding\":%s},\"capabilities\":{\"sandbox_ready\":%s,\"hardened_fs\":%s,\"process_isolation\":%s,\"network_policy\":%s}},"
                 "\"execution\":{\"mode_requested\":\"%s\",\"mode_effective\":\"%s\",\"degraded\":%s,\"degraded_reason\":\"%s\",\"unsupported_scopes\":\"%s\",\"advisory_scopes\":\"%s\",\"process_intent\":\"%s\",\"channel_mode\":\"%s\",\"artifact_policy_mode\":\"%s\",\"network_intent\":\"%s\",\"resource_intent\":\"%s\",\"privilege_intent\":\"%s\",\"attach_descriptor_ref\":\"%s\",\"execution_profile_ref\":\"%s\"},"
                 "\"boundary\":{\"namespace\":\"%s\",\"namespace_scope\":\"workspace\",\"namespace_valid\":%s,\"state\":\"%s\",\"reason\":\"%s\"},"
                 "\"shell\":{\"cwd\":\"%s\",\"cwd_relation\":\"%s\"},"
                 "\"session\":{\"session_binding\":\"%s\",\"runtime_attached\":%s,\"control_plane_attached\":%s,\"isolation_mode\":\"%s\",\"debug_mode\":%s},"
                 "\"normative\":{\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\",\"source\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"effective\":{\"stack_ref\":\"%s\",\"overlays_ref\":\"%s\",\"effect_summary\":\"%s\",\"authority_summary\":\"%s\",\"evidence_summary\":\"%s\"}},"
                 "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                 "\"operational_state\":{\"binding_state\":\"active\",\"attached_governance_objects\":\"%s\",\"active_effective_stack\":\"%s\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"%s\",\"last_authority\":\"%s\",\"last_evidence\":\"%s\",\"last_trace_ref\":\"%s\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                 "\"governance\":{\"policy_attachments\":\"%s\",\"policy_attachment_count\":%d},"
                 "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                 "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"},"
                 "\"inspect\":{\"last_resolution_summary\":\"%s\",\"last_resolution_trace_ref\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.workspace_alias,
                 info.root_path,
                 info.state,
                 info.workspace_store_root,
                 info.runtime_state_root,
                 info.metadata_root,
                 info.root_anchor_mode[0] ? info.root_anchor_mode : "managed_default_root",
                 info.containment_layout[0] ? info.containment_layout : "v1",
                 info.containment_ready ? "true" : "false",
                 info.state_surface_path,
                 info.traces_index_path,
                 info.artifacts_index_path,
                 info.runtime_surface_path,
                 info.binding_state_path,
                 info.security_level_declared[0] ? info.security_level_declared : "scoped",
                 info.security_level_effective[0] ? info.security_level_effective : "logical",
                 info.security_enforcement_mode[0] ? info.security_enforcement_mode : "runtime_scoped",
                 info.security_backend_mode[0] ? info.security_backend_mode : "none",
                 info.scope_process ? "true" : "false",
                 info.scope_filesystem ? "true" : "false",
                 info.scope_socket ? "true" : "false",
                 info.scope_network ? "true" : "false",
                 info.scope_resource ? "true" : "false",
                 info.scope_privilege ? "true" : "false",
                 info.scope_runtime_route ? "true" : "false",
                 info.scope_binding ? "true" : "false",
                 info.capability_sandbox_ready ? "true" : "false",
                 info.capability_hardened_fs ? "true" : "false",
                 info.capability_process_isolation ? "true" : "false",
                 info.capability_network_policy ? "true" : "false",
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false",
                 info.execution_degraded_reason[0] ? info.execution_degraded_reason : "none",
                 info.execution_unsupported_scopes[0] ? info.execution_unsupported_scopes : "none",
                 info.execution_advisory_scopes[0] ? info.execution_advisory_scopes : "none",
                 info.process_intent[0] ? info.process_intent : "shared_runtime",
                 info.channel_mode[0] ? info.channel_mode : "global_control_scoped_route",
                 info.artifact_policy_mode[0] ? info.artifact_policy_mode : "workspace_owned",
                 info.network_intent[0] ? info.network_intent : "advisory_none",
                 info.resource_intent[0] ? info.resource_intent : "advisory_none",
                 info.privilege_intent[0] ? info.privilege_intent : "inherited_host",
                 info.attach_descriptor_ref,
                 info.execution_profile_ref,
                 info.workspace_namespace,
                 info.namespace_valid ? "true" : "false",
                 info.namespace_valid ? "enforced" : "invalid",
                 info.boundary_reason[0] ? info.boundary_reason : "none",
                 info.shell_cwd,
                 info.shell_path_relation[0] ? info.shell_path_relation : "unknown",
                 info.session_binding,
                 info.runtime_attached ? "true" : "false",
                 info.control_plane_attached ? "true" : "false",
                 info.isolation_mode[0] ? info.isolation_mode : "process",
                 info.debug_mode ? "true" : "false",
                 info.declared_control_family,
                 info.declared_specialization,
                 info.declared_context_source[0] ? info.declared_context_source : "unset",
                 info.inferred_family,
                 info.inferred_specialization,
                 info.inferred_confidence,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 evt_id,
                 evt_stage,
                 evt_declared,
                 evt_business,
                 evt_enforcement,
                 evt_external ? "true" : "false",
                 info.policy_attachments_csv,
                 info.effective_stack_ref,
                 evt_id,
                 evt_stage,
                 evt_business,
                 evt_enforcement,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 info.last_resolution_trace_ref,
                 review_state,
                 op_summary,
                 info.policy_attachments_csv,
                 info.policy_attachment_count,
                 sci_experiment,
                 sci_parameter,
                 sci_repro,
                 sci_dataset,
                 sci_publication,
                 dig_outbound,
                 dig_sink,
                 dig_publication,
                 dig_retrieval,
                 dig_distribution,
                 info.last_resolution_summary,
                 info.last_resolution_trace_ref);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_domain_get_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    int rc;
    int n;
    if (!out || out_cap == 0)
        return -1;
    rc = yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err));
    if (rc != 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{\"type\":\"yai.workspace.domain.get.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.domain.get.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\",\"source\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"effective\":{\"family\":\"%s\",\"specialization\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.declared_control_family,
                 info.declared_specialization,
                 info.declared_context_source[0] ? info.declared_context_source : "unset",
                 info.inferred_family,
                 info.inferred_specialization,
                 info.inferred_confidence,
                 info.inferred_family[0] ? info.inferred_family : info.declared_control_family,
                 info.inferred_specialization[0] ? info.inferred_specialization : info.declared_specialization);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_set_workspace_declared_context(const char *family,
                                               const char *specialization,
                                               char *out_json,
                                               size_t out_cap,
                                               char *err,
                                               size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char bind_err[96];
    char resolved_family[96] = {0};
    int n;
    if (err && err_cap > 0)
        err[0] = '\0';

    if ((!family || !family[0]) && (!specialization || !specialization[0]))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_or_specialization_required");
        return -1;
    }

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }

    if (family && family[0])
        snprintf(resolved_family, sizeof(resolved_family), "%s", family);
    else
        snprintf(resolved_family, sizeof(resolved_family), "%s", info.declared_control_family);

    /* Deterministic error precedence: explicit family validity is checked before specialization matching. */
    if (family && family[0] && !yai_embedded_family_exists(resolved_family))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_not_found");
        return -1;
    }

    if (specialization && specialization[0])
    {
        char inferred_family[96] = {0};
        if (yai_embedded_resolve_specialization_family(specialization, inferred_family, sizeof(inferred_family)) != 0)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "specialization_not_found");
            return -1;
        }
        if (!resolved_family[0])
            snprintf(resolved_family, sizeof(resolved_family), "%s", inferred_family);
        if (strcmp(resolved_family, inferred_family) != 0)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "specialization_family_mismatch");
            return -1;
        }
    }

    if (resolved_family[0] && !yai_embedded_family_exists(resolved_family))
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "family_not_found");
        return -1;
    }
    if (!yai_embedded_specialization_matches_family(resolved_family, specialization && specialization[0] ? specialization : info.declared_specialization))
    {
        if (!specialization || !specialization[0])
            info.declared_specialization[0] = '\0';
        else
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", "specialization_family_mismatch");
            return -1;
        }
    }

    if (resolved_family[0])
        snprintf(info.declared_control_family, sizeof(info.declared_control_family), "%s", resolved_family);
    if (specialization && specialization[0])
        snprintf(info.declared_specialization, sizeof(info.declared_specialization), "%s", specialization);
    snprintf(info.declared_context_source, sizeof(info.declared_context_source), "%s", "declared");
    info.updated_at = (long)time(NULL);
    info.activated_at = info.activated_at ? info.activated_at : info.updated_at;
    info.exists = 1;
    snprintf(info.session_binding, sizeof(info.session_binding), "%s", info.ws_id);

    if (yai_workspace_write_manifest_ws_id(info.ws_id, &info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "manifest_write_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "containment_write_failed");
        return -1;
    }

    if (out_json && out_cap > 0)
    {
        n = snprintf(out_json,
                     out_cap,
                     "{\"type\":\"yai.workspace.domain.set.v1\",\"workspace_id\":\"%s\",\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\",\"source\":\"%s\"}}",
                     info.ws_id,
                     info.declared_control_family,
                     info.declared_specialization,
                     info.declared_context_source);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    return 0;
}

int yai_session_workspace_policy_attachment_update(const char *object_id,
                                                   int attach_mode,
                                                   char *out_json,
                                                   size_t out_cap,
                                                   char *err,
                                                   size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    yai_governable_object_meta_t meta;
    char status[24];
    char bind_err[96];
    char eligibility[64];
    char compatibility[64];
    char conflicts[160];
    char warnings[160];
    const char *attachment_state = "attached_inactive";
    const char *action = "attach";
    int blocking = 0;
    int already_attached = 0;
    int n;
    int op_rc;

    if (err && err_cap > 0) err[0] = '\0';
    if (!object_id || !object_id[0])
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "object_id_required");
        return -1;
    }
    if (!yai_policy_attachment_id_valid(object_id))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "invalid_object_id");
        return -1;
    }
    yai_governable_meta_defaults(&meta);
    if (!yai_embedded_governable_object_lookup(object_id, &meta))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "governable_object_not_found");
        return -1;
    }
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }

    already_attached = yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id);
    if (attach_mode == 1)
    {
        action = "attach";
        if (yai_workspace_policy_evaluate(&info,
                                          &meta,
                                          eligibility,
                                          sizeof(eligibility),
                                          compatibility,
                                          sizeof(compatibility),
                                          conflicts,
                                          sizeof(conflicts),
                                          warnings,
                                          sizeof(warnings),
                                          &blocking) != 0)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "policy_evaluation_failed");
            return -1;
        }
        if (blocking)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", conflicts[0] ? conflicts : "conflict_blocking");
            return -1;
        }
        if (!yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id) &&
            yai_policy_attachment_csv_count(info.policy_attachments_csv) >= YAI_POLICY_ATTACHMENTS_MAX)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_limit_reached");
            return -1;
        }
        op_rc = yai_policy_attachment_csv_add(info.policy_attachments_csv,
                                              sizeof(info.policy_attachments_csv),
                                              object_id);
        if (op_rc != 0)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_add_failed");
            return -1;
        }
    }
    else if (attach_mode == 2)
    {
        action = "activate";
        if (!yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id))
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_not_found");
            return -1;
        }
        snprintf(eligibility, sizeof(eligibility), "%s", "eligible");
        snprintf(compatibility, sizeof(compatibility), "%s", "dry_run_passed");
        snprintf(conflicts, sizeof(conflicts), "%s", "none");
        warnings[0] = '\0';
    }
    else
    {
        action = "detach";
        op_rc = yai_policy_attachment_csv_remove(info.policy_attachments_csv,
                                                 sizeof(info.policy_attachments_csv),
                                                 object_id);
        if (op_rc == 1)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_not_found");
            return -1;
        }
        if (op_rc != 0)
        {
            if (err && err_cap > 0) snprintf(err, err_cap, "%s", "attachment_remove_failed");
            return -1;
        }
    }

    info.policy_attachment_count = yai_policy_attachment_csv_count(info.policy_attachments_csv);
    attachment_state = yai_workspace_attachment_state_for_meta(&meta,
                                                               yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id));
    info.updated_at = (long)time(NULL);
    if (yai_workspace_write_manifest_ws_id(info.ws_id, &info) != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "manifest_write_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "containment_write_failed");
        return -1;
    }

    if (out_json && out_cap > 0)
    {
        if (attach_mode == 0)
        {
            snprintf(eligibility, sizeof(eligibility), "%s", "eligible");
            snprintf(compatibility, sizeof(compatibility), "%s", "dry_run_passed");
            snprintf(conflicts, sizeof(conflicts), "%s", "none");
            warnings[0] = '\0';
        }
        n = snprintf(out_json,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.policy.attachment.v1\","
                     "\"workspace_id\":\"%s\","
                     "\"binding_status\":\"active\","
                     "\"action\":\"%s\","
                     "\"object_id\":\"%s\","
                     "\"kind\":\"%s\","
                     "\"status\":\"%s\","
                     "\"review_state\":\"%s\","
                     "\"eligibility_result\":\"%s\","
                     "\"compatibility_result\":\"%s\","
                     "\"conflict_summary\":\"%s\","
                     "\"warnings\":\"%s\","
                     "\"attachment_state\":\"%s\","
                     "\"activation_state\":\"%s\","
                     "\"already_attached\":%s,"
                     "\"attachment_valid\":true,"
                     "\"policy_attachments\":\"%s\","
                     "\"policy_attachment_count\":%d"
                     "}",
                     info.ws_id,
                     action,
                     object_id,
                     meta.kind,
                     meta.status,
                     meta.review_state,
                     eligibility,
                     compatibility,
                     conflicts,
                     warnings,
                     attachment_state,
                     yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id) ? "active" : "inactive",
                     already_attached ? "true" : "false",
                     info.policy_attachments_csv,
                     info.policy_attachment_count);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    return 0;
}

int yai_session_workspace_policy_apply_dry_run(const char *object_id,
                                               char *out_json,
                                               size_t out_cap,
                                               char *err,
                                               size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    yai_governable_object_meta_t meta;
    char status[24];
    char bind_err[96];
    char eligibility[64];
    char compatibility[64];
    char conflicts[160];
    char warnings[160];
    char effective_family[96];
    char effective_specialization[96];
    int blocking = 0;
    int n;

    if (err && err_cap > 0) err[0] = '\0';
    if (!object_id || !object_id[0])
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "object_id_required");
        return -1;
    }
    if (!yai_policy_attachment_id_valid(object_id))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "invalid_object_id");
        return -1;
    }
    yai_governable_meta_defaults(&meta);
    if (!yai_embedded_governable_object_lookup(object_id, &meta))
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "governable_object_not_found");
        return -1;
    }
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }

    if (yai_workspace_policy_evaluate(&info,
                                      &meta,
                                      eligibility,
                                      sizeof(eligibility),
                                      compatibility,
                                      sizeof(compatibility),
                                      conflicts,
                                      sizeof(conflicts),
                                      warnings,
                                      sizeof(warnings),
                                      &blocking) != 0)
    {
        if (err && err_cap > 0) snprintf(err, err_cap, "%s", "policy_evaluation_failed");
        return -1;
    }
    yai_workspace_policy_effective_context(&info,
                                           effective_family,
                                           sizeof(effective_family),
                                           effective_specialization,
                                           sizeof(effective_specialization));

    if (out_json && out_cap > 0)
    {
        n = snprintf(out_json,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.policy.apply.dry_run.v1\","
                     "\"workspace_id\":\"%s\","
                     "\"object_id\":\"%s\","
                     "\"kind\":\"%s\","
                     "\"status\":\"%s\","
                     "\"review_state\":\"%s\","
                     "\"workspace_family\":\"%s\","
                     "\"workspace_specialization\":\"%s\","
                     "\"target_workspace_ids\":\"%s\","
                     "\"target_family\":\"%s\","
                     "\"target_specialization\":\"%s\","
                     "\"precedence_class\":\"%s\","
                     "\"attachment_modes\":\"%s\","
                     "\"eligibility_result\":\"%s\","
                     "\"compatibility_result\":\"%s\","
                     "\"conflict_summary\":\"%s\","
                     "\"warnings\":\"%s\","
                     "\"ready_for_attach\":%s,"
                     "\"activation_state\":\"%s\","
                     "\"effective_stack_ref\":\"%s\""
                     "}",
                     info.ws_id,
                     object_id,
                     meta.kind,
                     meta.status,
                     meta.review_state,
                     effective_family,
                     effective_specialization,
                     meta.workspace_targets_csv,
                     meta.family_targets_csv,
                     meta.specialization_targets_csv,
                     meta.precedence_class,
                     meta.attachment_modes_csv,
                     eligibility,
                     compatibility,
                     conflicts,
                     warnings,
                     blocking ? "false" : "true",
                     yai_policy_attachment_csv_contains(info.policy_attachments_csv, object_id) ? "active" : "inactive",
                     info.effective_stack_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    return 0;
}

int yai_session_build_workspace_policy_effective_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char sci_experiment[192];
    char sci_parameter[192];
    char sci_repro[192];
    char sci_dataset[192];
    char sci_publication[192];
    char dig_outbound[192];
    char dig_sink[192];
    char dig_publication[192];
    char dig_retrieval[192];
    char dig_distribution[192];
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char op_summary[192];
    const char *review_state;
    int evt_external;
    int n;
    if (!out || out_cap == 0)
        return -1;
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0)
    {
        n = snprintf(out, out_cap,
                     "{\"type\":\"yai.workspace.policy.effective.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    yai_workspace_build_scientific_summaries(&info,
                                             sci_experiment,
                                             sizeof(sci_experiment),
                                             sci_parameter,
                                             sizeof(sci_parameter),
                                             sci_repro,
                                             sizeof(sci_repro),
                                             sci_dataset,
                                             sizeof(sci_dataset),
                                             sci_publication,
                                             sizeof(sci_publication));
    yai_workspace_build_digital_summaries(&info,
                                          dig_outbound,
                                          sizeof(dig_outbound),
                                          dig_sink,
                                          sizeof(dig_sink),
                                          dig_publication,
                                          sizeof(dig_publication),
                                          dig_retrieval,
                                          sizeof(dig_retrieval),
                                          dig_distribution,
                                          sizeof(dig_distribution));
    yai_session_workspace_event_semantics(&info,
                                          evt_declared,
                                          sizeof(evt_declared),
                                          evt_business,
                                          sizeof(evt_business),
                                          evt_enforcement,
                                          sizeof(evt_enforcement),
                                          evt_stage,
                                          sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info.last_resolution_trace_ref[0] ? "evt-" : "none",
             info.last_resolution_trace_ref[0] ? info.last_resolution_trace_ref : "");
    review_state = yai_workspace_review_state_from_effect(info.last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info.last_effect_summary, op_summary, sizeof(op_summary));
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.policy.effective.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"workspace_namespace\":\"%s\","
                 "\"namespace_valid\":%s,"
                 "\"containment_ready\":%s,"
                 "\"security_level_effective\":\"%s\","
                 "\"execution_mode_requested\":\"%s\","
                 "\"execution_mode_effective\":\"%s\","
                 "\"execution_mode_degraded\":%s,"
                 "\"execution_degraded_reason\":\"%s\","
                 "\"execution_unsupported_scopes\":\"%s\","
                 "\"execution_profile_ref\":\"%s\","
                 "\"traces_index_ref\":\"%s\","
                 "\"artifacts_index_ref\":\"%s\","
                 "\"family_effective\":\"%s\","
                 "\"specialization_effective\":\"%s\","
                 "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                 "\"operational_state\":{\"binding_state\":\"active\",\"attached_governance_objects\":\"%s\",\"active_effective_stack\":\"%s\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"%s\",\"last_authority\":\"%s\",\"last_evidence\":\"%s\",\"last_trace_ref\":\"%s\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                 "\"effective_stack_ref\":\"%s\","
                 "\"effective_overlays_ref\":\"%s\","
                 "\"policy_attachments\":\"%s\","
                 "\"policy_attachment_count\":%d,"
                 "\"precedence\":\"specialization+overlays\","
                 "\"effect_summary\":\"%s\","
                 "\"authority_summary\":\"%s\","
                 "\"evidence_summary\":\"%s\","
                 "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                 "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.workspace_namespace,
                 info.namespace_valid ? "true" : "false",
                 info.containment_ready ? "true" : "false",
                 info.security_level_effective[0] ? info.security_level_effective : "logical",
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false",
                 info.execution_degraded_reason[0] ? info.execution_degraded_reason : "none",
                 info.execution_unsupported_scopes[0] ? info.execution_unsupported_scopes : "none",
                 info.execution_profile_ref,
                 info.traces_index_path,
                 info.artifacts_index_path,
                 info.inferred_family[0] ? info.inferred_family : info.declared_control_family,
                 info.inferred_specialization[0] ? info.inferred_specialization : info.declared_specialization,
                 evt_id,
                 evt_stage,
                 evt_declared,
                 evt_business,
                 evt_enforcement,
                 evt_external ? "true" : "false",
                 info.policy_attachments_csv,
                 info.effective_stack_ref,
                 evt_id,
                 evt_stage,
                 evt_business,
                 evt_enforcement,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 info.last_resolution_trace_ref,
                 review_state,
                 op_summary,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 info.policy_attachments_csv,
                 info.policy_attachment_count,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 sci_experiment,
                 sci_parameter,
                 sci_repro,
                 sci_dataset,
                 sci_publication,
                 dig_outbound,
                 dig_sink,
                 dig_publication,
                 dig_retrieval,
                 dig_distribution);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_debug_resolution_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char sci_experiment[192];
    char sci_parameter[192];
    char sci_repro[192];
    char sci_dataset[192];
    char sci_publication[192];
    char dig_outbound[192];
    char dig_sink[192];
    char dig_publication[192];
    char dig_retrieval[192];
    char dig_distribution[192];
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char op_summary[192];
    const char *review_state;
    int evt_external;
    int n;
    const char *context_source;
    if (!out || out_cap == 0)
        return -1;
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0)
    {
        n = snprintf(out, out_cap,
                     "{\"type\":\"yai.workspace.debug.resolution.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    context_source = info.declared_context_source[0] ? info.declared_context_source : "unset";
    yai_workspace_build_scientific_summaries(&info,
                                             sci_experiment,
                                             sizeof(sci_experiment),
                                             sci_parameter,
                                             sizeof(sci_parameter),
                                             sci_repro,
                                             sizeof(sci_repro),
                                             sci_dataset,
                                             sizeof(sci_dataset),
                                             sci_publication,
                                             sizeof(sci_publication));
    yai_workspace_build_digital_summaries(&info,
                                          dig_outbound,
                                          sizeof(dig_outbound),
                                          dig_sink,
                                          sizeof(dig_sink),
                                          dig_publication,
                                          sizeof(dig_publication),
                                          dig_retrieval,
                                          sizeof(dig_retrieval),
                                          dig_distribution,
                                          sizeof(dig_distribution));
    yai_session_workspace_event_semantics(&info,
                                          evt_declared,
                                          sizeof(evt_declared),
                                          evt_business,
                                          sizeof(evt_business),
                                          evt_enforcement,
                                          sizeof(evt_enforcement),
                                          evt_stage,
                                          sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info.last_resolution_trace_ref[0] ? "evt-" : "none",
             info.last_resolution_trace_ref[0] ? info.last_resolution_trace_ref : "");
    review_state = yai_workspace_review_state_from_effect(info.last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info.last_effect_summary, op_summary, sizeof(op_summary));
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.debug.resolution.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"workspace_namespace\":\"%s\","
                 "\"namespace_valid\":%s,"
                 "\"containment_ready\":%s,"
                 "\"security_level_effective\":\"%s\","
                 "\"execution_mode_requested\":\"%s\","
                 "\"execution_mode_effective\":\"%s\","
                 "\"execution_mode_degraded\":%s,"
                 "\"execution_degraded_reason\":\"%s\","
                 "\"execution_unsupported_scopes\":\"%s\","
                 "\"attach_descriptor_ref\":\"%s\","
                 "\"state_surface_ref\":\"%s\","
                 "\"context_source\":\"%s\","
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                 "\"operational_state\":{\"binding_state\":\"active\",\"attached_governance_objects\":\"%s\",\"active_effective_stack\":\"%s\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"%s\",\"last_authority\":\"%s\",\"last_evidence\":\"%s\",\"last_trace_ref\":\"%s\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                 "\"effective\":{\"stack_ref\":\"%s\",\"overlays_ref\":\"%s\"},"
                 "\"precedence_outcome\":\"%s\","
                 "\"effect_outcome\":\"%s\","
                 "\"last_resolution_trace_ref\":\"%s\","
                 "\"last_resolution_summary\":\"%s\","
                 "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                 "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.workspace_namespace,
                 info.namespace_valid ? "true" : "false",
                 info.containment_ready ? "true" : "false",
                 info.security_level_effective[0] ? info.security_level_effective : "logical",
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false",
                 info.execution_degraded_reason[0] ? info.execution_degraded_reason : "none",
                 info.execution_unsupported_scopes[0] ? info.execution_unsupported_scopes : "none",
                 info.attach_descriptor_ref,
                 info.state_surface_path,
                 context_source,
                 info.declared_control_family,
                 info.declared_specialization,
                 info.inferred_family,
                 info.inferred_specialization,
                 info.inferred_confidence,
                 evt_id,
                 evt_stage,
                 evt_declared,
                 evt_business,
                 evt_enforcement,
                 evt_external ? "true" : "false",
                 info.policy_attachments_csv,
                 info.effective_stack_ref,
                 evt_id,
                 evt_stage,
                 evt_business,
                 evt_enforcement,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 info.last_resolution_trace_ref,
                 review_state,
                 op_summary,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 "specialization+overlays",
                 info.last_effect_summary,
                 info.last_resolution_trace_ref,
                 info.last_resolution_summary,
                 sci_experiment,
                 sci_parameter,
                 sci_repro,
                 sci_dataset,
                 sci_publication,
                 dig_outbound,
                 dig_sink,
                 dig_publication,
                 dig_retrieval,
                 dig_distribution);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_record_resolution_snapshot(const char *ws_id,
                                          const yai_law_resolution_output_t *law_out,
                                          char *err,
                                          size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    const yai_law_effective_stack_t *stack;
    int i;
    char overlays[192];
    size_t used = 0;
    int n;

    if (err && err_cap > 0)
        err[0] = '\0';
    if (!ws_id || !law_out)
        return -1;
    if (yai_session_read_workspace_info(ws_id, &info) != 0 || !info.exists)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_manifest_missing");
        return -1;
    }
    if (yai_session_enforce_workspace_scope(ws_id, err, err_cap) != 0)
        return -1;

    stack = &law_out->decision.stack;
    snprintf(info.inferred_family, sizeof(info.inferred_family), "%s", law_out->decision.family_id);
    snprintf(info.inferred_specialization, sizeof(info.inferred_specialization), "%s", law_out->decision.specialization_id);
    info.inferred_confidence = 1.0;
    snprintf(info.effective_stack_ref, sizeof(info.effective_stack_ref), "%s", stack->stack_id);
    snprintf(info.last_effect_summary, sizeof(info.last_effect_summary), "%s", yai_law_effect_name(law_out->decision.final_effect));
    snprintf(info.last_authority_summary, sizeof(info.last_authority_summary), "%s", stack->authority_profile);
    snprintf(info.last_evidence_summary, sizeof(info.last_evidence_summary), "%s", stack->evidence_profile);
    snprintf(info.last_resolution_trace_ref, sizeof(info.last_resolution_trace_ref), "%s", law_out->evidence.trace_id);
    snprintf(info.last_resolution_summary,
             sizeof(info.last_resolution_summary),
             "%s/%s => %s",
             law_out->decision.family_id,
             law_out->decision.specialization_id,
             yai_law_effect_name(law_out->decision.final_effect));

    overlays[0] = '\0';
    for (i = 0; i < stack->overlay_count; i++)
    {
        n = snprintf(overlays + used, sizeof(overlays) - used, "%s%s", (i == 0) ? "" : ",", stack->overlay_layers[i]);
        if (n <= 0 || (size_t)n >= (sizeof(overlays) - used))
            break;
        used += (size_t)n;
    }
    snprintf(info.effective_overlays_ref, sizeof(info.effective_overlays_ref), "%s", overlays);
    info.updated_at = (long)time(NULL);
    info.last_attached_at = info.updated_at;
    info.runtime_attached = 1;
    info.control_plane_attached = 1;
    if (info.session_binding[0] == '\0')
        snprintf(info.session_binding, sizeof(info.session_binding), "%s", ws_id);
    if (info.declared_context_source[0] == '\0')
        snprintf(info.declared_context_source, sizeof(info.declared_context_source), "%s", "unset");
    if (info.isolation_mode[0] == '\0')
        snprintf(info.isolation_mode, sizeof(info.isolation_mode), "%s", "process");

    if (yai_workspace_write_manifest_ws_id(ws_id, &info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "manifest_write_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "containment_write_failed");
        return -1;
    }
    return 0;
}
