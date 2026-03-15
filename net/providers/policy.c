#define _DEFAULT_SOURCE
#define _POSIX_C_SOURCE 200809L

#include <yai/net/providers/policy.h>
#include <yai/orch/runtime.h>
#include "cJSON.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <errno.h>
#include <sys/time.h>

static yai_provider_config_t current_provider;
static yai_provider_policy_t g_provider_policy = {
    .allow_mock_providers = 1,
    .min_trust_for_embedding = YAI_PROVIDER_TRUST_SANDBOXED,
    .min_trust_for_inference = YAI_PROVIDER_TRUST_SANDBOXED,
};

void yai_provider_gate_init(const yai_provider_config_t* config) {
    if (config) {
        memcpy(&current_provider, config, sizeof(yai_provider_config_t));
        fprintf(stderr, "[PROVIDER_GATE] L2 Route: %s -> %s:%d%s\n", 
                current_provider.id, current_provider.host, 
                current_provider.port, current_provider.endpoint);
    }
}

/**
 * Micro-client HTTP Sovereign (Socket TCP Puri)
 * Legge la risposta in modo incrementale per gestire payload grandi
 */
static char* http_post_raw(const char* host, int port, const char* path, const char* body) {
    int sockfd;
    struct sockaddr_in serv_addr;
    struct hostent *server;
    char request[YAI_EXEC_RPC_BUFFER_MAX];

    sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) return NULL;

    server = gethostbyname(host);
    if (!server) {
        close(sockfd);
        return NULL;
    }

    memset(&serv_addr, 0, sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;
    memcpy(&serv_addr.sin_addr.s_addr, server->h_addr_list[0], server->h_length);
    serv_addr.sin_port = htons(port);

    // Timeout per evitare che l'Engine si blocchi se il llama-server è offline
    struct timeval tv;
    tv.tv_sec = 30; // 30 secondi di grazia per l'inferenza
    tv.tv_usec = 0;
    setsockopt(sockfd, SOL_SOCKET, SO_RCVTIMEO, (const char*)&tv, sizeof tv);

    if (connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) {
        close(sockfd);
        return NULL;
    }

    // Costruzione request HTTP 1.1 standard
    snprintf(request, sizeof(request),
             "POST %s HTTP/1.1\r\n"
             "Host: %s\r\n"
             "Content-Type: application/json\r\n"
             "Content-Length: %zu\r\n"
             "Connection: close\r\n\r\n"
             "%s",
             path, host, strlen(body), body);

    if (write(sockfd, request, strlen(request)) < 0) {
        close(sockfd);
        return NULL;
    }

    // Lettura dinamica della risposta
    size_t res_cap = 16384;
    size_t res_len = 0;
    char *full_res = malloc(res_cap);
    if (!full_res) { close(sockfd); return NULL; }

    int n;
    while ((n = read(sockfd, full_res + res_len, res_cap - res_len - 1)) > 0) {
        res_len += n;
        if (res_len >= res_cap - 1) {
            res_cap *= 2;
            full_res = realloc(full_res, res_cap);
        }
    }
    full_res[res_len] = '\0';
    close(sockfd);

    // Estrazione del body (salto gli header HTTP)
    char *json_start = strstr(full_res, "\r\n\r\n");
    if (json_start) {
        char *json_body = strdup(json_start + 4);
        free(full_res);
        return json_body;
    }

    free(full_res);
    return strdup("{\"error\":\"upstream_http_malformed\"}");
}

char* yai_provider_gate_dispatch(const yai_rpc_envelope_t* env, const char* json_payload) {
    if (!current_provider.host[0]) {
        return strdup("{\"error\":\"provider_not_configured\"}");
    }

    fprintf(stderr, "[PROVIDER_GATE] Dispatching trace %s to LLM @ %s:%d\n", 
            env->trace_id, current_provider.host, current_provider.port);

    char* raw_response = http_post_raw(
        current_provider.host, 
        current_provider.port, 
        current_provider.endpoint, 
        json_payload
    );
    
    if (!raw_response) {
        fprintf(stderr, "[PROVIDER_GATE] CRITICAL: Connection failed to %s\n", current_provider.host);
        return strdup("{\"error\":\"upstream_connection_failed\"}");
    }

    return raw_response; 
}

void yai_provider_policy_default(yai_provider_policy_t *policy_out)
{
    if (!policy_out) return;
    *policy_out = g_provider_policy;
}

int yai_provider_set_policy(const yai_provider_policy_t *policy)
{
    if (!policy) return YAI_MIND_ERR_INVALID_ARG;
    g_provider_policy = *policy;
    return YAI_MIND_OK;
}

int yai_provider_get_policy(yai_provider_policy_t *policy_out)
{
    if (!policy_out) return YAI_MIND_ERR_INVALID_ARG;
    *policy_out = g_provider_policy;
    return YAI_MIND_OK;
}

int yai_provider_policy_admits(const yai_provider_descriptor_t *descriptor,
                               yai_provider_capability_t capability,
                               const yai_provider_policy_t *policy)
{
    const yai_provider_policy_t *effective_policy = policy ? policy : &g_provider_policy;
    yai_provider_trust_level_t min_trust;

    if (!descriptor) return 0;
    if ((descriptor->capability_mask & (unsigned int)capability) == 0u) return 0;

    min_trust = effective_policy->min_trust_for_inference;
    if (capability == YAI_PROVIDER_CAPABILITY_EMBEDDING) {
        min_trust = effective_policy->min_trust_for_embedding;
    }

    if (!effective_policy->allow_mock_providers && descriptor->is_mock) return 0;
    if (descriptor->trust_level < min_trust) return 0;
    return 1;
}
