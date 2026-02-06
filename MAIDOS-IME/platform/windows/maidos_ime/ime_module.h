#ifndef _IME_MODULE_H_
#define _IME_MODULE_H_

#include <windows.h>
#include <imm.h>
#include <string>
#include <vector>

// 前向聲明
class ImeEngine;

#ifdef __cplusplus
extern "C" {
#endif

// 初始化 IME 模組
void InitializeImeModule();

// 清理 IME 模組
void CleanupImeModule();

// 啟用 IME 上下文
HRESULT ActivateImeContext(HIMC hIMC);

// 停用 IME 上下文
HRESULT DeactivateImeContext(HIMC hIMC);

// 處理鍵盤事件
BOOL ProcessImeKey(HIMC hIMC, UINT vKey, LPARAM lParam, const BYTE* lpbKeyState);

// 獲取組字字串
UINT GetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpBuf, UINT dwBufLen);

// 從核心引擎取得候選字詞
void GetCandidatesFromCore(const char* input, char* candidates, int bufferSize);

#ifdef __cplusplus
}

// 候選字結構
struct Candidate {
    std::wstring character;
    int frequency;
    std::vector<std::wstring> tags;
};

#endif

#endif // _IME_MODULE_H_
