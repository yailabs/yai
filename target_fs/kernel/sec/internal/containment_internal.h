#pragma once

#include <stdint.h>

int yai_security_compute_mount_decision(uint64_t mount_flags,
                                        uint64_t capability_mask,
                                        uint64_t *out_decision_flags);

int yai_security_compute_transition_decision(uint64_t lifecycle_state,
                                             uint64_t recovery_state,
                                             uint64_t *out_decision_flags);
