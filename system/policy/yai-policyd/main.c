/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/policy/engine.h>
#include <yai/policy/grants.h>
#include <yai/policy/overlays.h>
#include <yai/policy/review.h>

#include <stdio.h>

int main(int argc, char **argv)
{
  char decision[128];
  char grants[192];
  char overlays[128];
  char review[128];
  const char *scope = (argc > 1) ? argv[1] : "system";

  if (yai_policy_engine_evaluate(scope, decision, sizeof(decision)) != 0) return 1;
  if (yai_policy_grants_view_json(scope, grants, sizeof(grants)) != 0) return 1;
  if (yai_policy_overlays_apply(scope, overlays, sizeof(overlays)) != 0) return 1;
  if (yai_policy_review_status(scope, review, sizeof(review)) != 0) return 1;

  puts("yai-policyd - policy L2 entrypoint");
  printf("decision=%s\n", decision);
  printf("grants=%s\n", grants);
  printf("overlays=%s\n", overlays);
  printf("review=%s\n", review);
  return 0;
}
