// include/yai/shell/parse.h
#pragma once

#include <stddef.h>
#include "yai/shell/color.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_porcelain_kind_e {
  YAI_PORCELAIN_KIND_NONE = 0,
  YAI_PORCELAIN_KIND_HELP,
  YAI_PORCELAIN_KIND_LAW,     // "yai law ..."
  YAI_PORCELAIN_KIND_COMMAND, // "yai <group> <name> ..."
  YAI_PORCELAIN_KIND_WS_USE,  // "yai ws use <ws-id>"
  YAI_PORCELAIN_KIND_WS_CURRENT, // "yai ws current"
  YAI_PORCELAIN_KIND_WS_CLEAR, // "yai ws clear"
  YAI_PORCELAIN_KIND_WATCH,   // "yai watch <entrypoint> <topic> [op] ..."
  YAI_PORCELAIN_KIND_ERROR
} yai_porcelain_kind_t;

typedef struct yai_porcelain_request_s {
  yai_porcelain_kind_t kind;

  // For HELP:
  const char* help_token;
  const char* help_token2;
  const char* help_token3;

  // For COMMAND:
  const char* command_id;
  int verbose_contract;
  int verbose;
  int json_output;
  int quiet;
  int show_trace;
  int no_color;
  yai_color_mode_t color_mode;
  int pager;
  int no_pager;
  int interactive;
  int watch_interval_ms;
  int watch_count;
  int watch_no_clear;
  const char* ws_id;
  const char* role;
  int arming;
  int cmd_argc;
  char** cmd_argv;

  // For LAW:
  int law_argc;
  char** law_argv;

  // For ERROR:
  const char* error;
  const char* error_hint;
  int help_exit_code;
} yai_porcelain_request_t;

int yai_porcelain_parse_argv(int argc, char** argv, yai_porcelain_request_t* req);

#ifdef __cplusplus
}
#endif
