/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_desktop_layout_rect {
  int x;
  int y;
  int w;
  int h;
} yai_desktop_layout_rect_t;

int yai_desktop_layout_stack_vertical(yai_desktop_layout_rect_t bounds,
                                      int count,
                                      yai_desktop_layout_rect_t *out_rects);

#ifdef __cplusplus
}
#endif
