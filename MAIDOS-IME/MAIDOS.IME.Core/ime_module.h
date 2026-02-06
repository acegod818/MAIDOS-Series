#pragma once

#include <windows.h>
#include <msctf.h>
#include <string>

// Windows TSF API 定義
#define MAIDOSIMECORE_EXPORTS

// 主要 IME 模組類別
class ImeModule {
public:
    static ImeModule* GetInstance();
    
    // TSF 接口函數
    static HRESULT WINAPI ImeInquire(LPIMEINFO lpImeInfo);
    static HRESULT WINAPI ImeConfigure(HKL hKL, HWND hWnd, DWORD dwMode, LPVOID lpData);
    static HRESULT WINAPI ImeProcessKey(HIMC hIMC, UINT vKey, LPARAM lParam, CONST BYTE* lpbKeyState);
    static HRESULT WINAPI ImeSelect(HIMC hIMC, BOOL fSelect);
    static HRESULT WINAPI ImeToAsciiEx(UINT uVirtKey, UINT uScanCode, CONST BYTE* lpbKeyState, LPTRANSMSGLIST lpTransMsgList, UINT fuState, HIMC hIMC);
    
    // 組字和候選字操作
    static HRESULT WINAPI NotifyIME(HIMC hIMC, DWORD dwAction, DWORD dwIndex, DWORD dwValue);
    static HRESULT WINAPI ImeSetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpComp, DWORD dwCompLen, LPVOID lpRead, DWORD dwReadLen);
    static HRESULT WINAPI ImeGetImeMenuItems(HIMC hIMC, DWORD dwFlags, DWORD dwType, LPIMEMENUITEMINFOW lpImeParentMenu, LPIMEMENUITEMINFOW lpImeMenu, DWORD dwSize, LPDWORD pdwResult);
    
private:
    ImeModule();
    ~ImeModule();
    
    static ImeModule* s_instance;
    
    // TSF 管理器
    ITfThreadMgr* m_threadMgr;
    TfClientId m_clientId;
    
    // 初始化/清理方法
    HRESULT Initialize();
    HRESULT Uninitialize();
    
    // 輸入處理
    HRESULT ProcessKeyInput(HIMC hIMC, UINT vKey, LPARAM lParam, CONST BYTE* lpbKeyState);
    HRESULT GetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpBuf, DWORD dwBufLen);
    HRESULT SetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpComp, DWORD dwCompLen);
    
    // 候選字操作
    HRESULT GetCandidateList(HIMC hIMC, DWORD dwIndex, LPCANDIDATELIST lpCandList, DWORD dwBufLen);
    HRESULT GetCandidateListCount(HIMC hIMC, LPDWORD pdwListSize, DWORD dwBufLen);
    
    // 配置和狀態
    HRESULT SetOpenStatus(HIMC hIMC, BOOL fOpen);
    HRESULT GetOpenStatus(HIMC hIMC, LPBOOL pfOpen);
};

// 導出函數聲明（TSF 標準接口）
extern "C" {
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI ImeInquire(LPIMEINFO lpImeInfo);
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI ImeConfigure(HKL hKL, HWND hWnd, DWORD dwMode, LPVOID lpData);
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI ImeProcessKey(HIMC hIMC, UINT vKey, LPARAM lParam, CONST BYTE* lpbKeyState);
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI ImeSelect(HIMC hIMC, BOOL fSelect);
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI ImeToAsciiEx(UINT uVirtKey, UINT uScanCode, CONST BYTE* lpbKeyState, LPTRANSMSGLIST lpTransMsgList, UINT fuState, HIMC hIMC);
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI NotifyIME(HIMC hIMC, DWORD dwAction, DWORD dwIndex, DWORD dwValue);
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI ImeSetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpComp, DWORD dwCompLen, LPVOID lpRead, DWORD dwReadLen);
    MAIDOSIMECORE_EXPORTS HRESULT WINAPI ImeGetImeMenuItems(HIMC hIMC, DWORD dwFlags, DWORD dwType, LPIMEMENUITEMINFOW lpImeParentMenu, LPIMEMENUITEMINFOW lpImeMenu, DWORD dwSize, LPDWORD pdwResult);
    
    // DLL 入口點
    MAIDOSIMECORE_EXPORTS BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved);
}