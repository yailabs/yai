/* SPDX-License-Identifier: GPL-2.0 */
#ifndef __ASM_GENERIC_BITOPS_H
#define __ASM_GENERIC_BITOPS_H

/*
 * For the benefit of those who are trying to port Linux to another
 * architecture, here are some C-language equivalents.  They should
 * generate reasonable code, so take a look at what your compiler spits
 * out before rolling your own buggy implementation in assembly language.
 *
 * C language equivalents written by Theodore Ts'o, 9/26/92
 */

#include <yai/irqflags.h>
#include <yai/compiler.h>
#include <asm/barrier.h>

#include <yai/asm-generic/bitops/__ffs.h>
#include <yai/asm-generic/bitops/ffz.h>
#include <yai/asm-generic/bitops/fls.h>
#include <yai/asm-generic/bitops/__fls.h>
#include <yai/asm-generic/bitops/fls64.h>

#ifndef _LINUX_BITOPS_H
#error only <yai/bitops.h> can be included directly
#endif

#include <yai/asm-generic/bitops/sched.h>
#include <yai/asm-generic/bitops/ffs.h>
#include <yai/asm-generic/bitops/hweight.h>
#include <yai/asm-generic/bitops/lock.h>

#include <yai/asm-generic/bitops/atomic.h>
#include <yai/asm-generic/bitops/non-atomic.h>
#include <yai/asm-generic/bitops/le.h>
#include <yai/asm-generic/bitops/ext2-atomic.h>

#endif /* __ASM_GENERIC_BITOPS_H */
