#include "ime_module.h"
#include <string>
#include <vector>
#include <iostream>
#include <memory>
#include <algorithm>
#include <locale>
#include <codecvt>

// 包含C++核心引擎頭文件
#include "ime_engine.h"

// 全局引擎實例
static std::unique_ptr<ImeEngine> g_engine;
// 當前輸入緩衝區
static std::wstring inputBuffer;
// 候選字詞列表
static std::vector<Candidate> candidateList;

// 初始化 IME 模組
void InitializeImeModule() {
    // 初始化 C++ 核心引擎
    g_engine = std::make_unique<ImeEngine>();
    
    // 初始化引擎
    std::wstring configPath = L"src/config/maidos.toml"; // 配置文件路徑
    g_engine->Initialize(configPath);
    
    inputBuffer.clear();
    candidateList.clear();
}

// 清理 IME 模組
void CleanupImeModule() {
    g_engine.reset();
    inputBuffer.clear();
    candidateList.clear();
}

// 啟用 IME 上下文
HRESULT ActivateImeContext(HIMC hIMC) {
    // 設置 IME 狀態為開啟
    ImmAssociateContext(GetFocus(), hIMC);
    return S_OK;
}

// 停用 IME 上下文
HRESULT DeactivateImeContext(HIMC hIMC) {
    // 設置 IME 狀態為關閉
    ImmAssociateContext(GetFocus(), NULL);
    return S_OK;
}

// 處理鍵盤事件
BOOL ProcessImeKey(HIMC hIMC, UINT vKey, LPARAM lParam, const BYTE* lpbKeyState) {
    // 簡單示例：只處理字母鍵和空格鍵
    if (vKey >= 'A' && vKey <= 'Z') {
        // 添加字母到輸入緩衝區
        inputBuffer += static_cast<wchar_t>(towlower(vKey));
        return TRUE;
    } else if (vKey == VK_SPACE) {
        // 空格鍵觸發選字
        GetCandidatesFromCore(nullptr, nullptr, 0);
        inputBuffer.clear();
        return TRUE;
    } else if (vKey == VK_BACK) {
        // 退格鍵刪除最後一個字符
        if (!inputBuffer.empty()) {
            inputBuffer.pop_back();
        }
        return TRUE;
    }
    
    return FALSE;
}

// 獲取組字字串
UINT GetCompositionString(HIMC hIMC, DWORD dwIndex, LPVOID lpBuf, UINT dwBufLen) {
    switch (dwIndex) {
        case GCS_COMPSTR: {
            // 返回當前組字字串
            if (lpBuf && dwBufLen > 0) {
                // 將寬字符串轉換為多字節字符串
                std::string narrowBuffer(inputBuffer.begin(), inputBuffer.end());
                strncpy_s(static_cast<char*>(lpBuf), dwBufLen, narrowBuffer.c_str(), _TRUNCATE);
                return static_cast<UINT>(narrowBuffer.length());
            }
            break;
        }
        case GCS_RESULTSTR: {
            // 返回結果字串（假設第一個候選字為結果）
            if (!candidateList.empty() && lpBuf && dwBufLen > 0) {
                // 將寬字符串轉換為多字節字符串
                std::string narrowResult(candidateList[0].character.begin(), candidateList[0].character.end());
                strncpy_s(static_cast<char*>(lpBuf), dwBufLen, narrowResult.c_str(), _TRUNCATE);
                return static_cast<UINT>(narrowResult.length());
            }
            break;
        }
    }
    return 0;
}

// 從核心引擎取得候選字詞
void GetCandidatesFromCore(const char* input, char* candidates, int bufferSize) {
    // 清空候選列表
    candidateList.clear();
    
    // 如果提供了輸入，使用提供的輸入；否則使用內部緩衝區
    std::wstring inputStr;
    if (input && strlen(input) > 0) {
        // 將多字節字符串轉換為寬字符串
        std::string narrowInput(input);
        inputStr = std::wstring(narrowInput.begin(), narrowInput.end());
    } else {
        inputStr = inputBuffer;
    }
    
    // 調用 C++ 核心引擎獲取候選字詞
    if (!inputStr.empty() && g_engine) {
        auto engineCandidates = g_engine->ProcessInput(inputStr);
        candidateList = engineCandidates;
    }
    
    // 如果提供了緩衝區，則填充候選字詞
    if (candidates && bufferSize > 0) {
        std::string result;
        for (const auto& candidate : candidateList) {
            // 將寬字符串轉換為多字節字符串
            std::string narrowCandidate(candidate.character.begin(), candidate.character.end());
            result += narrowCandidate + " ";
        }
        
        strncpy_s(candidates, bufferSize, result.c_str(), _TRUNCATE);
    }
}
