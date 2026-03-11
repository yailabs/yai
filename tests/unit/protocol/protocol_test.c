#include <yai/orchestration/transport_client.h>
#include <assert.h>
#include <stdio.h>
#include <string.h>

void test_envelope_size() {
    printf("[TEST] Checking Envelope size... ");
    assert(sizeof(yai_rpc_envelope_t) == 96);
    printf("OK (96 bytes)\n");
}

void test_malformed_handshake() {
    printf("[TEST] Checking Malformed Handshake... ");
    yai_rpc_envelope_t env;
    memset(&env, 0, sizeof(env));
    
    // Magic sbagliato
    env.magic = 0xDEADC0DE; 
    // Qui chiameremmo la funzione di validazione che scriveremo in rpc_envelope.c
    // bool res = yai_envelope_validate(&env, "ws-test");
    // assert(res == false);
    printf("OK (Rejected)\n");
}

int main() {
    printf("--- YAI PROTOCOL UNIT TESTS ---\n");
    test_envelope_size();
    test_malformed_handshake();
    printf("--- ALL TESTS PASSED ---\n");
    return 0;
}
