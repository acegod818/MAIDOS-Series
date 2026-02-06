#pragma once

#include "pch.h"
#include "dictionary.h"
#include "pinyin_parser.h"
#include "converter.h"
#include "schemes.h"
#include "bopomofo_scheme.h"
#include <string>
#include <vector>
#include <memory>
#include <map>

// Candidate Manager class
class CandidateManager {
public:
    // Constructor
    explicit CandidateManager(PinyinParser& parser);
    
    // Destructor
    ~CandidateManager();
    
    // Get candidates for pinyin input
    std::vector<std::wstring> GetCandidates(const std::wstring& pinyinInput);
    
    // Get candidates with priority (AI-enhanced)
    std::vector<std::wstring> GetSmartCandidates(const std::wstring& pinyinInput, const std::wstring& context);
    
    // Get candidate frequency
    unsigned int GetCandidateFrequency(const std::wstring& candidate);
    
    // Select candidate by index
    bool SelectCandidate(int index, const std::vector<std::wstring>& candidates);
    
    // Get selected candidate
    std::wstring GetSelectedCandidate() const;
    
    // Clear selection
    void ClearSelection();
    
    // Check if selection is valid
    bool HasValidSelection() const;
    
    // Reset candidate state
    void Reset();
    
    // Add user preference for candidate
    void AddUserPreference(const std::wstring& pinyin, const std::wstring& candidate, int preferenceBoost);
    
    // Get candidate suggestions based on usage history
    std::vector<std::wstring> GetSmartSuggestions(const std::wstring& pinyinInput);

private:
    PinyinParser& m_parser;
    std::wstring m_selectedCandidate;
    std::map<std::wstring, std::map<std::wstring, int>> m_userPreferences;
};

// IME Engine class
class ImeEngine {
public:
    // Candidate structure
    struct Candidate {
        std::wstring character;
        int frequency;
        std::vector<std::wstring> tags;
    };

    // Constructor
    ImeEngine();

    // Destructor
    ~ImeEngine();

    // Initialize engine
    bool Initialize(const std::wstring& configPath);

    // Process input
    std::vector<Candidate> ProcessInput(const std::wstring& input, const std::wstring& context = L"");

    // Select character
    wchar_t SelectCharacter(const std::wstring& context, const std::vector<wchar_t>& candidates);

    // Auto correct
    std::wstring AutoCorrect(const std::wstring& text);

    // Smart suggestions
    std::vector<std::wstring> SmartSuggestions(const std::wstring& text);

    // Process cross input
    std::wstring ProcessCrossInput(const std::wstring& input, const std::wstring& context, 
                                 const std::wstring& scheme, const std::wstring& charset);

    // Get cross candidates
    std::vector<Candidate> GetCrossCandidates(const std::wstring& input, 
                                            const std::wstring& scheme, const std::wstring& charset);

private:
    // Configuration
    bool m_aiSelectionEnabled;
    bool m_autoCorrectionEnabled;
    bool m_smartSuggestionsEnabled;
    std::wstring m_defaultScheme;
    std::wstring m_charset;

    // Components
    std::unique_ptr<Dictionary> m_dictionary;
    std::unique_ptr<PinyinParser> m_pinyinParser;
    std::unique_ptr<CharsetConverter> m_converter;
    std::map<std::wstring, std::unique_ptr<InputScheme>> m_schemes;

    // Helper methods
    void LoadConfiguration(const std::wstring& configPath);
    std::vector<Candidate> GetCandidatesFromScheme(const std::wstring& input, const std::wstring& schemeName);
};