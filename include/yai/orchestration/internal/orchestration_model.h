#pragma once

#include <stdint.h>

typedef enum {
    CORTEX_STABLE = 0,
    CORTEX_COOLDOWN_UP,
    CORTEX_COOLDOWN_DOWN
} engine_cortex_mode_t;

typedef struct {
    uint32_t tick_ms;
    float ewma_alpha;
    float up_threshold;
    float down_threshold;
    float peak_delta;
    uint32_t up_hold_ms;
    uint32_t down_hold_ms;
    uint32_t cooldown_up_ms;
    uint32_t cooldown_down_ms;
    int min_target;
    int max_target;
    int step_up;
    int step_down;
} engine_cortex_config_t;

typedef struct {
    engine_cortex_mode_t mode;
    float queue_ewma;
    uint32_t above_ms;
    uint32_t below_ms;
    uint32_t cooldown_ms;
    int target;
} engine_cortex_state_t;

typedef struct {
    int triggered;
    int direction;
    const char *reason;
    int prev_target;
    int new_target;
    int queue_depth;
    float queue_ewma;
    float peak_delta;
} engine_cortex_decision_t;

engine_cortex_config_t engine_cortex_default_config(void);
int engine_cortex_validate_config(const engine_cortex_config_t *cfg);
void engine_cortex_init(engine_cortex_state_t *st, const engine_cortex_config_t *cfg, int initial_target);
engine_cortex_decision_t engine_cortex_tick(engine_cortex_state_t *st, const engine_cortex_config_t *cfg, int queue_depth);
