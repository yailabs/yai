#include <yai/pol/events.h>

#ifndef YAI_RUNTIME_EVENT_SCHEMA_ID
#define YAI_RUNTIME_EVENT_SCHEMA_ID "yai.runtime.event.v1"
#endif

#ifndef YAI_RUNTIME_EVENT_VERSION
#define YAI_RUNTIME_EVENT_VERSION 1
#endif
#include <stdio.h>
#include <string.h>
#include <time.h>

// tiny helper: safe JSON string escaping (minimal; good enough for logs)
static void json_escape(FILE *out, const char *s) {
    if (!s) { fputs("", out); return; }
    for (const unsigned char *p = (const unsigned char*)s; *p; p++) {
        unsigned char c = *p;
        switch (c) {
            case '\"': fputs("\\\"", out); break;
            case '\\': fputs("\\\\", out); break;
            case '\n': fputs("\\n", out); break;
            case '\r': fputs("\\r", out); break;
            case '\t': fputs("\\t", out); break;
            default:
                if (c < 0x20) fputs(" ", out); // drop control chars
                else fputc((int)c, out);
        }
    }
}

void yai_log_static(
    int type,
    const char *ws_id,
    const char *trace_id,
    const char *level,
    const char *msg,
    const char *data_json
) {
    const time_t now = time(NULL);

    // data_json must be valid JSON; if missing, force null
    const char *data = (data_json && data_json[0] != '\0') ? data_json : "null";
    const char *lvl  = (level && level[0] != '\0') ? level : "info";
    const char *ws   = (ws_id && ws_id[0] != '\0') ? ws_id : "";
    const char *tr   = (trace_id && trace_id[0] != '\0') ? trace_id : "";

    // JSONL event (single line)
    // schema_id + event_version are always present.
    fprintf(stderr,
        "{\"schema_id\":\"%s\",\"event_version\":%d,"
        "\"ts\":%ld,\"ws_id\":\"",
        YAI_RUNTIME_EVENT_SCHEMA_ID,
        YAI_RUNTIME_EVENT_VERSION,
        (long)now
    );
    json_escape(stderr, ws);
    fputs("\",\"trace_id\":\"", stderr);
    json_escape(stderr, tr);
    fputs("\",\"type\":", stderr);
    fprintf(stderr, "%d", (int)type);
    fputs(",\"level\":\"", stderr);
    json_escape(stderr, lvl);
    fputs("\",\"msg\":\"", stderr);
    json_escape(stderr, msg ? msg : "");
    fputs("\",\"data\":", stderr);
    fputs(data, stderr);
    fputs("}\n", stderr);
}
