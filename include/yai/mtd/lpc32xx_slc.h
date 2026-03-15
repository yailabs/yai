/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Platform data for LPC32xx SoC SLC NAND controller
 *
 * Copyright © 2012 Roland Stigge
 */

#ifndef __LINUX_MTD_LPC32XX_SLC_H
#define __LINUX_MTD_LPC32XX_SLC_H

#include <yai/dmaengine.h>

struct lpc32xx_slc_platform_data {
	dma_filter_fn dma_filter;
};

#endif  /* __LINUX_MTD_LPC32XX_SLC_H */
