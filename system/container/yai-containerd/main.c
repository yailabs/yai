#define _POSIX_C_SOURCE 200809L

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <yai/api/version.h>
#include <yai/container/container.h>

static void print_help(void) {
  puts("yai-containerd - canonical container manager service surface");
  printf("version: %s\n", YAI_VERSION_STRING);
  puts("");
  puts("usage:");
  puts("  yai-containerd create <container-id> [interactive|managed|service|system|recovery]");
  puts("  yai-containerd initialize <container-id>");
  puts("  yai-containerd open <container-id>");
  puts("  yai-containerd attach <container-id> <session-id>");
  puts("  yai-containerd bind <container-id> <session-id> [normal|privileged|recovery|diagnostic] [cap-mask]");
  puts("  yai-containerd unbind <container-id> <session-id>");
  puts("  yai-containerd rebind <container-id> <old-session-id> <new-session-id> [mode] [cap-mask]");
  puts("  yai-containerd enter <container-id> <session-id>");
  puts("  yai-containerd leave <container-id> <session-id>");
  puts("  yai-containerd escape <container-id> <session-id> [none|admin|recovery|debug]");
  puts("  yai-containerd recovery-enter <container-id> <session-id>");
  puts("  yai-containerd mount <container-id> <source> <target> <ro|rw|privileged> [internal|attached|read-only|read-write|hidden|privileged-only]");
  puts("  yai-containerd resolve <container-id> <container-path>");
  puts("  yai-containerd visible <container-id> <container-path> [0|1]");
  puts("  yai-containerd recover <container-id>");
  puts("  yai-containerd config-show <container-id>");
  puts("  yai-containerd config-apply-defaults <container-id>");
  puts("  yai-containerd state-read <container-id>");
  puts("  yai-containerd state-snapshot <container-id> <snapshot-id>");
  puts("  yai-containerd recovery-check <container-id>");
  puts("  yai-containerd recovery-prepare <container-id> [none|soft-rebind|state-reload|root-reproject|service-rehydrate|full-domain] [reason-flags]");
  puts("  yai-containerd recovery-execute <container-id> [class]");
  puts("  yai-containerd recovery-complete <container-id> [open|active|degraded]");
  puts("  yai-containerd recovery-seal <container-id>");
  puts("  yai-containerd registry-lookup <container-id>");
  puts("  yai-containerd registry-list");
  puts("  yai-containerd service-register <container-id> <service-name> [service-handle]");
  puts("  yai-containerd service-ready <container-id> <service-name>");
  puts("  yai-containerd service-degraded <container-id> <service-name>");
  puts("  yai-containerd service-list <container-id>");
  puts("  yai-containerd runtime-view <container-id>");
  puts("  yai-containerd seal <container-id>");
  puts("  yai-containerd destroy <container-id>");
  puts("  yai-containerd show <container-id>");
}

static yai_container_class_t parse_class(const char *value) {
  if (!value || !value[0] || strcmp(value, "interactive") == 0) return YAI_CONTAINER_CLASS_INTERACTIVE;
  if (strcmp(value, "managed") == 0) return YAI_CONTAINER_CLASS_MANAGED;
  if (strcmp(value, "service") == 0) return YAI_CONTAINER_CLASS_SERVICE;
  if (strcmp(value, "system") == 0) return YAI_CONTAINER_CLASS_SYSTEM;
  if (strcmp(value, "recovery") == 0) return YAI_CONTAINER_CLASS_RECOVERY;
  return YAI_CONTAINER_CLASS_INTERACTIVE;
}

static yai_container_session_mode_t parse_session_mode(const char *value) {
  if (!value || strcmp(value, "normal") == 0) return YAI_CONTAINER_SESSION_MODE_NORMAL;
  if (strcmp(value, "privileged") == 0) return YAI_CONTAINER_SESSION_MODE_PRIVILEGED;
  if (strcmp(value, "recovery") == 0) return YAI_CONTAINER_SESSION_MODE_RECOVERY;
  if (strcmp(value, "diagnostic") == 0) return YAI_CONTAINER_SESSION_MODE_DIAGNOSTIC;
  return YAI_CONTAINER_SESSION_MODE_NORMAL;
}

