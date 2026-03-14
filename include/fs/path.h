#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_FS_PATH_MAX 512u

int yai_fs_path_join(const char *base,
                     const char *relative,
                     char *out,
                     size_t out_cap);
int yai_fs_path_normalize(const char *input,
                          char *out,
                          size_t out_cap);
int yai_fs_path_is_absolute(const char *path);

#ifdef __cplusplus
}
#endif
