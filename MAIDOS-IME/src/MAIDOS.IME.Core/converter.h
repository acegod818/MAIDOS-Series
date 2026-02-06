#pragma once

#include "pch.h"
#include <string>
#include <vector>
#include <map>

// Charset enumeration
enum class Charset {
    Simplified,
    Traditional
};

// Charset converter class
class CharsetConverter {
public:
    // Constructor
    CharsetConverter();

    // Destructor
    ~CharsetConverter();

    // Convert text
    std::wstring Convert(const std::wstring& text, const std::wstring& from, const std::wstring& to);

    // Convert candidates
    std::vector<wchar_t> ConvertCandidates(const std::vector<wchar_t>& candidates, Charset from, Charset to);

private:
    std::map<wchar_t, wchar_t> m_s2t;
    std::map<wchar_t, wchar_t> m_t2s;

    void InitializeMaps();
};