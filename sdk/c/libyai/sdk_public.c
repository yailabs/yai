/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/sdk.h>

#ifndef YAI_SDK_VERSION_STR
#define YAI_SDK_VERSION_STR "dev"
#endif

int yai_sdk_abi_version(void)
{
    return YAI_SDK_ABI_VERSION;
}

const char *yai_sdk_version(void)
{
    return YAI_SDK_VERSION_STR;
}

const char *yai_sdk_errstr(yai_sdk_err_t code)
{
    return yai_sdk_err_str(code);
}
