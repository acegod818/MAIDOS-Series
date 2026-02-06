#include "pch.h"
#include "ime_module.h"
#include <iostream>
#include <vector>

// 單例實例指針
ImeModule* ImeModule::s_instance = nullptr;

// DLL 入口點
extern "C" BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
    switch (fdwReason) {
    case DLL_PROCESS_ATTACH:
        // 初始化 TSF 模組
        if (ImeModule::GetInstance()->Initialize() != S_OK) {
            return FALSE;
        }
        break;
    case DLL_PROCESS_DETACH:
        // 清理 TSF 模組
        if (ImeModule::GetInstance()) {
            ImeModule::GetInstance()->Uninitialize();
        }
        break;
    }
    return TRUE;
}

// TSF 接口回調函數實現
extern "C" HRESULT WINAPI ImeInquire(LPIMEINFO lpImeInfo) {
    if (!lpImeInfo) {
        return E_INVALIDARG;
    }
    
    // 初始化 IMEINFO 結構
    ZeroMemory(lpImeInfo, sizeof(IMEINFO));
    
    lpImeInfo->dwPrivateDataSize = 0;
    lpImeInfo->fdwProperty = IME_PROP_AT_CARET | IME_PROP_SPECIAL_UI | IME_PROP_NEED_ALTKEY;
    lpImeInfo->fdwConversionCaps = IME_CMODE_NATIVE | IME_CMODE_FULLSHAPE;
    lpImeInfo->fdwSentenceCaps = IME_SMODE_NONE;
    lpImeInfo->fdwUICaps = UI_CAP_2700;
    lpImeInfo->fdwSCSCaps = 0;
    lpImeInfo->fdwSelectCaps = SELECT_CAP_CONVERSION | SELECT_CAP_SENTENCE;
    
    return S_OK;
}

extern "C" HRESULT WINAPI ImeConfigure(HKL hKL, HWND hWnd, DWORD dwMode, LPVOID lpData) {
    // 顯示配置對話框
    MessageBoxW(hWnd, L"MAIDOS IME 配置對話框", L"MAIDOS IME", MB_OK | MB_ICONINFORMATION);
    return S_OK;
}

extern "C" HRESULT WINAPI ImeProcessKey(HIMC hIMC, UINT vKey, LPARAM lParam, CONST BYTE* lpbKeyState) {
    if (!hIMC) {
        return E_INVALIDARG;
    }
    
    // 只處理字母和數字鍵
    if ((vKey >= 'A' && vKey <= 'Z') || (vKey >= '0' && vKey <= '9')) {
        // 處理鍵盤輸入
        return ImeModule::GetInstance()->ProcessKeyInput(hIMC, vKey, lParam, lpbKeyState);
    }
    
    return S_FALSE; // 不處理此按鍵
}

extern "C" HRESULT WINAPI ImeSelect(HIMC hIMC, BOOL fSelect) {
    if (!hIMC) {
        return E_INVALIDARG;
    }
    
    return ImeModule::GetInstance()->SetOpenStatus(hIMC, fSelect);
}

extern "C" HRESULT WINAPI ImeToAsciiEx(UINT uVirtKey, UINT uScanCode, CONST BYTE* lpbKeyState, 
                                      LPTRANSMSGLIST lpTransMsgList, UINT fuState, HIMC hIMC) {
    if (!lpTransMsgList) {
        return E_INVALIDARG;
    }
    
    // 簡單實現：返回 WM_CHAR 消息
    lpTransMsgList->uMsgCount = 1;
    lpTransMsgList->TransMsg[0].message = WM_CHAR;
    lpTransMsgList->TransMsg[0].wParam = uVirtKey;
    lpTransMsgList->TransMsg[0].lParam = 1; // 重複計數
    
    return S_OK;
}

extern "C" HRESULT WINAPI NotifyIME(HIMC hIMC, DWORD dwAction, DWORD dwIndex, DWORD dwValue) {
    if (!hIMC) {
        return E_INVALIDARG;
    }
    
    switch (dwAction) {
    case NI_COMPOSITIONSTR:
        // 處理組字串通知
        break;
    case NI_OPENCANDIDATE:
        // 打開候選字視窗
        break;
    case NI_CLOSECANDIDATE:
        // 關閉候選字視窗
        break;
    case NI_SELECTCANDIDATESTR:
        // 選擇候選字
        break;
    }
    
    return S_OK;
}

extern "C" HRESULT WINAPI ImeSetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpComp, DWORD dwCompLen, LPVOID lpRead, DWORD dwReadLen) {
    if (!hIMC) {
        return E_INVALIDARG;
    }
    
    return ImeModule::GetInstance()->SetCompositionString(hIMC, dwIndex, lpComp, dwCompLen);
}

extern "C" HRESULT WINAPI ImeGetImeMenuItems(HIMC hIMC, DWORD dwFlags, DWORD dwType, 
                                           LPIMEMENUITEMINFOW lpImeParentMenu, 
                                           LPIMEMENUITEMINFOW lpImeMenu, DWORD dwSize, LPDWORD pdwResult) {
    if (!hIMC || !pdwResult) {
        return E_INVALIDARG;
    }
    
    *pdwResult = 0; // 沒有菜單項
    return S_OK;
}

// ImeModule 類實現
ImeModule::ImeModule() : m_threadMgr(nullptr), m_clientId(TF_CLIENTID_NULL) {
}

ImeModule::~ImeModule() {
    Uninitialize();
}

ImeModule* ImeModule::GetInstance() {
    if (!s_instance) {
        s_instance = new ImeModule();
    }
    return s_instance;
}

HRESULT ImeModule::Initialize() {
    HRESULT hr = CoCreateInstance(CLSID_TF_ThreadMgr, nullptr, CLSCTX_INPROC_SERVER,
                                 IID_ITfThreadMgr, reinterpret_cast<void**>(&m_threadMgr));
    if (FAILED(hr)) {
        return hr;
    }
    
    hr = m_threadMgr->Activate(&m_clientId);
    return hr;
}

HRESULT ImeModule::Uninitialize() {
    if (m_threadMgr) {
        if (m_clientId != TF_CLIENTID_NULL) {
            m_threadMgr->Deactivate();
            m_clientId = TF_CLIENTID_NULL;
        }
        m_threadMgr->Release();
        m_threadMgr = nullptr;
    }
    return S_OK;
}

HRESULT ImeModule::ProcessKeyInput(HIMC hIMC, UINT vKey, LPARAM lParam, CONST BYTE* lpbKeyState) {
    // 記錄輸入處理
    std::wcout << L"Processing key: " << static_cast<wchar_t>(vKey) << std::endl;
    
    // 處理鍵盤輸入邏輯
    // 這裡會調用拼音解析、候選字生成等
    
    return S_OK;
}

HRESULT ImeModule::SetOpenStatus(HIMC hIMC, BOOL fOpen) {
    // 設置 IME 開啟狀態
    std::wcout << L"Setting IME open status: " << fOpen << std::endl;
    return S_OK;
}

HRESULT ImeModule::GetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpBuf, DWORD dwBufLen) {
    // 獲取組字串
    return S_OK;
}

HRESULT ImeModule::SetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpComp, DWORD dwCompLen) {
    // 設置組字串
    return S_OK;
}