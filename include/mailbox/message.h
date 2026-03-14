#pragma once

#ifndef INCLUDE_MAILBOX_MESSAGE_H
#define INCLUDE_MAILBOX_MESSAGE_H

#include <mailbox/types.h>

struct yai_mailbox_message {
    yai_mailbox_message_id_t id;
    const char *topic;
};

#endif