static const char *session_mode_name(yai_container_session_mode_t mode) {
  switch (mode) {
    case YAI_CONTAINER_SESSION_MODE_GLOBAL: return "global";
    case YAI_CONTAINER_SESSION_MODE_NORMAL: return "normal";
    case YAI_CONTAINER_SESSION_MODE_PRIVILEGED: return "privileged";
    case YAI_CONTAINER_SESSION_MODE_RECOVERY: return "recovery";
    case YAI_CONTAINER_SESSION_MODE_DIAGNOSTIC: return "diagnostic";
    default: return "unknown";
  }
}

static yai_container_escape_policy_class_t parse_escape_class(const char *value) {
  if (!value || strcmp(value, "none") == 0) return YAI_CONTAINER_ESCAPE_NONE;
  if (strcmp(value, "admin") == 0) return YAI_CONTAINER_ESCAPE_CONTROLLED_ADMIN;
  if (strcmp(value, "recovery") == 0) return YAI_CONTAINER_ESCAPE_RECOVERY;
  if (strcmp(value, "debug") == 0) return YAI_CONTAINER_ESCAPE_DEBUG;
  return YAI_CONTAINER_ESCAPE_NONE;
}

static yai_container_mount_policy_t parse_mount_policy(const char *value) {
  if (!value || strcmp(value, "ro") == 0) return YAI_CONTAINER_MOUNT_RO;
  if (strcmp(value, "rw") == 0) return YAI_CONTAINER_MOUNT_RW;
  return YAI_CONTAINER_MOUNT_PRIVILEGED;
}

static yai_container_visibility_class_t parse_visibility(const char *value) {
  if (!value || strcmp(value, "attached") == 0) return YAI_CONTAINER_VISIBILITY_ATTACHED;
  if (strcmp(value, "internal") == 0) return YAI_CONTAINER_VISIBILITY_INTERNAL;
  if (strcmp(value, "read-only") == 0) return YAI_CONTAINER_VISIBILITY_READ_ONLY;
  if (strcmp(value, "read-write") == 0) return YAI_CONTAINER_VISIBILITY_READ_WRITE;
  if (strcmp(value, "hidden") == 0) return YAI_CONTAINER_VISIBILITY_HIDDEN;
  if (strcmp(value, "privileged-only") == 0) return YAI_CONTAINER_VISIBILITY_PRIVILEGED_ONLY;
  return YAI_CONTAINER_VISIBILITY_ATTACHED;
}

static yai_container_recovery_class_t parse_recovery_class(const char *value) {
  if (!value || strcmp(value, "state-reload") == 0) return YAI_CONTAINER_RECOVERY_CLASS_STATE_RELOAD;
  if (strcmp(value, "none") == 0) return YAI_CONTAINER_RECOVERY_CLASS_NONE;
  if (strcmp(value, "soft-rebind") == 0) return YAI_CONTAINER_RECOVERY_CLASS_SOFT_REBIND;
  if (strcmp(value, "root-reproject") == 0) return YAI_CONTAINER_RECOVERY_CLASS_ROOT_REPROJECT;
  if (strcmp(value, "service-rehydrate") == 0) return YAI_CONTAINER_RECOVERY_CLASS_SERVICE_REHYDRATE;
  if (strcmp(value, "full-domain") == 0) return YAI_CONTAINER_RECOVERY_CLASS_FULL_DOMAIN;
  return YAI_CONTAINER_RECOVERY_CLASS_STATE_RELOAD;
}

static yai_container_lifecycle_state_t parse_recovery_target(const char *value) {
  if (!value || strcmp(value, "active") == 0) return YAI_CONTAINER_LIFECYCLE_ACTIVE;
  if (strcmp(value, "open") == 0) return YAI_CONTAINER_LIFECYCLE_OPEN;
  if (strcmp(value, "degraded") == 0) return YAI_CONTAINER_LIFECYCLE_DEGRADED;
  return YAI_CONTAINER_LIFECYCLE_ACTIVE;
}

