#pragma once

#include "pch.h"
#include <string>
#include <vector>

// 輸入法引擎核心類別
class ImeEngine {
public:
    ImeEngine();
    ~ImeEngine();
    
    // 初始化/清理
    HRESULT Initialize();
    HRESULT Uninitialize();
    
    // 輸入處理
    HRESULT ProcessInput(const std::wstring& input, const std::wstring& context);
    HRESULT SelectCandidate(int candidateIndex);
    HRESULT ClearComposition();
    
    // 組字操作
    HRESULT SetComposition(const std::wstring& composition);
    HRESULT GetComposition(std::wstring& composition) const;
    
    // 候選字操作
    HRESULT GetCandidates(std::vector<std::wstring>& candidates);
    HRESULT GetCandidateCount(size_t& count) const;
    
    // 配置管理
    HRESULT SetInputScheme(const std::wstring& scheme);
    HRESULT SetCharset(const std::wstring& charset);
    HRESULT SetAISelectionEnabled(bool enabled);
    HRESULT SetAutoCorrectionEnabled(bool enabled);
    
    // 狀態查詢
    bool IsOpen() const;
    bool IsComposing() const;
    bool HasCandidates() const;
    
    // AI 功能
    HRESULT GetSmartSuggestions(const std::wstring& context, std::vector<std::wstring>& suggestions);
    HRESULT AutoCorrect(const std::wstring& input, std::wstring& corrected);
    
    // 語音輸入
    HRESULT ProcessVoiceInput(const std::string& audioData, std::wstring& text);
    
private:
    // 內部狀態
    bool m_initialized{false};
    bool m_open{false};
    bool m_composing{false};
    bool m_aiEnabled{true};
    bool m_autoCorrectionEnabled{true};
    
    // 當前組字和候選字
    std::wstring m_composition;
    std::vector<std::wstring> m_candidates;
    size_t m_selectedCandidate{0};
    
    // 配置
    std::wstring m_inputScheme{L"pinyin"};
    std::wstring m_charset{L"Traditional"};
    
    // 詞典和 AI 管理
    std::vector<std::pair<std::wstring, int>> m_dictionary; // 詞典: 詞彙和頻率
    
    // 內部方法
    HRESULT LoadDictionary();
    HRESULT SaveDictionary();
    HRESULT ProcessPinyinInput(const std::wstring& pinyin);
    HRESULT ProcessBopomofoInput(const std::wstring& bopomofo);
    HRESULT ProcessCangjieInput(const std::wstring& cangjie);
    
    // AI 集成
    HRESULT CallAIForCandidates(const std::wstring& input, const std::wstring& context);
    HRESULT CallAIForCorrection(const std::wstring& input, std::wstring& corrected);
    
    // 拼音解析
    std::vector<std::wstring> ParsePinyin(const std::wstring& pinyin);
    std::vector<std::wstring> FindCandidatesFromDictionary(const std::vector<std::wstring>& pinyinTokens);
    
    // 字符集轉換
    std::wstring ConvertToTraditional(const std::wstring& simplified);
    std::wstring ConvertToSimplified(const std::wstring& traditional);
    
    // 手寫識別（佔位）
    HRESULT ProcessHandwriting(const std::vector<POINT>& strokes, std::wstring& recognized);
};

// 候選字結構
struct CandidateInfo {
    std::wstring text;
    int frequency{0};
    std::vector<std::wstring> tags;
};

// 組字信息
struct CompositionInfo {
    std::wstring text;
    size_t caretPosition{0};
    std::vector<size_t> segmentOffsets;
};

// 輸入法配置
struct ImeConfig {
    std::wstring defaultScheme{L"pinyin"};
    std::wstring defaultCharset{L"Traditional"};
    bool aiSelectionEnabled{true};
    bool autoCorrectionEnabled{true};
    bool smartSuggestionsEnabled{true};
    int maxCandidates{9};
    
    // AI 配置
    std::wstring aiModelPath{L"./models/maidos-llm"};
    std::wstring whisperModelPath{L"./models/whisper"};
    int maxContextLength{2048};
    
    // 詞典配置
    std::wstring dictionaryPath{L"./dicts/basic.dict.json"};
    std::wstring userDictionaryPath{L"./dicts/user.dict.json"};
};

// 導出函數聲明
extern "C" {
    MAIDOS_API ImeEngine* CreateImeEngine();
    MAIDOS_API void DestroyImeEngine(ImeEngine* engine);
    MAIDOS_API HRESULT InitializeImeEngine(ImeEngine* engine);
    MAIDOS_API HRESULT UninitializeImeEngine(ImeEngine* engine);
    
    // 輸入處理
    MAIDOS_API HRESULT ProcessImeInput(ImeEngine* engine, const wchar_t* input, const wchar_t* context);
    MAIDOS_API HRESULT GetImeCandidates(ImeEngine* engine, wchar_t** candidates, size_t* count);
    MAIDOS_API HRESULT SelectImeCandidate(ImeEngine* engine, int index);
    MAIDOS_API HRESULT ClearImeComposition(ImeEngine* engine);
    
    // 配置管理
    MAIDOS_API HRESULT SetImeInputScheme(ImeEngine* engine, const wchar_t* scheme);
    MAIDOS_API HRESULT SetImeCharset(ImeEngine* engine, const wchar_t* charset);
    MAIDOS_API HRESULT SetImeAISelection(ImeEngine* engine, BOOL enabled);
    MAIDOS_API HRESULT SetImeAutoCorrection(ImeEngine* engine, BOOL enabled);
}