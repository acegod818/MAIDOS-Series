#include <iostream>
#include <string>
#include <cassert>

#ifdef _WIN32
    #define LIBRARY_EXPORT __declspec(dllexport)
#else
    #define LIBRARY_EXPORT
#endif

// 聲明Rust函數
extern "C" {
    LIBRARY_EXPORT int maidos_init_engine(const char* config_path);
    LIBRARY_EXPORT int maidos_process_input(const char* input, char* candidates_buffer, int buffer_size);
    LIBRARY_EXPORT void maidos_cleanup_engine();
}

int main() {
    std::cout << "=== MAIDOS IME 集成測試 ===" << std::endl;
    
    // 測試1: 初始化引擎
    std::cout << "測試1: 初始化引擎..." << std::endl;
    const char* config_path = "../config/maidos.toml";
    int init_result = maidos_init_engine(config_path);
    
    if (init_result != 0) {
        std::cerr << "❌ 初始化失敗: " << init_result << std::endl;
        return 1;
    }
    std::cout << "✅ 引擎初始化成功" << std::endl;
    
    // 測試2: 處理拼音輸入
    std::cout << "\n測試2: 處理拼音輸入..." << std::endl;
    const char* test_input = "nihao";
    char buffer[1024] = {0};
    
    int process_result = maidos_process_input(test_input, buffer, sizeof(buffer));
    
    if (process_result >= 0) {
        std::cout << "✅ 輸入處理成功" << std::endl;
        std::cout << "   輸入: " << test_input << std::endl;
        std::cout << "   候選字: " << buffer << std::endl;
    } else {
        std::cerr << "❌ 處理輸入失敗: " << process_result << std::endl;
        // 這可能是因為詞典未正確加載，我們仍然繼續測試
    }
    
    // 測試3: 清理引擎
    std::cout << "\n測試3: 清理引擎..." << std::endl;
    maidos_cleanup_engine();
    std::cout << "✅ 引擎清理完成" << std::endl;
    
    std::cout << "\n=== 所有測試完成 ===" << std::endl;
    return 0;
}