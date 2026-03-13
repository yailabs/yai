/* SPDX-License-Identifier: Apache-2.0 */

#include "../internal/cognition/core/cognition/cognition_internal.h"

#include <stdio.h>
#include <string.h>

#define YAI_MIND_RAG_SESSION_CAP 16
#define YAI_MIND_RAG_ARENA_SIZE (32u * 1024u)

static yai_rag_session_state_t g_sessions[YAI_MIND_RAG_SESSION_CAP];

static int find_slot(const char *session_id)
{
  for (int i = 0; i < YAI_MIND_RAG_SESSION_CAP; i++) {
    if (g_sessions[i].initialized && strcmp(g_sessions[i].session_id, session_id) == 0) return i;
  }
  return -1;
}

static int alloc_slot(void)
{
  for (int i = 0; i < YAI_MIND_RAG_SESSION_CAP; i++) {
    if (!g_sessions[i].initialized) return i;
  }
  return -1;
}

int yai_rag_session_acquire(const yai_knowledge_session_t *session,
                                 yai_rag_session_state_t **session_out)
{
  const char *sid;
  int slot;

  if (!session || !session_out) return YAI_MIND_ERR_INVALID_ARG;
  sid = session->session_id[0] ? session->session_id : "default";

  slot = find_slot(sid);
  if (slot < 0) {
    slot = alloc_slot();
    if (slot < 0) return YAI_MIND_ERR_STATE;
    memset(&g_sessions[slot], 0, sizeof(g_sessions[slot]));
    snprintf(g_sessions[slot].session_id, sizeof(g_sessions[slot].session_id), "%s", sid);
    g_sessions[slot].slot = slot;
    if (yai_arena_init(&g_sessions[slot].arena, YAI_MIND_RAG_ARENA_SIZE) != YAI_MIND_OK) {
      return YAI_MIND_ERR_NO_MEMORY;
    }
    g_sessions[slot].initialized = 1;
  } else {
    yai_arena_reset(&g_sessions[slot].arena);
  }

  g_sessions[slot].cycle++;
  *session_out = &g_sessions[slot];
  return YAI_MIND_OK;
}

int yai_rag_session_release(yai_rag_session_state_t *session_state)
{
  (void)session_state;
  return YAI_MIND_OK;
}

void yai_rag_sessions_reset(void)
{
  for (int i = 0; i < YAI_MIND_RAG_SESSION_CAP; i++) {
    if (g_sessions[i].initialized) {
      yai_arena_destroy(&g_sessions[i].arena);
    }
    memset(&g_sessions[i], 0, sizeof(g_sessions[i]));
  }
}
