/* SPDX-License-Identifier: Apache-2.0 */

#include "watch_internal.h"

#include <string.h>

void yai_watch_model_init(yai_watch_model_t *m)
{
  if (!m) return;
  memset(m, 0, sizeof(*m));
  m->view = YAI_WATCH_VIEW_HUMAN;
  m->last_latency_ms = -1;
  m->last_sev = YAI_WATCH_WARN;
}

void yai_watch_model_push(yai_watch_model_t *m, const yai_watch_entry_t *e)
{
  yai_watch_entry_t *slot;
  if (!m || !e) return;
  slot = &m->entries[m->head];
  *slot = *e;
  m->head = (m->head + 1U) % YAI_WATCH_HISTORY_CAP;
  if (m->count < YAI_WATCH_HISTORY_CAP) m->count++;
  if (e->sev == YAI_WATCH_OK) m->ok_count++;
  else if (e->sev == YAI_WATCH_WARN) m->warn_count++;
  else m->err_count++;
  m->last_latency_ms = e->latency_ms;
  m->last_sev = e->sev;
  m->tick_count++;
}

void yai_watch_model_clear(yai_watch_model_t *m)
{
  if (!m) return;
  memset(m->entries, 0, sizeof(m->entries));
  m->count = 0;
  m->head = 0;
  m->scroll = 0;
  m->ok_count = 0;
  m->warn_count = 0;
  m->err_count = 0;
  m->last_latency_ms = -1;
}

size_t yai_watch_model_visible_count(const yai_watch_model_t *m)
{
  if (!m) return 0U;
  return m->count;
}

const yai_watch_entry_t *yai_watch_model_at(const yai_watch_model_t *m, size_t index_from_oldest)
{
  size_t start;
  size_t idx;
  if (!m || index_from_oldest >= m->count) return NULL;
  start = (m->head + YAI_WATCH_HISTORY_CAP - m->count) % YAI_WATCH_HISTORY_CAP;
  idx = (start + index_from_oldest) % YAI_WATCH_HISTORY_CAP;
  return &m->entries[idx];
}
