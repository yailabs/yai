#include <yai/orchestration/internal/orchestration_model.h>
#include <stdio.h>
#include <string.h>

#define MAX_EVENTS 256

typedef struct {
    int dir[MAX_EVENTS];
    int count;
} event_log_t;

static void run_seq(const int* q, int n, engine_cortex_config_t cfg, int init_target, event_log_t* log) {
    engine_cortex_state_t st;
    engine_cortex_init(&st, &cfg, init_target);
    log->count = 0;
    for (int i = 0; i < n; i++) {
        engine_cortex_decision_t d = engine_cortex_tick(&st, &cfg, q[i]);
        if (d.triggered && log->count < MAX_EVENTS) {
            log->dir[log->count++] = d.direction;
        }
    }
}

static int test_no_flap(void) {
    engine_cortex_config_t cfg = engine_cortex_default_config();
    cfg.up_threshold = 60.0f;
    cfg.down_threshold = 20.0f;
    cfg.peak_delta = 100.0f;
    int q[120];
    for (int i = 0; i < 120; i++) q[i] = (i % 2 == 0) ? 35 : 45;
    event_log_t log;
    run_seq(q, 120, cfg, 3, &log);
    return log.count == 0 ? 0 : 1;
}

static int test_scale_up(void) {
    engine_cortex_config_t cfg = engine_cortex_default_config();
    int q[40];
    for (int i = 0; i < 40; i++) q[i] = 100;
    event_log_t log;
    run_seq(q, 40, cfg, 1, &log);
    if (log.count < 1) return 1;
    if (log.dir[0] != 1) return 2;
    return 0;
}

static int test_scale_down(void) {
    engine_cortex_config_t cfg = engine_cortex_default_config();
    int q[80];
    for (int i = 0; i < 80; i++) q[i] = 0;
    event_log_t log;
    run_seq(q, 80, cfg, 4, &log);
    if (log.count < 1) return 1;
    if (log.dir[0] != -1) return 2;
    return 0;
}

static int test_peak(void) {
    engine_cortex_config_t cfg = engine_cortex_default_config();
    cfg.up_hold_ms = 2000;
    int q[20];
    for (int i = 0; i < 20; i++) q[i] = 10;
    q[6] = 80;
    q[7] = 80;
    event_log_t log;
    run_seq(q, 20, cfg, 1, &log);
    if (log.count < 1) return 1;
    if (log.dir[0] != 1) return 2;
    return 0;
}

static int test_determinism(void) {
    engine_cortex_config_t cfg = engine_cortex_default_config();
    int q[60];
    for (int i = 0; i < 60; i++) q[i] = (i < 20) ? 50 : (i < 40 ? 5 : 80);

    event_log_t a, b;
    run_seq(q, 60, cfg, 2, &a);
    run_seq(q, 60, cfg, 2, &b);

    if (a.count != b.count) return 1;
    for (int i = 0; i < a.count; i++) {
        if (a.dir[i] != b.dir[i]) return 2;
    }
    return 0;
}

int main(void) {
    struct {
        const char* name;
        int (*fn)(void);
    } tests[] = {
        {"no_flap", test_no_flap},
        {"scale_up", test_scale_up},
        {"scale_down", test_scale_down},
        {"peak", test_peak},
        {"determinism", test_determinism},
    };

    int failed = 0;
    for (unsigned i = 0; i < sizeof(tests) / sizeof(tests[0]); i++) {
        int rc = tests[i].fn();
        if (rc != 0) {
            fprintf(stderr, "FAIL: %s rc=%d\n", tests[i].name, rc);
            failed = 1;
        }
    }
    if (failed) return 1;
    printf("OK: cortex harness passed\n");
    return 0;
}
