/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/shell/app.h>

#include <stdio.h>

int main(int argc, char **argv)
{
    if (argc <= 1) {
        // porcelain gestisce help/usage; qui evitiamo exit 1 “muto”
        // e lasciamo che la UX sia unica.
        const char *fallback_argv[] = {"yai", "help", NULL};
        return yai_porcelain_run(2, (char**)fallback_argv);
    }

    return yai_porcelain_run(argc, argv);
}