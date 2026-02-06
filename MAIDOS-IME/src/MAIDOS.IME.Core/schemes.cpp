#include "pch.h"
#include "schemes.h"
#include "pinyin_parser.h"
#include "bopomofo_scheme.h"
#include <algorithm>
#include <functional>

// PinyinScheme - Process input
std::vector<InputScheme::Candidate> PinyinScheme::ProcessInput(const std::wstring& input)
{
    return GetCandidates(input);
}

// PinyinScheme - Get candidates
std::vector<InputScheme::Candidate> PinyinScheme::GetCandidates(const std::wstring& input)
{
    if (!m_parser)
        return {};

    auto result = m_parser->ParseContinuousPinyin(input);

    std::vector<Candidate> candidates;
    for (size_t i = 0; i < result.candidates.size() && i < result.frequencies.size(); ++i)
    {
        Candidate c;
        c.character = result.candidates[i];
        c.frequency = static_cast<int>(result.frequencies[i]);
        candidates.push_back(c);
    }

    return candidates;
}

// PinyinScheme - Add word
void PinyinScheme::AddWord(const std::wstring& word, int frequency)
{
    m_userWords[word] = frequency;
}

// PinyinScheme - Remove word
void PinyinScheme::RemoveWord(const std::wstring& word)
{
    m_userWords.erase(word);
}

// CangjieScheme - Process input
std::vector<InputScheme::Candidate> CangjieScheme::ProcessInput(const std::wstring& input)
{
    return {};
}

// CangjieScheme - Get candidates
std::vector<InputScheme::Candidate> CangjieScheme::GetCandidates(const std::wstring& input)
{
    return {};
}

// CangjieScheme - Add word
void CangjieScheme::AddWord(const std::wstring& word, int frequency)
{
    m_userWords[word] = frequency;
}

// CangjieScheme - Remove word
void CangjieScheme::RemoveWord(const std::wstring& word)
{
    m_userWords.erase(word);
}

// Scheme factory - Create scheme
std::unique_ptr<InputScheme> SchemeFactory::CreateScheme(const std::wstring& schemeName)
{
    if (schemeName == L"pinyin")
    {
        return std::make_unique<PinyinScheme>();
    }
    else if (schemeName == L"bopomofo")
    {
        return std::make_unique<BopomofoScheme>();
    }
    else if (schemeName == L"cangjie")
    {
        return std::make_unique<CangjieScheme>();
    }
    
    return nullptr;
}