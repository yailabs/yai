/* SPDX-License-Identifier: Apache-2.0 */

#include "../../../cognition/core/cognition/cognition_internal.h"

#include <stdio.h>

int yai_agent_historian_handle(const yai_agent_frame_t *frame,
                                    yai_cognition_response_t *response_out)
{
  yai_episodic_record_t episodic = {0};
  int rc;
  if (!frame || !response_out) return YAI_MIND_ERR_INVALID_ARG;

  rc = yai_domain_episodic_latest(&episodic);
  response_out->status = 200;
  if (rc == YAI_MIND_OK) {
    snprintf(response_out->summary, sizeof(response_out->summary), "Historian agent loaded latest episode.");
    snprintf(response_out->output, sizeof(response_out->output),
             "episode=%s node=%llu summary=%s",
             episodic.episode_id,
             (unsigned long long)episodic.node_id,
             episodic.summary);
    return YAI_MIND_OK;
  }

  snprintf(response_out->summary, sizeof(response_out->summary), "Historian agent found no episodic history.");
  snprintf(response_out->output, sizeof(response_out->output), "no_episodes recorded yet");
  return YAI_MIND_OK;
}
