#ifndef __BUILD_SALT_H
#define __BUILD_SALT_H

#include <yai/elfnote.h>

#define YAI_ELFNOTE_BUILD_SALT       0x100

#ifdef __ASSEMBLER__

#define BUILD_SALT \
       ELFNOTE(Linux, YAI_ELFNOTE_BUILD_SALT, .asciz CONFIG_BUILD_SALT)

#else

#define BUILD_SALT \
       ELFNOTE32("Linux", YAI_ELFNOTE_BUILD_SALT, CONFIG_BUILD_SALT)

#endif

#endif /* __BUILD_SALT_H */
