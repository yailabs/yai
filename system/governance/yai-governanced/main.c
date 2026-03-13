/* SPDX-License-Identifier: Apache-2.0 */
#include <yai/policy/governance/authority.h>
#include <yai/policy/governance/escalation.h>
#include <yai/policy/governance/decisions.h>
#include <yai/policy/governance/publication.h>

#include <stdio.h>

int main(int argc, char **argv)
{
  const char *scope = (argc > 1) ? argv[1] : "system";
  char authority[128], escalation[128], decisions[192], publication[128];

  if (yai_governance_authority_state(scope, authority, sizeof(authority)) != 0) return 1;
  if (yai_governance_escalation_route(scope, escalation, sizeof(escalation)) != 0) return 1;
  if (yai_governance_decisions_snapshot(scope, decisions, sizeof(decisions)) != 0) return 1;
  if (yai_governance_publication_status(scope, publication, sizeof(publication)) != 0) return 1;

  puts("yai-governanced - governance L2 entrypoint");
  printf("authority=%s\n", authority);
  printf("escalation=%s\n", escalation);
  printf("decisions=%s\n", decisions);
  printf("publication=%s\n", publication);
  return 0;
}
