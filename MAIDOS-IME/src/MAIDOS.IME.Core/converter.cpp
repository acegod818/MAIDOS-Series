#include "pch.h"
#include "converter.h"
#include <algorithm>

// Constructor
CharsetConverter::CharsetConverter()
{
    InitializeMaps();
}

// Destructor
CharsetConverter::~CharsetConverter()
{
}

// Convert text
std::wstring CharsetConverter::Convert(const std::wstring& text, const std::wstring& from, const std::wstring& to)
{
    if (from == to)
    {
        return text;
    }

    std::wstring result = text;
    
    if (from == L"Simplified" && to == L"Traditional")
    {
        for (size_t i = 0; i < result.length(); ++i)
        {
            auto it = m_s2t.find(result[i]);
            if (it != m_s2t.end())
            {
                result[i] = it->second;
            }
        }
    }
    else if (from == L"Traditional" && to == L"Simplified")
    {
        for (size_t i = 0; i < result.length(); ++i)
        {
            auto it = m_t2s.find(result[i]);
            if (it != m_t2s.end())
            {
                result[i] = it->second;
            }
        }
    }

    return result;
}

// Convert candidates
std::vector<wchar_t> CharsetConverter::ConvertCandidates(const std::vector<wchar_t>& candidates, Charset from, Charset to)
{
    if (from == to)
    {
        return candidates;
    }

    std::vector<wchar_t> result = candidates;
    
    if (from == Charset::Simplified && to == Charset::Traditional)
    {
        for (size_t i = 0; i < result.size(); ++i)
        {
            auto it = m_s2t.find(result[i]);
            if (it != m_s2t.end())
            {
                result[i] = it->second;
            }
        }
    }
    else if (from == Charset::Traditional && to == Charset::Simplified)
    {
        for (size_t i = 0; i < result.size(); ++i)
        {
            auto it = m_t2s.find(result[i]);
            if (it != m_t2s.end())
            {
                result[i] = it->second;
            }
        }
    }

    return result;
}

// Initialize maps
void CharsetConverter::InitializeMaps()
{
    // Minimal mapping for testing (ASCII placeholders to avoid encoding issues)
    m_s2t[L'A'] = L'A';
    m_s2t[L'B'] = L'B';
    
    m_t2s[L'A'] = L'A';
    m_t2s[L'B'] = L'B';
}