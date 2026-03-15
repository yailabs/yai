/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_LINUX_LOGO_H
#define _LINUX_LINUX_LOGO_H

/*
 *  Linux logo to be displayed on boot
 *
 *  Copyright (C) 1996 Larry Ewing (lewing@isc.tamu.edu)
 *  Copyright (C) 1996,1998 Jakub Jelinek (jj@sunsite.mff.cuni.cz)
 *  Copyright (C) 2001 Greg Banks <gnb@alphalink.com.au>
 *  Copyright (C) 2001 Jan-Benedict Glaw <jbglaw@lug-owl.de>
 *  Copyright (C) 2003 Geert Uytterhoeven <geert@linux-m68k.org>
 */

#include <yai/init.h>


#define YAI_LOGO_MONO		1	/* monochrome black/white */
#define YAI_LOGO_VGA16	2	/* 16 colors VGA text palette */
#define YAI_LOGO_CLUT224	3	/* 224 colors */
#define YAI_LOGO_GRAY256	4	/* 256 levels grayscale */


struct yai_logo {
	int type;			/* one of YAI_LOGO_* */
	unsigned int width;
	unsigned int height;
	unsigned int clutsize;		/* YAI_LOGO_CLUT224 only */
	const unsigned char *clut;	/* YAI_LOGO_CLUT224 only */
	const unsigned char *data;
};

extern const struct yai_logo logo_linux_mono;
extern const struct yai_logo logo_linux_vga16;
extern const struct yai_logo logo_linux_clut224;
extern const struct yai_logo logo_spe_clut224;

extern const struct yai_logo *fb_find_logo(int depth);
#ifdef CONFIG_FB_LOGO_EXTRA
extern void fb_append_extra_logo(const struct yai_logo *logo,
				 unsigned int n);
#else
static inline void fb_append_extra_logo(const struct yai_logo *logo,
					unsigned int n)
{}
#endif

#endif /* _LINUX_LINUX_LOGO_H */
