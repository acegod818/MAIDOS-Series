// pch.cpp: 預編譯頭文件的實現

#include "pch.h"

// 預編譯頭文件的實現
// 這裡包含最常用的標準庫頭文件

// Windows API 版本檢測
#ifndef WINVER
#define WINVER 0x0A00 // Windows 10
#endif

#ifndef _WIN32_WINNT  
#define _WIN32_WINNT 0x0A00 // Windows 10
#endif

// TSF 版本檢測  
#ifndef _TSF_VER
#define _TSF_VER 0x0100
#endif

// COM 初始化檢查
static bool g_comInitialized = false;

bool InitializeCOM() {
    if (!g_comInitialized) {
        HRESULT hr = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
        if (SUCCEEDED(hr)) {
            g_comInitialized = true;
            return true;
        }
    }
    return g_comInitialized;
}

void UninitializeCOM() {
    if (g_comInitialized) {
        CoUninitialize();
        g_comInitialized = false;
    }
}

// 字符串工具函數
std::wstring StringToWString(const std::string& str) {
    int size_needed = MultiByteToWideChar(CP_UTF8, 0, str.c_str(), (int)str.size(), nullptr, 0);
    std::wstring wstr(size_needed, 0);
    MultiByteToWideChar(CP_UTF8, 0, str.c_str(), (int)str.size(), &wstr[0], size_needed);
    return wstr;
}

std::string WStringToString(const std::wstring& wstr) {
    int size_needed = WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), (int)wstr.size(), nullptr, 0, nullptr, nullptr);
    std::string str(size_needed, 0);
    WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), (int)wstr.size(), &str[0], size_needed, nullptr, nullptr);
    return str;
}

// UTF-8/UTF-16 轉換函數
std::wstring UTF8ToUTF16(const std::string& utf8) {
    return StringToWString(utf8);
}

std::string UTF16ToUTF8(const std::wstring& utf16) {
    return WStringToString(utf16);
}

// 錯誤處理函數
std::wstring GetLastErrorString() {
    DWORD error = GetLastError();
    if (error == 0) {
        return L"No error";
    }

    LPWSTR buffer = nullptr;
    DWORD length = FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        nullptr, error, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
        reinterpret_cast<LPWSTR>(&buffer), 0, nullptr);

    std::wstring message(buffer, length);
    LocalFree(buffer);
    return message;
}

// HRESULT 錯誤訊息
std::wstring GetHRESULTString(HRESULT hr) {
    LPWSTR buffer = nullptr;
    DWORD length = FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        nullptr, hr, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
        reinterpret_cast<LPWSTR>(&buffer), 0, nullptr);

    if (buffer) {
        std::wstring message(buffer, length);
        LocalFree(buffer);
        return message;
    }
    return L"Unknown HRESULT error";
}

// 日誌系統
void LogDebug(const wchar_t* format, ...) {
    wchar_t buffer[1024];
    va_list args;
    va_start(args, format);
    StringCchVPrintfW(buffer, ARRAYSIZE(buffer), format, args);
    va_end(args);
    
    OutputDebugStringW(buffer);
    OutputDebugStringW(L"\n");
}

void LogErrorHR(HRESULT hr, const wchar_t* context) {
    std::wstring errorMsg = GetHRESULTString(hr);
    LogDebug(L"[ERROR] %s: HRESULT=0x%08X - %s", context, hr, errorMsg.c_str());
}

// 性能計時器
class PerformanceTimer {
private:
    LARGE_INTEGER m_start;
    LARGE_INTEGER m_frequency;
    
public:
    PerformanceTimer() {
        QueryPerformanceFrequency(&m_frequency);
        Start();
    }
    
    void Start() {
        QueryPerformanceCounter(&m_start);
    }
    
    double Elapsed() const {
        LARGE_INTEGER end;
        QueryPerformanceCounter(&end);
        return static_cast<double>(end.QuadPart - m_start.QuadPart) / m_frequency.QuadPart;
    }
    
    double Restart() {
        double elapsed = Elapsed();
        Start();
        return elapsed;
    }
};

// 安全字符串操作
template<size_t size>
void SafeCopy(wchar_t(&dest)[size], const wchar_t* src) {
    StringCchCopyW(dest, size, src);
}

template<size_t size>
void SafeCopy(char(&dest)[size], const char* src) {
    StringCchCopyA(dest, size, src);
}

// GUID 生成器
class GuidGenerator {
public:
    static GUID Generate() {
        GUID guid;
        CoCreateGuid(&guid);
        return guid;
    }
    
    static std::wstring ToString(const GUID& guid) {
        wchar_t buffer[64];
        StringFromGUID2(guid, buffer, ARRAYSIZE(buffer));
        return std::wstring(buffer);
    }
};

// TSF 工具函數
class TSFHelper {
public:
    static HRESULT CreateCategoryManager(ITfCategoryMgr** ppCategoryMgr) {
        return CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                               IID_ITfCategoryMgr, reinterpret_cast<void**>(ppCategoryMgr));
    }
    
    static HRESULT CreateDisplayAttributeMgr(ITfDisplayAttributeMgr** ppDisplayAttributeMgr) {
        return CoCreateInstance(CLSID_TF_DisplayAttributeMgr, nullptr, CLSCTX_INPROC_SERVER,
                               IID_ITfDisplayAttributeMgr, reinterpret_cast<void**>(ppDisplayAttributeMgr));
    }
};