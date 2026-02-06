#include <windows.h>
#include <imm.h>
#include "ime_module.h"

// 全局實例句柄
HINSTANCE g_hInst;

// DLL 入口點
BOOL WINAPI DllMain(HINSTANCE hInstance, DWORD dwReason, LPVOID lpReserved) {
    switch (dwReason) {
        case DLL_PROCESS_ATTACH:
            g_hInst = hInstance;
            // 初始化 IME 模組
            InitializeImeModule();
            break;
            
        case DLL_PROCESS_DETACH:
            // 清理 IME 模組
            CleanupImeModule();
            break;
    }
    
    return TRUE;
}

// IME 組件入口點
__declspec(dllexport) HRESULT WINAPI ImeSelect(HIMC hIMC, BOOL fSelect) {
    if (fSelect) {
        return ActivateImeContext(hIMC);
    } else {
        return DeactivateImeContext(hIMC);
    }
}

// 處理鍵盤消息
__declspec(dllexport) BOOL WINAPI ImeProcessKey(HIMC hIMC, UINT vKey, LPARAM lParam, const BYTE* lpbKeyState) {
    return ProcessImeKey(hIMC, vKey, lParam, lpbKeyState);
}

// 獲取結果字串
__declspec(dllexport) UINT WINAPI ImeGetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpBuf, UINT dwBufLen) {
    return GetCompositionString(hIMC, dwIndex, lpBuf, dwBufLen);
}