cmake_minimum_required(VERSION 3.12)
project(maidos_ime_test)

set(CMAKE_CXX_STANDARD 17)

# 添加可執行文件
add_executable(test_ime test_ime.cpp)

# 鏈接Rust庫
# 注意：這裏需要根據實際的Rust庫位置進行調整
target_link_libraries(test_ime PRIVATE maidos_core)

# 設置包含目錄
target_include_directories(test_ime PRIVATE 
    ${CMAKE_CURRENT_SOURCE_DIR}/../../platform/windows/maidos_ime
)