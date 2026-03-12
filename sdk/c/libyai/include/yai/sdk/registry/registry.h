/* NOTE: internal compatibility/tooling surface; not public-stable SDK API. */
// include/yai/sdk/registry/law.h
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Entry for `yai law ...`
int yai_cmd_law(int argc, char** argv);

#ifdef __cplusplus
}
#endif