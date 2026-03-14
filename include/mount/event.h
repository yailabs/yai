#pragma once

#ifndef INCLUDE_MOUNT_EVENT_H
#define INCLUDE_MOUNT_EVENT_H

enum yai_mount_event_kind {
    YAI_MOUNT_EVENT_ATTACH = 0,
    YAI_MOUNT_EVENT_DETACH,
    YAI_MOUNT_EVENT_REMOUNT,
    YAI_MOUNT_EVENT_FAIL
};

#endif
