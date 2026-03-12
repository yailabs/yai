// src/app/errors.c
#include "yai/shell/errors.h"

#include <stdio.h>

const char* yai_porcelain_err_name(yai_porcelain_err_t e) {
  switch (e) {
    case YAI_PORCELAIN_ERR_OK: return "OK";
    case YAI_PORCELAIN_ERR_USAGE: return "BAD ARGS";
    case YAI_PORCELAIN_ERR_RUNTIME_NOT_READY: return "RUNTIME NOT READY";
    case YAI_PORCELAIN_ERR_DEP_MISSING: return "INTERNAL ERROR";
    case YAI_PORCELAIN_ERR_GENERIC: return "INTERNAL ERROR";
    default: return "INTERNAL ERROR";
  }
}

int yai_porcelain_err_exit_code(yai_porcelain_err_t e) {
  switch (e) {
    case YAI_PORCELAIN_ERR_OK: return 0;
    case YAI_PORCELAIN_ERR_USAGE: return 20;
    case YAI_PORCELAIN_ERR_RUNTIME_NOT_READY: return 40;
    case YAI_PORCELAIN_ERR_DEP_MISSING: return 50;
    case YAI_PORCELAIN_ERR_GENERIC: return 50;
    default: return 50;
  }
}

void yai_porcelain_err_print(yai_porcelain_err_t e, const char* msg) {
  if (!msg) msg = "error";
  fprintf(stderr, "%s\n", yai_porcelain_err_name(e));
  fprintf(stderr, "%s\n", msg);
}
