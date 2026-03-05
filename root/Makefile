# =========================================
# YAI — Root Control Plane
# =========================================

CC ?= cc

ROOT_DIR := $(abspath ..)

BUILD_ROOT ?= $(ROOT_DIR)/build
BIN_BUILD ?= $(BUILD_ROOT)/bin

COMPONENT_BUILD_DIR ?= $(BUILD_ROOT)/root
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
	-I../kernel/include \
	-I$(SPECS_DIR) \
	-I$(SPECS_INC_PROTOCOL) \
	-I$(SPECS_INC_RUNTIME) \
	-I$(SPECS_INC_VAULT)

ROOT_SRC := \
	src/yai_root_server.c \
	src/control_transport.c \
	src/core/commands/root_command_dispatch.c

# Single protocol runtime artifact (Fix #2)
PROTOCOL_RUNTIME_LIB := ../build/runtime-protocol/libruntime_protocol.a

OBJS_ROOT := $(patsubst src/%.c,$(BUILD_DIR)/src/%.o,$(ROOT_SRC))

TARGET := $(BIN_DIR)/yai-root-server

.PHONY: all build dirs clean

all: build

build: dirs $(TARGET)
	@echo "--- [YAI-ROOT] Build Complete ---"

dirs:
	@mkdir -p $(BUILD_DIR)/src $(BUILD_DIR)/src/core/commands $(BIN_DIR)

$(TARGET): $(OBJS_ROOT) $(PROTOCOL_RUNTIME_LIB)
	@echo "[LINK] $@"
	$(CC) $(OBJS_ROOT) $(PROTOCOL_RUNTIME_LIB) -o $@ $(LDFLAGS)

$(BUILD_DIR)/src/%.o: src/%.c | dirs
	@echo "[CC] $<"
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

clean:
	@rm -rf $(BUILD_DIR)
	@echo "--- [YAI-ROOT] Cleaned ---"

-include $(OBJS_ROOT:.o=.d)
