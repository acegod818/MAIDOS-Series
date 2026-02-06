#pragma once

#include "pch.h"
#include "schemes.h"
#include "dictionary.h"
#include <string>
#include <vector>
#include <map>
#include <memory>

// Bopomofo input scheme class
class BopomofoScheme : public InputScheme {
public:
    // Constructor
    BopomofoScheme();

    // Destructor
    virtual ~BopomofoScheme();

    // Process input
    std::vector<Candidate> ProcessInput(const std::wstring& input) override;

    // Get candidates
    std::vector<Candidate> GetCandidates(const std::wstring& input) override;

    // Add word
    void AddWord(const std::wstring& word, int frequency = 0) override;

    // Remove word
    void RemoveWord(const std::wstring& word) override;

    // Initialize bopomofo scheme
    bool Initialize();

private:
    // Bopomofo to Pinyin mapping
    std::map<std::wstring, std::wstring> m_bopomofoToPinyin;
    
    // User words
    std::map<std::wstring, int> m_userWords;

    // Dictionary (bopomofo.dict.json) used as the real data source for candidates.
    std::unique_ptr<Dictionary> m_dictionary;
    bool m_dictionaryLoaded;
    
    // Initialize bopomofo mapping
    void InitializeBopomofoMapping();

    // Ensure dictionary is loaded from disk (soft-config path resolution).
    bool EnsureDictionaryLoaded();

    // Normalize input for dictionary lookup (trim + collapse whitespace).
    std::wstring NormalizeForLookup(const std::wstring& input) const;
    
    // Parse bopomofo input
    std::wstring ParseBopomofoInput(const std::wstring& input) const;
    
    // Convert bopomofo to pinyin
    std::wstring ConvertBopomofoToPinyin(const std::wstring& bopomofo) const;
    
    // Validate bopomofo input
    bool IsValidBopomofoInput(const std::wstring& input) const;
};
