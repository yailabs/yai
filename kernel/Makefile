# =========================================
# YAI Kernel Build Spine
# =========================================

ROOT_DIR := $(abspath $(CURDIR)/..)

CC ?= cc
AR ?= ar

CPPFLAGS ?= -I$(ROOT_DIR) \
            -I$(ROOT_DIR)/kernel/include \
            -I$(ROOT_DIR)/legacy_include \
            -I$(ROOT_DIR)/vendor/cjson
CFLAGS ?= -std=c11 -Wall -Wextra -Werror -O2
LDFLAGS ?=
LDLIBS ?=

KERNEL_BUILD_DIR ?= $(CURDIR)/build
KERNEL_OBJ_DIR ?= $(KERNEL_BUILD_DIR)/obj
KERNEL_BIN_DIR ?= $(KERNEL_BUILD_DIR)/bin
KERNEL_LIB_DIR ?= $(KERNEL_BUILD_DIR)/lib

define find_c
$(sort $(shell find $(1) -type f -name '*.c' ! -name '*.inc.c' 2>/dev/null))
endef

KERNEL_LIBCORE_SRCS := $(call find_c,lib/core)
KERNEL_HAL_SRCS := $(call find_c,hal/core)
KERNEL_PROTOCOL_SRCS := $(call find_c,ipc)

KERNEL_SUBSTRATE_SRCS := \
        $(call find_c,arch) \
        $(call find_c,core) \
        $(call find_c,lifecycle) \
        $(call find_c,state) \
        $(call find_c,registry) \
        $(call find_c,policy) \
        $(call find_c,grants) \
        $(call find_c,security) \
        $(call find_c,session) \
        $(call find_c,container) \
        $(call find_c,net) \
        $(call find_c,proc) \
        $(call find_c,trace) \
        $(call find_c,mm) \
        $(call find_c,fs) \
        $(call find_c,drivers) \
        $(call find_c,scheduler)

KERNEL_EXCLUDE_SRCS := \
        ipc/message_types.c

KERNEL_SRCS := $(filter-out $(KERNEL_EXCLUDE_SRCS), \
        $(KERNEL_LIBCORE_SRCS) \
        $(KERNEL_HAL_SRCS) \
        $(KERNEL_PROTOCOL_SRCS) \
        $(KERNEL_SUBSTRATE_SRCS))

KERNEL_OBJS := $(patsubst %.c,$(KERNEL_OBJ_DIR)/%.o,$(KERNEL_SRCS))

KERNEL_LIB := $(KERNEL_LIB_DIR)/libyai_kernel.a

KERNEL_SMOKE_SRC := $(CURDIR)/tests/kernel_vertical_smoke.c
KERNEL_SMOKE_OBJ := $(KERNEL_OBJ_DIR)/tests/kernel_vertical_smoke.o
KERNEL_SMOKE_BIN := $(KERNEL_BIN_DIR)/kernel-vertical-smoke

KERNEL_DIRS := $(KERNEL_BUILD_DIR) $(KERNEL_OBJ_DIR) $(KERNEL_BIN_DIR) $(KERNEL_LIB_DIR)

.PHONY: all dirs check smoke clean help print-sources

all: $(KERNEL_LIB)
	@echo "[kernel] substrate built: $(KERNEL_LIB)"

check:
	$(CC) $(CPPFLAGS) $(CFLAGS) -fsyntax-only $(KERNEL_SRCS)

ifeq ($(wildcard $(KERNEL_SMOKE_SRC)),)
smoke: $(KERNEL_LIB)
	@echo "[kernel] smoke source not present: $(KERNEL_SMOKE_SRC)"
else
smoke: $(KERNEL_SMOKE_BIN)
	@$(KERNEL_SMOKE_BIN)
endif

$(KERNEL_LIB): $(KERNEL_OBJS) | dirs
	$(AR) rcs $@ $^

$(KERNEL_SMOKE_OBJ): $(KERNEL_SMOKE_SRC) | dirs
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

$(KERNEL_SMOKE_BIN): $(KERNEL_SMOKE_OBJ) $(KERNEL_LIB) | dirs
	@mkdir -p $(dir $@)
	$(CC) $(LDFLAGS) $(KERNEL_SMOKE_OBJ) -o $@ $(KERNEL_LIB) $(LDLIBS)

$(KERNEL_OBJ_DIR)/%.o: %.c | dirs
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

dirs:
	@mkdir -p $(KERNEL_DIRS)

clean:
	rm -rf $(KERNEL_BUILD_DIR)

print-sources:
	@printf "%s\n" $(KERNEL_SRCS)

help:
	@echo "kernel targets:"
	@echo "  all           build kernel substrate archive"
	@echo "  check         syntax-check curated kernel sources"
	@echo "  smoke         run kernel smoke if present"
	@echo "  print-sources print active kernel source set"
	@echo "  clean         remove kernel build artifacts"
