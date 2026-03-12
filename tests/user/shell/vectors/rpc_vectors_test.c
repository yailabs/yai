#include <assert.h>
#include <yai_protocol_ids.h>

int main(void) {
    assert(YAI_CMD_PING != 0u);
    assert(YAI_CMD_CONTROL != 0u);
    return 0;
}
