// pch.h: 預編譯頭文件

#pragma once

// Windows 標準頭文件
#include <windows.h>
#include <tchar.h>
#include <strsafe.h>

// TSF 相關頭文件
#include <msctf.h>
#include <textstor.h>
#include <imm.h>

// STL 標準庫
#include <string>
#include <vector>
#include <map>
#include <memory>
#include <algorithm>
#include <functional>

// 實用宏定義
#define SAFE_RELEASE(p) { if ((p) != nullptr) { (p)->Release(); (p) = nullptr; } }
#define RETURN_IF_FAILED(hr) { HRESULT __hr = (hr); if (FAILED(__hr)) { return __hr; } }

// 專案特定頭文件
#include "ime_module.h"
#include "ime_engine.h"

// DLL 導出宏
#ifdef MAIDOSIMECORE_EXPORTS
#define MAIDOS_API __declspec(dllexport)
#else
#define MAIDOS_API __declspec(dllimport)
#endif

// 錯誤處理
inline void LogError(const wchar_t* message) {
    OutputDebugStringW(message);
}

inline void LogInfo(const wchar_t* message) {
    OutputDebugStringW(message);
}