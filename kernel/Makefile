# YAI kernel build spine (vertical slice enabled).

CC ?= cc
CFLAGS ?= -std=c11 -Wall -Wextra -Werror -Iinclude -I.
LDFLAGS ?=
LDLIBS ?=

KERNEL_BUILD_DIR ?= build
KERNEL_OBJ_DIR ?= $(KERNEL_BUILD_DIR)/obj
KERNEL_BIN_DIR ?= $(KERNEL_BUILD_DIR)/bin
KERNEL_LIB_DIR ?= $(KERNEL_BUILD_DIR)/lib

KERNEL_CORE_SRCS := \
	core/kernel.c \
	lifecycle/lifecycle_state.c \
	lifecycle/readiness.c \
	lifecycle/transitions.c \
	state/kernel_state.c \
	model/registry/kernel_registry.c \
	model/registry/container_registry.c \
	model/registry/session_registry.c \
	model/registry/capability_registry.c \
	model/registry/containment_registry.c \
	model/registry/trace_registry.c \
	model/registry/grants_registry.c \
	policy/capabilities.c \
	policy/policy_hooks.c \
	grants/grant_state.c \
	grants/grant_checks.c \
	grants/grant_lifecycle.c \
	session/session_admission.c \
	session/session_state.c \
	session/session_registry.c \
	security/containment.c \
	container/namespaces.c \
	container/rootfs.c \
	container/mounts.c \
	container/limits.c \
	../boot/handoff/handoff.c \
	../boot/initrd/preboot.c \
	../boot/bootloader/bootstrap.c

KERNEL_CORE_OBJS := $(patsubst %.c,$(KERNEL_OBJ_DIR)/%.o,$(KERNEL_CORE_SRCS))
KERNEL_LIB := $(KERNEL_LIB_DIR)/libyai_kernel_core.a
KERNEL_SMOKE_SRC := tests/kernel_vertical_smoke.c
KERNEL_SMOKE_OBJ := $(KERNEL_OBJ_DIR)/tests/kernel_vertical_smoke.o
KERNEL_SMOKE_BIN := $(KERNEL_BIN_DIR)/kernel-vertical-smoke

KERNEL_DIRS := $(KERNEL_BUILD_DIR) $(KERNEL_OBJ_DIR) $(KERNEL_BIN_DIR) $(KERNEL_LIB_DIR)

.PHONY: all dirs
all: $(KERNEL_LIB)
	@echo "YAI kernel core built: $(KERNEL_LIB)"

.PHONY: check
check:
	$(CC) $(CFLAGS) -fsyntax-only $(KERNEL_CORE_SRCS)

.PHONY: smoke
smoke: $(KERNEL_SMOKE_BIN)
	@$(KERNEL_SMOKE_BIN)

$(KERNEL_LIB): $(KERNEL_CORE_OBJS) | dirs
	ar rcs $@ $^

$(KERNEL_SMOKE_BIN): $(KERNEL_SMOKE_OBJ) $(KERNEL_LIB) | dirs
	$(CC) $(LDFLAGS) $(KERNEL_SMOKE_OBJ) -o $@ $(KERNEL_LIB) $(LDLIBS)

$(KERNEL_OBJ_DIR)/%.o: %.c | dirs
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

dirs:
	@mkdir -p $(KERNEL_DIRS)