static int cmd_create(const char *container_id, const char *class_name) {
  yai_container_record_t record;
  time_t now = time(NULL);

  if (!container_id || !container_id[0]) return 1;

  memset(&record, 0, sizeof(record));
  (void)snprintf(record.identity.container_id, sizeof(record.identity.container_id), "%s", container_id);
  record.identity.container_class = parse_class(class_name);
  (void)snprintf(record.identity.container_profile,
                 sizeof(record.identity.container_profile),
                 "%s",
                 yai_container_class_name(record.identity.container_class));
  (void)snprintf(record.identity.creation_source, sizeof(record.identity.creation_source), "%s", "yai-containerd");
  record.identity.owner_handle = 1;
  record.identity.state_handle = (uint64_t)now;

  yai_container_config_defaults(&record.config);
  record.lifecycle.current = YAI_CONTAINER_LIFECYCLE_CREATED;
  record.lifecycle.previous = YAI_CONTAINER_LIFECYCLE_CREATED;
  record.lifecycle.created_at = (int64_t)now;
  record.lifecycle.updated_at = (int64_t)now;
  (void)snprintf(record.root.root_path, sizeof(record.root.root_path), "/container/%s", record.identity.container_id);
  record.root.container_root_handle = record.identity.state_handle;
  record.session_domain.container_session_scope = record.identity.state_handle;
  yai_container_state_defaults(&record.state);

  if (yai_container_create(&record) != 0) {
    fprintf(stderr, "create failed for container '%s'\n", container_id);
    return 1;
  }

  printf("created container %s class=%s\n",
         record.identity.container_id,
         yai_container_class_name(record.identity.container_class));
  return 0;
}

static int cmd_show(const char *container_id) {
  yai_container_identity_t identity;
  yai_container_state_t state;
  yai_container_root_t root;
  yai_container_session_domain_t session;
  yai_container_policy_view_t policy;
  yai_container_grants_view_t grants;

  if (yai_container_get_identity(container_id, &identity) != 0 ||
      yai_container_get_state(container_id, &state) != 0 ||
      yai_container_get_root_view(container_id, &root) != 0 ||
      yai_container_get_session_view(container_id, &session) != 0 ||
      yai_container_get_policy_view(container_id, &policy) != 0 ||
      yai_container_get_grants_view(container_id, &grants) != 0) {
    fprintf(stderr, "show failed for container '%s'\n", container_id);
    return 1;
  }

  printf("container_id=%s\n", identity.container_id);
  printf("class=%s profile=%s source=%s\n",
         yai_container_class_name(identity.container_class),
         identity.container_profile,
         identity.creation_source);
  printf("lifecycle=%s\n", yai_container_lifecycle_name(state.lifecycle_state));
  printf("state-updated-at=%lld recovery-flags=%llu\n",
         (long long)state.updated_at,
         (unsigned long long)state.recovery_reason_flags);
  printf("runtime-state=%u root-status=%u mount-status=%u session-status=%u service-status=%u recovery-status=%u health=%u\n",
         (unsigned)state.runtime_state,
         (unsigned)state.root_status,
         (unsigned)state.mount_status,
         (unsigned)state.session_status,
         (unsigned)state.service_status,
         (unsigned)state.recovery_status,
         (unsigned)state.health_state);
  printf("state-created-at=%lld degraded-flags=%llu last-error-class=%llu\n",
         (long long)state.created_at,
         (unsigned long long)state.degraded_flags,
         (unsigned long long)state.last_error_class);
  printf("root_handle=%llu projection=%s path=%s\n",
         (unsigned long long)root.container_root_handle,
         root.projection_ready ? "ready" : "not-ready",
         root.root_path);
  printf("projected-root=%s backing-store=%s\n",
         root.projected_root_host_path,
         root.backing_store_path);
  printf("session_bound=%u count=%llu last=%llu\n",
         (unsigned)session.bound,
         (unsigned long long)session.bound_session_count,
         (unsigned long long)session.last_bound_session_id);
  printf("active_session=%llu mode=%s cap-mask=%llu escape-class=%u iflags=%llu\n",
         (unsigned long long)session.active_session_id,
         session_mode_name(session.session_mode),
         (unsigned long long)session.capability_mask,
         (unsigned)session.escape_policy_class,
         (unsigned long long)session.interactive_flags);
  printf("policy_view=%llu grants_view=%llu\n",
         (unsigned long long)policy.policy_view_handle,
         (unsigned long long)grants.grants_view_handle);
  return 0;
}

