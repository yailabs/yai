/* SPDX-License-Identifier: Apache-2.0 */
#include "yai/desktop/layout.h"

int yai_desktop_layout_stack_vertical(yai_desktop_layout_rect_t bounds,
                                      int count,
                                      yai_desktop_layout_rect_t *out_rects)
{
  int i;
  int h;
  if (!out_rects || count <= 0) return -1;
  h = bounds.h / count;
  for (i = 0; i < count; ++i) {
    out_rects[i].x = bounds.x;
    out_rects[i].y = bounds.y + (i * h);
    out_rects[i].w = bounds.w;
    out_rects[i].h = h;
  }
  return 0;
}
