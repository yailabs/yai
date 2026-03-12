// include/yai/shell/help.h
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// token meanings:
// - token1 NULL => global help
// - token1 "<group>" => group help
// - token1 "<group>" + token2 "<command>" => command help
// - token1 "yai.<group>.<command>" => command help by canonical id
int yai_porcelain_help_print(const char *token1, const char *token2, const char *token3, int pager, int no_pager);
int yai_porcelain_help_print_any(const char* token);

#ifdef __cplusplus
}
#endif
