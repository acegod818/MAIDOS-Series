#pragma once

#include "pch.h"
#include "dictionary.h"
#include <string>
#include <vector>
#include <map>

// Pinyin parser class
class PinyinParser {
public:
    // Parse result structure
    struct ParseResult {
        std::vector<std::wstring> candidates;
        std::vector<unsigned int> frequencies;
    };

    // Constructor
    PinyinParser(const Dictionary& dictionary);

    // Destructor
    ~PinyinParser();

    // Parse single pinyin
    std::vector<Dictionary::DictEntry> ParseSinglePinyin(const std::wstring& pinyin) const;

    // Parse continuous pinyin
    ParseResult ParseContinuousPinyin(const std::wstring& pinyinSequence);

    // Get dictionary
    const Dictionary& GetDictionary() const;

    // Clear cache
    void ClearCache();

private:
    std::vector<Dictionary::DictEntry> GenerateCandidates(const std::wstring& pinyinSequence) const;

    const Dictionary& m_dictionary;
    std::map<std::wstring, ParseResult> m_cache;
};