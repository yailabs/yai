#include <yai/orchestration/engine_cortex.h>

static int clamp_int(int v, int lo, int hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

engine_cortex_config_t engine_cortex_default_config(void) {
    engine_cortex_config_t cfg;
    cfg.tick_ms = 100;
    cfg.ewma_alpha = 0.2f;
    cfg.up_threshold = 30.0f;
    cfg.down_threshold = 10.0f;
    cfg.peak_delta = 20.0f;
    cfg.up_hold_ms = 500;
    cfg.down_hold_ms = 2000;
    cfg.cooldown_up_ms = 3000;
    cfg.cooldown_down_ms = 5000;
    cfg.min_target = 1;
    cfg.max_target = 8;
    cfg.step_up = 1;
    cfg.step_down = 1;
    return cfg;
}

int engine_cortex_validate_config(const engine_cortex_config_t* cfg) {
    if (!cfg) return -1;
    if (cfg->tick_ms == 0) return -2;
    if (cfg->ewma_alpha <= 0.0f || cfg->ewma_alpha > 1.0f) return -3;
    if (cfg->down_threshold >= cfg->up_threshold) return -4;
    if (cfg->min_target <= 0 || cfg->max_target < cfg->min_target) return -5;
    if (cfg->step_up <= 0 || cfg->step_down <= 0) return -6;
    return 0;
}

void engine_cortex_init(engine_cortex_state_t* st, const engine_cortex_config_t* cfg, int initial_target) {
    if (!st || !cfg) return;
    st->mode = CORTEX_STABLE;
    st->queue_ewma = 0.0f;
    st->above_ms = 0;
    st->below_ms = 0;
    st->cooldown_ms = 0;
    st->target = clamp_int(initial_target, cfg->min_target, cfg->max_target);
}

engine_cortex_decision_t engine_cortex_tick(engine_cortex_state_t* st, const engine_cortex_config_t* cfg, int queue_depth) {
    engine_cortex_decision_t out;
    out.triggered = 0;
    out.direction = 0;
    out.reason = "none";
    out.prev_target = st ? st->target : 0;
    out.new_target = st ? st->target : 0;
    out.queue_depth = queue_depth;
    out.queue_ewma = st ? st->queue_ewma : 0.0f;
    out.peak_delta = 0.0f;

    if (!st || !cfg) return out;

    if (queue_depth < 0) queue_depth = 0;

    if (st->queue_ewma == 0.0f) {
        st->queue_ewma = (float)queue_depth;
    } else {
        st->queue_ewma = (cfg->ewma_alpha * (float)queue_depth) + ((1.0f - cfg->ewma_alpha) * st->queue_ewma);
    }

    out.queue_ewma = st->queue_ewma;
    out.peak_delta = (float)queue_depth - st->queue_ewma;

    if (st->queue_ewma > cfg->up_threshold) {
        st->above_ms += cfg->tick_ms;
    } else {
        st->above_ms = 0;
    }

    if (st->queue_ewma < cfg->down_threshold) {
        st->below_ms += cfg->tick_ms;
    } else {
        st->below_ms = 0;
    }

    if (st->cooldown_ms > 0) {
        if (st->cooldown_ms <= cfg->tick_ms) st->cooldown_ms = 0;
        else st->cooldown_ms -= cfg->tick_ms;
        return out;
    }

    {
        int peak = out.peak_delta > cfg->peak_delta;
        int should_up = (st->above_ms >= cfg->up_hold_ms) || peak;
        int should_down = (st->below_ms >= cfg->down_hold_ms);

        if (should_up) {
            int next = clamp_int(st->target + cfg->step_up, cfg->min_target, cfg->max_target);
            if (next != st->target) {
                out.triggered = 1;
                out.direction = 1;
                out.reason = peak ? "peak_detected" : "ewma_over_threshold";
                out.prev_target = st->target;
                out.new_target = next;
                st->target = next;
                st->mode = CORTEX_COOLDOWN_UP;
                st->cooldown_ms = cfg->cooldown_up_ms;
                st->above_ms = 0;
                st->below_ms = 0;
            }
            return out;
        }

        if (should_down) {
            int next = clamp_int(st->target - cfg->step_down, cfg->min_target, cfg->max_target);
            if (next != st->target) {
                out.triggered = 1;
                out.direction = -1;
                out.reason = "ewma_below_threshold";
                out.prev_target = st->target;
                out.new_target = next;
                st->target = next;
                st->mode = CORTEX_COOLDOWN_DOWN;
                st->cooldown_ms = cfg->cooldown_down_ms;
                st->above_ms = 0;
                st->below_ms = 0;
            }
            return out;
        }
    }

    st->mode = CORTEX_STABLE;
    return out;
}
