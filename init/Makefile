# SPDX-License-Identifier: Apache-2.0
#
# init/Makefile - YAI init subtree
#

obj-y += main.o
obj-y += init_task.o
obj-y += version.o
obj-y += version-timestamp.o

# Transitional root/bootstrap mounting surface
obj-y += mounts.o
mounts-y += do_mounts.o
mounts-y += do_mounts_initrd.o
mounts-y += do_mounts_rd.o

# Optional local initramfs path
obj-y += initramfs.o
obj-y += noinitramfs.o

# Transitional support
obj-y += calibrate.o

# Keep function/data sections coherent during early bootstrap assembly
ccflags-y += -fno-function-sections -fno-data-sections

#
# YAI build identity
#

yai-smp-flag-$(CONFIG_SMP) := SMP
yai-preempt-flag-$(CONFIG_PREEMPT_BUILD) := PREEMPT
yai-preempt-flag-$(CONFIG_PREEMPT_DYNAMIC) := PREEMPT_DYNAMIC
yai-preempt-flag-$(CONFIG_PREEMPT_RT) := PREEMPT_RT

yai-build-version = $(or $(KBUILD_BUILD_VERSION),$(build-version-auto),0)
yai-build-timestamp = $(or $(KBUILD_BUILD_TIMESTAMP),$(build-timestamp-auto),unknown)

# Maximum length kept aligned with Linux-style uts version discipline
filechk_yai_uts_version = \
	utsver=$$(echo '$(pound)'"YAI $(yai-build-version) $(yai-smp-flag-y) $(yai-preempt-flag-y) $(yai-build-timestamp)" | cut -b -64); \
	echo '$(pound)'define YAI_UTS_VERSION \""$${utsver}"\"

$(obj)/yai-utsversion-tmp.h: FORCE
	$(call filechk,yai_uts_version)

clean-files += yai-utsversion-tmp.h

$(obj)/version.o: $(obj)/yai-utsversion-tmp.h
CFLAGS_version.o := -include $(obj)/yai-utsversion-tmp.h

include/generated/yai-utsversion.h: build-version-auto = $(shell test -x $(srctree)/scripts/build-version && $(srctree)/scripts/build-version || echo 0)
include/generated/yai-utsversion.h: build-timestamp-auto = $(shell LC_ALL=C date 2>/dev/null || echo unknown)
include/generated/yai-utsversion.h: FORCE
	$(call filechk,yai_uts_version)

$(obj)/version-timestamp.o: include/generated/yai-utsversion.h
CFLAGS_version-timestamp.o := -include include/generated/yai-utsversion.h