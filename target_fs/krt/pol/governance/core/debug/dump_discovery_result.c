#include "../internal.h"

int yai_governance_dump_discovery_result(const yai_governance_discovery_result_t *disc, char *out, size_t out_cap) {
  const char *f1 = "";
  const char *f2 = "";
  const char *f3 = "";
  const char *s1 = "";
  const char *s2 = "";
  const char *s3 = "";
  if (!disc || !out || out_cap == 0) return -1;
  if (disc->family_candidate_count > 0) f1 = disc->family_candidates[0];
  if (disc->family_candidate_count > 1) f2 = disc->family_candidates[1];
  if (disc->family_candidate_count > 2) f3 = disc->family_candidates[2];
  if (disc->specialization_candidate_count > 0) s1 = disc->specialization_candidates[0];
  if (disc->specialization_candidate_count > 1) s2 = disc->specialization_candidates[1];
  if (disc->specialization_candidate_count > 2) s3 = disc->specialization_candidates[2];
  return yai_governance_safe_snprintf(out,
                               out_cap,
                               "family=%s specialization=%s compat_domain_id=%s confidence=%.3f ambiguous=%d family_candidates=[%s,%s,%s] specialization_candidates=[%s,%s,%s] rationale=%s",
                               disc->family_id,
                               disc->specialization_id,
                               disc->domain_id,
                               disc->confidence,
                               disc->ambiguous,
                               f1, f2, f3,
                               s1, s2, s3,
                               disc->rationale);
}
