#pragma once

#ifndef YAI_KERNEL_POLICY_H
#define YAI_KERNEL_POLICY_H

#include <stdint.h>

#include <yai/security/containment.h>
#include "mount_policy.h"
#include "objects.h"
#include "session.h"

enum yai_kernel_policy_result {
    YAI_KERNEL_POLICY_ALLOW = 0,
    YAI_KERNEL_POLICY_DENY = 1,
    YAI_KERNEL_POLICY_DEFER = 2,
    YAI_KERNEL_POLICY_REQUIRE_PRIVILEGED_PATH = 3
};

int yai_kernel_policy_can_admit_session(
    yai_object_id_t subject_handle,
    const struct yai_kernel_session_request* request,
    enum yai_kernel_policy_result* out_result);

int yai_kernel_policy_can_bind_container(
    yai_object_id_t subject_handle,
    yai_object_id_t session_id,
    yai_object_id_t container_id,
    enum yai_kernel_policy_result* out_result);

int yai_kernel_policy_can_mount(
    yai_object_id_t subject_handle,
    yai_object_id_t container_id,
    enum yai_mount_policy_class mount_policy,
    enum yai_kernel_policy_result* out_result);

int yai_kernel_policy_can_escape(
    yai_object_id_t subject_handle,
    yai_object_id_t container_id,
    enum yai_escape_policy_class requested_class,
    enum yai_kernel_policy_result* out_result);

int yai_kernel_policy_can_spawn(
    yai_object_id_t subject_handle,
    yai_object_id_t container_id,
    enum yai_kernel_policy_result* out_result);

#endif /* YAI_KERNEL_POLICY_H */
