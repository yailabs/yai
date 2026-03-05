# ===============================
# YAI Kernel Build System (L0+L1)
# ===============================

CC ?= cc

ROOT_DIR := $(abspath ..)

BUILD_ROOT ?= $(ROOT_DIR)/build
BIN_BUILD ?= $(BUILD_ROOT)/bin

COMPONENT_BUILD_DIR ?= $(BUILD_ROOT)/kernel
BUILD_DIR := $(COMPONENT_BUILD_DIR)
BIN_DIR := $(BIN_BUILD)

CFLAGS ?= -Wall -Wextra -O2 -std=c11 -MMD -MP
LDFLAGS ?=

SPECS_DIR := $(ROOT_DIR)/deps/yai-law
SPECS_INC_PROTOCOL := $(SPECS_DIR)/contracts/protocol/include
SPECS_INC_RUNTIME := $(SPECS_DIR)/contracts/protocol/runtime/include
SPECS_INC_VAULT := $(SPECS_DIR)/contracts/vault/include

INCLUDES := \
	-I./include \
	-I$(ROOT_DIR)/boot/include \
	-I$(ROOT_DIR)/root/include \
	-I$(SPECS_DIR) \
	-I$(SPECS_INC_PROTOCOL) \
	-I$(SPECS_INC_RUNTIME) \
	-I$(SPECS_INC_VAULT)

BOOT_SRC := \
	../boot/src/bootstrap.c \
	../boot/src/preboot.c

KERNEL_SRC := \
	src/enforcement/enforcement.c \
	src/core/fsm.c \
	src/core/ids.c \
	src/core/logger.c \
	src/core/project_tree.c \
	src/core/transport.c \
	src/core/yai_session.c \
	src/core/commands/yai_session_utils.c \
	src/core/commands/yai_session_reply.c \
	src/core/commands/yai_control_call.c \
	src/core/rpc_binary.c \
	src/core/rpc_codec.c \
	src/bin/workspace_kernel_main.c

# Single protocol runtime artifact (Fix #2)
PROTOCOL_RUNTIME_LIB := ../build/runtime-protocol/libruntime_protocol.a

# Root control transport reused by kernel
CORE_CONTROL_SRC := ../root/src/control_transport.c

OBJS_BOOT := $(patsubst ../%.c,$(BUILD_DIR)/%.o,$(BOOT_SRC))
OBJS_KERNEL := $(patsubst src/%.c,$(BUILD_DIR)/src/%.o,$(KERNEL_SRC))
CORE_CONTROL_OBJ := $(BUILD_DIR)/root/src/control_transport.o

TARGET_KERNEL := $(BIN_DIR)/yai-kernel

.PHONY: all build dirs abi clean

all: build

build: dirs abi $(TARGET_KERNEL)
	@echo "--- [YAI-KERNEL] Build Complete ---"

abi:
	@test -f $(SPECS_INC_VAULT)/yai_vault_abi.h || \
	 (echo "ERROR: Missing yai_vault_abi.h in ../deps/yai-law/contracts/vault/include"; exit 1)

dirs:
	@mkdir -p \
		$(BUILD_DIR)/boot/src \
		$(BUILD_DIR)/src/enforcement \
		$(BUILD_DIR)/src/core \
		$(BUILD_DIR)/src/bin \
		$(BUILD_DIR)/root/src \
		$(BIN_DIR)

$(TARGET_KERNEL): $(OBJS_BOOT) $(OBJS_KERNEL) $(CORE_CONTROL_OBJ) $(PROTOCOL_RUNTIME_LIB)
	@echo "[LINK] $@"
	$(CC) $^ -o $@ $(LDFLAGS)

$(CORE_CONTROL_OBJ): $(CORE_CONTROL_SRC) | dirs
	@echo "[CC] $<"
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/%.o: ../%.c | dirs
	@mkdir -p $(dir $@)
	@echo "[CC] $<"
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/src/%.o: src/%.c | dirs
	@mkdir -p $(dir $@)
	@echo "[CC] $<"
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

clean:
	@rm -rf $(BUILD_DIR)
	@echo "--- [YAI-KERNEL] Cleaned ---"

-include $(OBJS_BOOT:.o=.d)
-include $(OBJS_KERNEL:.o=.d)
-include $(CORE_CONTROL_OBJ:.o=.d)
