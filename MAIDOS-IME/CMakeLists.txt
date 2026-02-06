cmake_minimum_required(VERSION 3.15)
project(MAIDOS_IME_Core)

# 設置 C++ 標準
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 包含路徑
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src/MAIDOS.IME.Core)

# 源文件列表
set(SOURCES
    src/MAIDOS.IME.Core/pch.cpp
    src/MAIDOS.IME.Core/dictionary.cpp
    src/MAIDOS.IME.Core/pinyin_parser.cpp
    src/MAIDOS.IME.Core/schemes.cpp
    src/MAIDOS.IME.Core/converter.cpp
    src/MAIDOS.IME.Core/ime_engine.cpp
    src/MAIDOS.IME.Core/test_ime.cpp
)

# 頭文件列表
set(HEADERS
    src/MAIDOS.IME.Core/pch.h
    src/MAIDOS.IME.Core/dictionary.h
    src/MAIDOS.IME.Core/pinyin_parser.h
    src/MAIDOS.IME.Core/schemes.h
    src/MAIDOS.IME.Core/converter.h
    src/MAIDOS.IME.Core/ime_engine.h
)

# 編譯器特定選項
if(MSVC)
    add_compile_options(/W4 /WX)
else()
    add_compile_options(-Wall -Wextra -Werror)
endif()

# 可執行文件
add_executable(maidos_ime_core ${SOURCES} ${HEADERS})

# 輸出目錄
set_target_properties(maidos_ime_core PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin
)