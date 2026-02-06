#include "pch.h"
#include "pinyin_parser.h"
#include "dictionary.h"
#include <algorithm>
#include <iostream>

// Constructor
PinyinParser::PinyinParser(const Dictionary& dictionary) : m_dictionary(dictionary)
{
    std::wcout << L"[MAIDOS-AUDIT] PinyinParser initialized" << std::endl;
}

// Destructor
PinyinParser::~PinyinParser()
{
    std::wcout << L"[MAIDOS-AUDIT] PinyinParser destroyed" << std::endl;
}

// Parse single pinyin
std::vector<Dictionary::DictEntry> PinyinParser::ParseSinglePinyin(const std::wstring& pinyin) const
{
    std::wcout << L"[MAIDOS-AUDIT] Parse single pinyin: " << pinyin << std::endl;
    
    auto entries = m_dictionary.Lookup(pinyin);
    
    // Sort by frequency (highest first)
    std::sort(entries.begin(), entries.end(), 
              [](const Dictionary::DictEntry& a, const Dictionary::DictEntry& b) {
                  return a.frequency > b.frequency;
              });
    
    std::wcout << L"[MAIDOS-AUDIT] Found " << entries.size() << " entries for pinyin: " << pinyin << std::endl;
    return entries;
}

// Parse continuous pinyin
PinyinParser::ParseResult PinyinParser::ParseContinuousPinyin(const std::wstring& pinyinSequence)
{
    std::wcout << L"[MAIDOS-AUDIT] Parse continuous pinyin: " << pinyinSequence << std::endl;
    
    // Check cache first
    auto it = m_cache.find(pinyinSequence);
    if (it != m_cache.end())
    {
        std::wcout << L"[MAIDOS-AUDIT] Cache hit for pinyin sequence: " << pinyinSequence << std::endl;
        return it->second;
    }
    
    // Generate candidates if not in cache
    auto candidates = GenerateCandidates(pinyinSequence);
    
    ParseResult result;
    result.candidates.reserve(candidates.size());
    result.frequencies.reserve(candidates.size());
    
    for (const auto& candidate : candidates)
    {
        result.candidates.push_back(candidate.word);
        result.frequencies.push_back(candidate.frequency);
    }
    
    // Cache the result
    m_cache[pinyinSequence] = result;
    
    std::wcout << L"[MAIDOS-AUDIT] Generated " << candidates.size() << " candidates for sequence: " << pinyinSequence << std::endl;
    return result;
}

// Get dictionary
const Dictionary& PinyinParser::GetDictionary() const
{
    return m_dictionary;
}

// Clear cache
void PinyinParser::ClearCache()
{
    std::wcout << L"[MAIDOS-AUDIT] Cache cleared, size=" << m_cache.size() << std::endl;
    m_cache.clear();
}

// Generate candidates
std::vector<Dictionary::DictEntry> PinyinParser::GenerateCandidates(const std::wstring& pinyinSequence) const
{
    std::wcout << L"[MAIDOS-AUDIT] Generating candidates for sequence: " << pinyinSequence << std::endl;
    
    std::vector<Dictionary::DictEntry> candidates;
    
    // Try exact match first
    auto entries = m_dictionary.Lookup(pinyinSequence);
    if (!entries.empty())
    {
        candidates.insert(candidates.end(), entries.begin(), entries.end());
    }
    
    // If no exact match and sequence is long enough, try word segmentation
    if (candidates.empty() && pinyinSequence.length() > 1)
    {
        std::wcout << L"[MAIDOS-AUDIT] Attempting word segmentation..." << std::endl;
        
        for (size_t i = 1; i < pinyinSequence.length(); ++i)
        {
            std::wstring left = pinyinSequence.substr(0, i);
            std::wstring right = pinyinSequence.substr(i);
            
            auto leftEntries = m_dictionary.Lookup(left);
            auto rightEntries = m_dictionary.Lookup(right);
            
            if (!leftEntries.empty() && !rightEntries.empty())
            {
                std::wcout << L"[MAIDOS-AUDIT] Segmentation found: " << left << " + " << right << std::endl;
                
                for (const auto& leftEntry : leftEntries)
                {
                    for (const auto& rightEntry : rightEntries)
                    {
                        Dictionary::DictEntry combinedEntry;
                        combinedEntry.word = leftEntry.word + rightEntry.word;
                        combinedEntry.frequency = std::min(leftEntry.frequency, rightEntry.frequency);
                        combinedEntry.pronunciation = leftEntry.pronunciation + L" " + rightEntry.pronunciation;
                        combinedEntry.tags = { L"combined" };
                        
                        candidates.push_back(combinedEntry);
                    }
                }
            }
        }
    }
    
    // Sort by frequency (highest first)
    std::sort(candidates.begin(), candidates.end(), 
              [](const Dictionary::DictEntry& a, const Dictionary::DictEntry& b) {
                  return a.frequency > b.frequency;
              });
    
    // Remove duplicates
    candidates.erase(std::unique(candidates.begin(), candidates.end(), 
                                 [](const Dictionary::DictEntry& a, const Dictionary::DictEntry& b) {
                                     return a.word == b.word;
                                 }), 
                     candidates.end());
    
    // Limit to top 20 candidates
    if (candidates.size() > 20)
    {
        candidates.resize(20);
    }
    
    std::wcout << L"[MAIDOS-AUDIT] Generated " << candidates.size() << " unique candidates" << std::endl;
    return candidates;
}