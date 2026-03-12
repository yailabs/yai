/* SPDX-License-Identifier: Apache-2.0 */

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv)
{
  char **exec_argv = argv;
  if (argc > 0 && argv && argv[0]) exec_argv[0] = "yai";
  execvp("yai", exec_argv);
  fprintf(stderr, "yai-ctl: failed to exec yai: %s\n", strerror(errno));
  return 127;
}