static int cmd_registry_lookup(const char *container_id) {
  yai_container_registry_entry_t entry;
  if (yai_container_registry_lookup(container_id, &entry) != 0) {
    return 1;
  }
  printf("container_id=%s class=%s lifecycle=%s\n",
         entry.container_id,
         yai_container_class_name(entry.container_class),
         yai_container_lifecycle_name(entry.lifecycle_state));
  printf("root=%llu runtime-view=%llu policy-view=%llu grants-view=%llu service-surface=%llu\n",
         (unsigned long long)entry.root_handle,
         (unsigned long long)entry.runtime_view_handle,
         (unsigned long long)entry.policy_view_handle,
         (unsigned long long)entry.grants_view_handle,
         (unsigned long long)entry.service_surface_handle);
  printf("health=%u recovery=%u sessions=%llu daemons=%llu created=%lld updated=%lld\n",
         (unsigned)entry.health_state,
         (unsigned)entry.recovery_state,
         (unsigned long long)entry.bound_session_count,
         (unsigned long long)entry.attached_daemon_count,
         (long long)entry.created_at,
         (long long)entry.updated_at);
  return 0;
}

static int cmd_registry_list(void) {
  yai_container_registry_entry_t entries[YAI_CONTAINER_REGISTRY_LIST_MAX];
  size_t len = 0;
  size_t i;

  if (yai_container_registry_list(entries, YAI_CONTAINER_REGISTRY_LIST_MAX, &len) != 0) {
    return 1;
  }
  printf("registry-count=%zu\n", len);
  for (i = 0; i < len; ++i) {
    printf("%s class=%s lifecycle=%s health=%u recovery=%u sessions=%llu services=%llu\n",
           entries[i].container_id,
           yai_container_class_name(entries[i].container_class),
           yai_container_lifecycle_name(entries[i].lifecycle_state),
           (unsigned)entries[i].health_state,
           (unsigned)entries[i].recovery_state,
           (unsigned long long)entries[i].bound_session_count,
           (unsigned long long)entries[i].service_surface_handle);
  }
  return 0;
}

static int cmd_service_list(const char *container_id) {
  yai_container_service_entry_t entries[YAI_CONTAINER_SERVICE_SURFACE_MAX];
  size_t len = 0;
  size_t i;
  if (yai_container_service_list(container_id, entries, YAI_CONTAINER_SERVICE_SURFACE_MAX, &len) != 0) {
    return 1;
  }
  printf("services=%zu\n", len);
  for (i = 0; i < len; ++i) {
    printf("%s handle=%llu status=%u updated-at=%lld\n",
           entries[i].name,
           (unsigned long long)entries[i].service_handle,
           (unsigned)entries[i].status,
           (long long)entries[i].updated_at);
  }
  return 0;
}

static int cmd_runtime_view(const char *container_id) {
  yai_container_runtime_view_t view;
  if (yai_container_runtime_view_get(container_id, &view) != 0) {
    return 1;
  }
  printf("container=%s lifecycle=%s runtime=%u health=%u recovery=%u\n",
         view.identity.container_id,
         yai_container_lifecycle_name(view.state.lifecycle_state),
         (unsigned)view.state.runtime_state,
         (unsigned)view.health.health_state,
         (unsigned)view.recovery.recovery_status);
  printf("services-total=%zu ready=%zu degraded=%zu surface=%llu\n",
         view.services.total_services,
         view.services.ready_services,
         view.services.degraded_services,
         (unsigned long long)view.services.service_surface_handle);
  printf("policy-view=%llu grants-view=%llu bindings(daemon=%llu orchestration=%llu network=%llu)\n",
         (unsigned long long)view.policy_view.policy_view_handle,
         (unsigned long long)view.grants_view.grants_view_handle,
         (unsigned long long)view.bindings.daemon_binding_handle,
         (unsigned long long)view.bindings.orchestration_binding_handle,
         (unsigned long long)view.bindings.network_binding_handle);
  return 0;
}

