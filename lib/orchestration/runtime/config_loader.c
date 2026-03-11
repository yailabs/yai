#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

#include <yai/orchestration/runtime.h>
#include <yai/orchestration/provider_gate.h>

/* Usiamo il nome nudo perché il Makefile ha -I./src/external */
#include "cJSON.h" 

/* --- HELPER: Lettura File --- */
static char* read_file_to_string(const char* path) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);

    char *string = (char *)malloc(fsize + 1);
    if (string) {
        size_t read_bytes = fread(string, 1, fsize, f);
        string[read_bytes] = 0;
    }
    fclose(f);
    return string;
}

/* --- CORE LOADER --- */
int yai_exec_config_load_initial(const char* config_path, yai_exec_config_t* out_cfg) {
    printf("[YAI-CONFIG] Loading from: %s\n", config_path);

    char* config_raw = read_file_to_string(config_path);
    if (!config_raw) return -1;

    cJSON *json = cJSON_Parse(config_raw);
    if (!json) { 
        free(config_raw); 
        return -1; 
    }

    /* * Sostituito cJSON_GetObjectItemCaseSensitive con cJSON_GetObjectItem
     * Sostituito cJSON_IsNumber con controllo diretto del bitmask nel type
     */
    if (out_cfg) {
        cJSON *parallel = cJSON_GetObjectItem(json, "max_parallel_agents");
        if (parallel && (parallel->type & cJSON_Number)) {
            out_cfg->max_parallel_agents = (uint32_t)parallel->valueint;
        }
    }

    cJSON *active = cJSON_GetObjectItem(json, "active");
    if (active && (active->type & cJSON_Object)) {
        yai_provider_config_t p_cfg;
        memset(&p_cfg, 0, sizeof(p_cfg));
        
        cJSON *endpoint = cJSON_GetObjectItem(active, "endpoint");
        if (endpoint && (endpoint->type & cJSON_String) && endpoint->valuestring) {
            char temp[256];
            strncpy(temp, endpoint->valuestring, 255);
            temp[255] = '\0';
            
            char *host_start = strstr(temp, "://");
            if (host_start) {
                host_start += 3;
                char *port_start = strchr(host_start, ':');
                char *path_start = strchr(host_start, '/');

                if (port_start && path_start) {
                    *port_start = '\0';
                    *path_start = '\0';
                    strncpy(p_cfg.host, host_start, 127);
                    p_cfg.port = atoi(port_start + 1);
                    strncpy(p_cfg.endpoint, endpoint->valuestring + (path_start - temp), 127);
                }
            }
        }

        cJSON *model = cJSON_GetObjectItem(active, "model");
        if (model && (model->type & cJSON_String) && model->valuestring) {
            strncpy(p_cfg.id, model->valuestring, 31);
        } else {
            strncpy(p_cfg.id, "active_provider", 31);
        }

        yai_provider_gate_init(&p_cfg);
    }

    free(config_raw);
    cJSON_Delete(json);
    return 0;
}

bool yai_exec_config_enforce_limits(yai_exec_config_t* cfg) {
    if (!cfg) return false;
    if (cfg->max_parallel_agents > 32) return false;
    return true;
}
