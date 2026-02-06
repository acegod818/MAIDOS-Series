// MAIDOS IME ?†æ?æ¸¬è©¦
// æ¸¬è©¦ C++ ??C# çµ„ä»¶?„å??Œå·¥ä½?
#include <iostream>
#include "ime_engine.h"

int main() {
    std::cout << "=== MAIDOS IME INTEGRATION TESTS ===" << std::endl;
    
    // æ¸¬è©¦å¼•æ??å???    ImeEngine engine;
    if (engine.Initialize("src/config/maidos.toml")) {
        std::cout << "??Engine initialization: PASS" << std::endl;
    } else {
        std::cout << "??Engine initialization: FAIL" << std::endl;
        return 1;
    }
    
    // æ¸¬è©¦?ºæœ¬è¼¸å…¥?•ç?
    auto candidates = engine.ProcessInput("nihao");
    if (!candidates.empty()) {
        std::cout << "??Input processing: PASS (" << candidates.size() << " candidates)" << std::endl;
    } else {
        std::cout << "??Input processing: FAIL" << std::endl;
        return 1;
    }
    
    std::cout << "=== ALL INTEGRATION TESTS PASSED ===" << std::endl;
    return 0;
}
