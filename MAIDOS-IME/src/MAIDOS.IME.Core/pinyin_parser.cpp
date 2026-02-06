#include "pch.h"
#include "pinyin_parser.h"
#include <algorithm>

// Constructor
PinyinParser::PinyinParser(const Dictionary& dictionary) : m_dictionary(dictionary)
{
}

// Destructor
PinyinParser::~PinyinParser()
{
}

// Parse single pinyin
std::vector<Dictionary::DictEntry> PinyinParser::ParseSinglePinyin(const std::wstring& pinyin) const
{
    auto entries = m_dictionary.Lookup(pinyin);
    
    std::sort(entries.begin(), entries.end(), 
              [](const Dictionary::DictEntry& a, const Dictionary::DictEntry& b) {
                  return a.frequency > b.frequency;
              });
    
    return entries;
}

// Parse continuous pinyin
PinyinParser::ParseResult PinyinParser::ParseContinuousPinyin(const std::wstring& pinyinSequence)
{
    auto it = m_cache.find(pinyinSequence);
    if (it != m_cache.end())
    {
        return it->second;
    }
    
    auto candidates = GenerateCandidates(pinyinSequence);
    
    ParseResult result;
    result.candidates.reserve(candidates.size());
    result.frequencies.reserve(candidates.size());
    
    for (const auto& candidate : candidates)
    {
        result.candidates.push_back(candidate.word);
        result.frequencies.push_back(candidate.frequency);
    }
    
    m_cache[pinyinSequence] = result;
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
    m_cache.clear();
}

// Generate candidates
std::vector<Dictionary::DictEntry> PinyinParser::GenerateCandidates(const std::wstring& pinyinSequence) const
{
    std::vector<Dictionary::DictEntry> candidates;
    
    auto entries = m_dictionary.Lookup(pinyinSequence);
    if (!entries.empty())
    {
        candidates.insert(candidates.end(), entries.begin(), entries.end());
    }
    
    if (candidates.empty() && pinyinSequence.length() > 1)
    {
        for (size_t i = 1; i < pinyinSequence.length(); ++i)
        {
            std::wstring left = pinyinSequence.substr(0, i);
            std::wstring right = pinyinSequence.substr(i);
            
            auto leftEntries = m_dictionary.Lookup(left);
            auto rightEntries = m_dictionary.Lookup(right);
            
            if (!leftEntries.empty() && !rightEntries.empty())
            {
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
    
    std::sort(candidates.begin(), candidates.end(), 
              [](const Dictionary::DictEntry& a, const Dictionary::DictEntry& b) {
                  return a.frequency > b.frequency;
              });
    
    candidates.erase(std::unique(candidates.begin(), candidates.end(), 
                                 [](const Dictionary::DictEntry& a, const Dictionary::DictEntry& b) {
                                     return a.word == b.word;
                                 }), 
                     candidates.end());
    
    if (candidates.size() > 20)
    {
        candidates.resize(20);
    }
    
    return candidates;
}