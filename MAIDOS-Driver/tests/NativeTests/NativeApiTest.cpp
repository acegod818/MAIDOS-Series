#include <windows.h>
#include <iostream>
#include <assert.h>

// [MAIDOS-AUDIT] Native API Unit Test Suite
// 用於驗證 C++ 核心組件的穩定性

typedef int (*SCAN_FUNC)(void*, int);
typedef int (*PROBLEM_FUNC)(const char*);

void TestScanner() {
    HMODULE hLib = LoadLibraryA("MAIDOS.Driver.Native.dll");
    assert(hLib != NULL);

    SCAN_FUNC scan = (SCAN_FUNC)GetProcAddress(hLib, "scan_hardware_native");
    assert(scan != NULL);

    // 模擬緩衝區
    char dummyBuffer[1024 * 100]; 
    int count = scan(dummyBuffer, 100);
    
    std::cout << "[TEST] Scanner found " << count << " devices." << std::endl;
    assert(count >= 0);

    FreeLibrary(hLib);
}

void TestDiagnostics() {
    HMODULE hLib = LoadLibraryA("MAIDOS.Driver.Native.dll");
    assert(hLib != NULL);

    PROBLEM_FUNC getProb = (PROBLEM_FUNC)GetProcAddress(hLib, "get_device_problem_code");
    assert(getProb != NULL);

    // 測試一個無效 ID
    int code = getProb("NON_EXISTENT_DEVICE_ID");
    assert(code == -1);

    std::cout << "[TEST] Diagnostics robust against invalid IDs." << std::endl;

    FreeLibrary(hLib);
}

int main() {
    std::cout << "Starting MAIDOS Native Tests..." << std::endl;
    TestScanner();
    TestDiagnostics();
    std::cout << "All Native Tests Passed!" << std::endl;
    return 0;
}