#include <yai/law/overlay_merge.h>
#include <stdio.h>
#include <string.h>

static int has_token(char arr[][64], int count, const char *id) {
  int i;
  if (!id || !id[0]) return 0;
  for (i = 0; i < count; ++i) {
    if (strcmp(arr[i], id) == 0) return 1;
  }
  return 0;
}

static void add_unique(char arr[][64], int *count, int cap, const char *id) {
  if (!arr || !count || !id || !id[0]) return;
  if (*count >= cap) return;
  if (has_token(arr, *count, id)) return;
  (void)snprintf(arr[*count], 64, "%s", id);
  (*count)++;
}

int yai_law_overlay_merge_apply(yai_law_effective_stack_t *stack) {
  int i;
  if (!stack) return -1;
  for (i = 0; i < stack->regulatory_overlay_count; ++i) {
    add_unique(stack->overlay_layers, &stack->overlay_count, YAI_LAW_COMPLIANCE_MAX, stack->regulatory_overlays[i]);
    add_unique(stack->compliance_layers, &stack->compliance_count, YAI_LAW_COMPLIANCE_MAX, stack->regulatory_overlays[i]);
  }
  for (i = 0; i < stack->sector_overlay_count; ++i) {
    add_unique(stack->overlay_layers, &stack->overlay_count, YAI_LAW_COMPLIANCE_MAX, stack->sector_overlays[i]);
    add_unique(stack->compliance_layers, &stack->compliance_count, YAI_LAW_COMPLIANCE_MAX, stack->sector_overlays[i]);
  }
  for (i = 0; i < stack->contextual_overlay_count; ++i) {
    add_unique(stack->overlay_layers, &stack->overlay_count, YAI_LAW_COMPLIANCE_MAX, stack->contextual_overlays[i]);
  }
  return 0;
}
