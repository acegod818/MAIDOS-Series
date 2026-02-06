#include <iostream>
#include <string>
#include <cstring>

// 聲明Rust函數
extern "C" {
    int maidos_init_engine(const char* config_path);
    int maidos_process_input(const char* input, char* candidates_buffer, int buffer_size);
    void maidos_cleanup_engine();
}

int main() {
    std::cout << "測試 MAIDOS IME 核心功能" << std::endl;
    
    // 初始化引擎
    const char* config_path = "../config/maidos.toml";
    int init_result = maidos_init_engine(config_path);
    
    if (init_result != 0) {
        std::cerr << "初始化失敗: " << init_result << std::endl;
        return 1;
    }
    
    std::cout << "引擎初始化成功" << std::endl;
    
    // 測試拼音輸入
    const char* test_input = "nihao";
    char buffer[1024] = {0};
    
    int process_result = maidos_process_input(test_input, buffer, sizeof(buffer));
    
    if (process_result >= 0) {
        std::cout << "輸入 '" << test_input << "' 的候選字: " << buffer << std::endl;
    } else {
        std::cerr << "處理輸入失敗: " << process_result << std::endl;
    }
    
    // 清理
    maidos_cleanup_engine();
    
    std::cout << "測試完成" << std::endl;
    return 0;
}