int main(int argc, char **argv) {
  if (argc < 2 || strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "help") == 0) {
    print_help();
    return 0;
  }

  if (strcmp(argv[1], "create") == 0) return cmd_create(argc > 2 ? argv[2] : NULL, argc > 3 ? argv[3] : NULL);
  if (strcmp(argv[1], "initialize") == 0 && argc > 2) return yai_container_initialize(argv[2]) == 0 ? 0 : 1;
  if (strcmp(argv[1], "open") == 0 && argc > 2) return yai_container_open(argv[2]) == 0 ? 0 : 1;
  if (strcmp(argv[1], "attach") == 0 && argc > 3) return yai_container_attach(argv[2], (uint64_t)strtoull(argv[3], NULL, 10)) == 0 ? 0 : 1;
  if (strcmp(argv[1], "bind") == 0 && argc > 3) {
    uint64_t sid = (uint64_t)strtoull(argv[3], NULL, 10);
    yai_container_session_mode_t mode = parse_session_mode(argc > 4 ? argv[4] : NULL);
    uint64_t cap = argc > 5 ? (uint64_t)strtoull(argv[5], NULL, 10) : UINT64_MAX;
    return yai_container_bind_session(argv[2], sid, mode, cap, 0) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "unbind") == 0 && argc > 3) return yai_container_unbind_session(argv[2], (uint64_t)strtoull(argv[3], NULL, 10)) == 0 ? 0 : 1;
  if (strcmp(argv[1], "rebind") == 0 && argc > 4) {
    uint64_t old_sid = (uint64_t)strtoull(argv[3], NULL, 10);
    uint64_t new_sid = (uint64_t)strtoull(argv[4], NULL, 10);
    yai_container_session_mode_t mode = parse_session_mode(argc > 5 ? argv[5] : NULL);
    uint64_t cap = argc > 6 ? (uint64_t)strtoull(argv[6], NULL, 10) : UINT64_MAX;
    return yai_container_rebind_session(argv[2], old_sid, new_sid, mode, cap, 0) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "enter") == 0 && argc > 3) {
    yai_container_bound_session_t s;
    uint64_t sid = (uint64_t)strtoull(argv[3], NULL, 10);
    if (yai_container_session_enter(argv[2], sid, &s) != 0) return 1;
    printf("session=%llu container=%s mode=%s root=%llu runtime-view=%llu\n",
           (unsigned long long)s.session_id,
           s.bound_container_id,
           session_mode_name(s.session_mode),
           (unsigned long long)s.root_handle,
           (unsigned long long)s.runtime_view_handle);
    printf("projected-root=%s\n", s.path_context.projected_root);
    return 0;
  }
  if (strcmp(argv[1], "leave") == 0 && argc > 3) return yai_container_session_leave(argv[2], (uint64_t)strtoull(argv[3], NULL, 10)) == 0 ? 0 : 1;
  if (strcmp(argv[1], "escape") == 0 && argc > 3) {
    uint64_t sid = (uint64_t)strtoull(argv[3], NULL, 10);
    return yai_container_session_request_escape(argv[2], sid, parse_escape_class(argc > 4 ? argv[4] : NULL)) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "recovery-enter") == 0 && argc > 3) {
    return yai_container_session_enter_recovery(argv[2], (uint64_t)strtoull(argv[3], NULL, 10), 1u) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "mount") == 0 && argc > 5) {
    yai_container_mount_t mount;
    memset(&mount, 0, sizeof(mount));
    (void)snprintf(mount.source, sizeof(mount.source), "%s", argv[3]);
    (void)snprintf(mount.target, sizeof(mount.target), "%s", argv[4]);
    mount.policy = parse_mount_policy(argv[5]);
    mount.mount_class = YAI_CONTAINER_MOUNT_ATTACHED_EXTERNAL;
    mount.visibility_class = parse_visibility(argc > 6 ? argv[6] : NULL);
    mount.attachability_class = YAI_CONTAINER_ATTACHABILITY_CONTROLLED;
    return yai_container_attach_mount(argv[2], &mount) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "resolve") == 0 && argc > 3) {
    yai_container_path_context_t context;
    char resolved[2048];
    if (yai_container_path_context_load(argv[2], &context) != 0) return 1;
    if (yai_container_resolve_path(&context, argv[3], resolved, sizeof(resolved)) != 0) return 1;
    puts(resolved);
    return 0;
  }
  if (strcmp(argv[1], "visible") == 0 && argc > 3) {
    int privileged = (argc > 4 && strcmp(argv[4], "1") == 0) ? 1 : 0;
    int visible = yai_container_is_path_visible(argv[2], argv[3], privileged);
    if (visible < 0) return 1;
    puts(visible ? "visible" : "hidden");
    return visible ? 0 : 1;
  }
  if (strcmp(argv[1], "recover") == 0 && argc > 2) return yai_container_recover(argv[2], 1u) == 0 ? 0 : 1;
  if (strcmp(argv[1], "config-show") == 0 && argc > 2) {
    yai_container_config_t c;
    if (yai_container_config_load(argv[2], &c) != 0) return 1;
    printf("creation-policy=%u root-model=%u mount-policy=%u visibility-policy=%u\n",
           (unsigned)c.creation_policy,
           (unsigned)c.root.root_model,
           (unsigned)c.root.mount_policy,
           (unsigned)c.root.visibility_policy);
    printf("session-policy=%u allowed-session-mask=%llu interactive-flags=%llu\n",
           (unsigned)c.session.session_policy,
           (unsigned long long)c.session.allowed_session_modes_mask,
           (unsigned long long)c.session.interactive_flags);
    printf("services-profile=%u readiness-minimum=%llu required-service-mask=%llu\n",
           (unsigned)c.services.services_profile,
           (unsigned long long)c.services.readiness_minimum,
           (unsigned long long)c.services.required_service_mask);
    printf("recovery-policy=%u recoverable-mask=%llu daemon-attachment-policy=%u\n",
           (unsigned)c.recovery.recovery_policy,
           (unsigned long long)c.recovery.recoverable_mask,
           (unsigned)c.daemon_attachment_policy);
    printf("resource-profile=%u limits-class=%llu isolation-profile=%llu\n",
           (unsigned)c.resources.resource_profile,
           (unsigned long long)c.resources.limits_class,
           (unsigned long long)c.resources.isolation_profile);
    printf("network-profile=%llu policy-profile=%llu grants-profile=%llu runtime-profile=%llu\n",
           (unsigned long long)c.network_profile,
           (unsigned long long)c.policy_profile,
           (unsigned long long)c.grants_profile,
           (unsigned long long)c.runtime_profile);
    printf("config-revision=%llu applied-at=%lld\n",
           (unsigned long long)c.config_revision,
           (long long)c.applied_at);
    return 0;
  }
  if (strcmp(argv[1], "config-apply-defaults") == 0 && argc > 2) {
    yai_container_config_t c;
    yai_container_config_defaults(&c);
    return yai_container_config_apply(argv[2], &c, (int64_t)time(NULL)) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "state-read") == 0 && argc > 2) {
    yai_container_state_t s;
    if (yai_container_state_read(argv[2], &s) != 0) return 1;
    printf("lifecycle=%s runtime=%u root=%u mount=%u session=%u services=%u recovery=%u health=%u\n",
           yai_container_lifecycle_name(s.lifecycle_state),
           (unsigned)s.runtime_state,
           (unsigned)s.root_status,
           (unsigned)s.mount_status,
           (unsigned)s.session_status,
           (unsigned)s.service_status,
           (unsigned)s.recovery_status,
           (unsigned)s.health_state);
    printf("created-at=%lld updated-at=%lld recovery-flags=%llu degraded-flags=%llu\n",
           (long long)s.created_at,
           (long long)s.updated_at,
           (unsigned long long)s.recovery_reason_flags,
           (unsigned long long)s.degraded_flags);
    return 0;
  }
  if (strcmp(argv[1], "state-snapshot") == 0 && argc > 3) {
    yai_container_state_snapshot_t snap;
    if (yai_container_state_snapshot(argv[2], argv[3], &snap) != 0) return 1;
    printf("snapshot-id=%s captured-at=%lld lifecycle=%s runtime=%u health=%u\n",
           snap.snapshot_id,
           (long long)snap.captured_at,
           yai_container_lifecycle_name(snap.state.lifecycle_state),
           (unsigned)snap.state.runtime_state,
           (unsigned)snap.state.health_state);
    return 0;
  }
  if (strcmp(argv[1], "recovery-check") == 0 && argc > 2) {
    yai_container_recovery_status_t status = YAI_CONTAINER_RECOVERY_STATUS_NONE;
    yai_container_recovery_record_t rec;
    if (yai_container_recovery_check(argv[2], &status, &rec) != 0) return 1;
    printf("recovery-status=%u class=%u outcome=%u reason-flags=%llu marker-flags=%llu last-error-class=%llu updated-at=%lld\n",
           (unsigned)status,
           (unsigned)rec.recovery_class,
           (unsigned)rec.last_outcome,
           (unsigned long long)rec.reason_flags,
           (unsigned long long)rec.marker_flags,
           (unsigned long long)rec.last_error_class,
           (long long)rec.updated_at);
    return 0;
  }
  if (strcmp(argv[1], "recovery-prepare") == 0 && argc > 2) {
    yai_container_recovery_class_t klass = parse_recovery_class(argc > 3 ? argv[3] : NULL);
    uint64_t reason = argc > 4 ? (uint64_t)strtoull(argv[4], NULL, 10) : 1u;
    return yai_container_recovery_prepare(argv[2], klass, reason) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "recovery-execute") == 0 && argc > 2) {
    yai_container_recovery_class_t klass = parse_recovery_class(argc > 3 ? argv[3] : NULL);
    return yai_container_recovery_execute(argv[2], klass) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "recovery-complete") == 0 && argc > 2) {
    return yai_container_recovery_complete(argv[2], parse_recovery_target(argc > 3 ? argv[3] : NULL)) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "recovery-seal") == 0 && argc > 2) {
    return yai_container_recovery_seal(argv[2], (int64_t)time(NULL)) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "registry-lookup") == 0 && argc > 2) {
    return cmd_registry_lookup(argv[2]);
  }
  if (strcmp(argv[1], "registry-list") == 0) {
    return cmd_registry_list();
  }
  if (strcmp(argv[1], "service-register") == 0 && argc > 3) {
    uint64_t handle = argc > 4 ? (uint64_t)strtoull(argv[4], NULL, 10) : 0;
    return yai_container_service_register(argv[2], argv[3], handle) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "service-ready") == 0 && argc > 3) {
    return yai_container_service_mark_ready(argv[2], argv[3]) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "service-degraded") == 0 && argc > 3) {
    return yai_container_service_mark_degraded(argv[2], argv[3]) == 0 ? 0 : 1;
  }
  if (strcmp(argv[1], "service-list") == 0 && argc > 2) {
    return cmd_service_list(argv[2]);
  }
  if (strcmp(argv[1], "runtime-view") == 0 && argc > 2) {
    return cmd_runtime_view(argv[2]);
  }
  if (strcmp(argv[1], "seal") == 0 && argc > 2) return yai_container_seal_runtime(argv[2], (int64_t)time(NULL)) == 0 ? 0 : 1;
  if (strcmp(argv[1], "destroy") == 0 && argc > 2) return yai_container_destroy(argv[2], (int64_t)time(NULL)) == 0 ? 0 : 1;
  if (strcmp(argv[1], "show") == 0 && argc > 2) return cmd_show(argv[2]);

  print_help();
  return 1;
}
