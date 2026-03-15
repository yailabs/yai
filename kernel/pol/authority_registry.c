#include <yai/krt/authority.h>
#include <yai/ipc/errors.h>
#include <yai/ipc/roles.h>

#include <stdio.h>
#include <string.h>

static void reset_eval(yai_authority_evaluation_t *out)
{
  if (!out) return;
  memset(out, 0, sizeof(*out));
  out->decision = YAI_AUTHORITY_ALLOW;
}

int yai_authority_command_gate(const yai_vault_t *vault,
                               uint32_t command_id,
                               uint16_t role,
                               uint16_t arming,
                               yai_authority_evaluation_t *out,
                               char *err,
                               size_t err_cap)
{
  if (err && err_cap > 0) err[0] = '\0';
  if (!out) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "authority_eval_missing");
    return -1;
  }

  reset_eval(out);
  out->command_id = command_id;
  out->command_class = yai_resolve_command_class(command_id);
  out->operator_armed = (role == YAI_ROLE_OPERATOR && arming == 1) ? 1 : 0;
  out->authority_lock = (vault && vault->authority_lock) ? 1 : 0;

  if (out->command_class == 0x02u) {
    if (!out->operator_armed) {
      out->decision = YAI_AUTHORITY_DENY;
      snprintf(out->reason, sizeof(out->reason), "%s", "operator_arming_required");
      return 0;
    }
    if (out->authority_lock) {
      out->decision = YAI_AUTHORITY_DENY;
      snprintf(out->reason, sizeof(out->reason), "%s", "authority_lock_active");
      return 0;
    }
  }

  snprintf(out->reason, sizeof(out->reason), "%s", "authority_ok");
  return 0;
}
