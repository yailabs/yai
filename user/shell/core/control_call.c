/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/commands/control_call.h"

#include <cJSON.h>

#include <string.h>

static const char *infer_target_plane(const char *command_id, int argc, char **argv)
{
  for (int i = 0; i + 1 < argc; i++) {
    if (argv && argv[i] && argv[i + 1] && strcmp(argv[i], "--target-plane") == 0) {
      return argv[i + 1];
    }
  }
  (void)command_id;
  return "runtime";
}

char *yai_build_control_call_json(const char *command_id, int argc, char **argv)
{
  if (!command_id || !command_id[0]) return NULL;

  cJSON *doc = cJSON_CreateObject();
  if (!doc) return NULL;

  cJSON_AddStringToObject(doc, "type", "yai.control.call.v1");
  cJSON_AddStringToObject(doc, "target_plane", infer_target_plane(command_id, argc, argv));
  cJSON_AddStringToObject(doc, "command_id", command_id);

  cJSON *arr = cJSON_AddArrayToObject(doc, "argv");
  for (int i = 0; i < argc; i++) {
    cJSON_AddItemToArray(arr, cJSON_CreateString((argv && argv[i]) ? argv[i] : ""));
  }

  char *json = cJSON_PrintUnformatted(doc);
  cJSON_Delete(doc);
  return json;
}
