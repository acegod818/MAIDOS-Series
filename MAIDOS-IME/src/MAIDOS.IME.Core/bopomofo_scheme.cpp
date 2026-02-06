#include "pch.h"
#include "bopomofo_scheme.h"
#include <algorithm>
#include <cwctype>

extern HMODULE g_hModule;

namespace {

std::wstring GetEnvVarW(const wchar_t* name)
{
    wchar_t buf[32767];
    const DWORD len = GetEnvironmentVariableW(name, buf, static_cast<DWORD>(sizeof(buf) / sizeof(buf[0])));
    if (len == 0 || len >= (sizeof(buf) / sizeof(buf[0])))
    {
        return L"";
    }
    return std::wstring(buf, len);
}

std::wstring GetExeDirW()
{
    wchar_t buf[MAX_PATH];
    const HMODULE h = g_hModule ? g_hModule : nullptr;
    const DWORD len = GetModuleFileNameW(h, buf, static_cast<DWORD>(sizeof(buf) / sizeof(buf[0])));
    if (len == 0 || len >= (sizeof(buf) / sizeof(buf[0])))
    {
        return L"";
    }

    std::wstring path(buf, len);
    const size_t pos = path.find_last_of(L"\\/");
    if (pos == std::wstring::npos)
    {
        return L"";
    }
    return path.substr(0, pos);
}

std::wstring JoinPathW(const std::wstring& a, const std::wstring& b)
{
    if (a.empty()) return b;
    if (b.empty()) return a;
    if (a.back() == L'\\' || a.back() == L'/') return a + b;
    return a + L"\\" + b;
}

bool FileExistsW(const std::wstring& path)
{
    const DWORD attrs = GetFileAttributesW(path.c_str());
    if (attrs == INVALID_FILE_ATTRIBUTES)
    {
        return false;
    }
    return (attrs & FILE_ATTRIBUTE_DIRECTORY) == 0;
}

std::wstring TrimAndCollapseWs(const std::wstring& input)
{
    std::wstring out;
    out.reserve(input.size());

    bool pendingSpace = false;
    for (const wchar_t ch : input)
    {
        if (iswspace(ch))
        {
            pendingSpace = true;
            continue;
        }

        if (pendingSpace && !out.empty())
        {
            out.push_back(L' ');
        }

        pendingSpace = false;
        out.push_back(ch);
    }

    return out;
}

} // namespace

// Constructor
BopomofoScheme::BopomofoScheme() :
    m_dictionaryLoaded(false)
{
    InitializeBopomofoMapping();
}

// Destructor
BopomofoScheme::~BopomofoScheme()
{
}

// Initialize bopomofo scheme
bool BopomofoScheme::Initialize()
{
    return EnsureDictionaryLoaded();
}

// Process input
std::vector<InputScheme::Candidate> BopomofoScheme::ProcessInput(const std::wstring& input)
{
    return GetCandidates(input);
}

bool BopomofoScheme::EnsureDictionaryLoaded()
{
    if (m_dictionaryLoaded && m_dictionary)
    {
        return true;
    }

    m_dictionary = std::make_unique<Dictionary>();

    // Soft-config: allow overriding dictionary directory.
    // Example: set MAIDOS_IME_DICT_DIR=F:\MAIDOS_PORTABLE\dist
    std::vector<std::wstring> candidates;

    const std::wstring dictDir = GetEnvVarW(L"MAIDOS_IME_DICT_DIR");
    if (!dictDir.empty())
    {
        candidates.push_back(JoinPathW(dictDir, L"bopomofo.dict.json"));
        candidates.push_back(JoinPathW(dictDir, L"dicts\\bopomofo.dict.json"));
    }

    const std::wstring exeDir = GetExeDirW();
    if (!exeDir.empty())
    {
        candidates.push_back(JoinPathW(exeDir, L"bopomofo.dict.json"));
        candidates.push_back(JoinPathW(exeDir, L"dicts\\bopomofo.dict.json"));
        // When running from repo tree, the process dir may be ...\\src\\core; try walking up once.
        candidates.push_back(JoinPathW(exeDir, L"..\\dicts\\bopomofo.dict.json"));
    }

    // Repo-relative fallbacks.
    candidates.push_back(L"src/dicts/bopomofo.dict.json");
    candidates.push_back(L"dicts/bopomofo.dict.json");

    for (const auto& path : candidates)
    {
        if (!FileExistsW(path))
        {
            continue;
        }

        if (m_dictionary->LoadFromFile(path))
        {
            m_dictionaryLoaded = true;
            return true;
        }
    }

    m_dictionaryLoaded = false;
    return false;
}

std::wstring BopomofoScheme::NormalizeForLookup(const std::wstring& input) const
{
    return TrimAndCollapseWs(input);
}

// Get candidates
std::vector<InputScheme::Candidate> BopomofoScheme::GetCandidates(const std::wstring& input)
{
    std::vector<Candidate> candidates;

    if (!IsValidBopomofoInput(input))
    {
        return candidates;
    }

    if (!EnsureDictionaryLoaded())
    {
        return candidates;
    }

    const std::wstring key = NormalizeForLookup(input);
    auto entries = m_dictionary->Lookup(key);

    // Some callers may omit spaces; try a no-space match against the loaded dictionary keys.
    if (entries.empty())
    {
        const std::wstring needle = ParseBopomofoInput(key);
        if (!needle.empty())
        {
            const auto& all = m_dictionary->GetAllEntries();
            for (const auto& kv : all)
            {
                if (ParseBopomofoInput(kv.first) == needle)
                {
                    entries = kv.second;
                    break;
                }
            }
        }
    }

    for (const auto& entry : entries)
    {
        Candidate c;
        c.character = entry.word;
        c.frequency = static_cast<int>(entry.frequency);
        c.tags = entry.tags;

        const auto it = m_userWords.find(entry.word);
        if (it != m_userWords.end())
        {
            c.frequency += it->second;
        }

        candidates.push_back(std::move(c));
    }

    std::sort(candidates.begin(), candidates.end(),
        [](const Candidate& a, const Candidate& b) {
            return a.frequency > b.frequency;
        });

    if (candidates.size() > 10)
    {
        candidates.resize(10);
    }

    return candidates;
}

// Add word
void BopomofoScheme::AddWord(const std::wstring& word, int frequency)
{
    m_userWords[word] = frequency;
}

// Remove word
void BopomofoScheme::RemoveWord(const std::wstring& word)
{
    m_userWords.erase(word);
}

// Initialize bopomofo mapping using escape sequences to avoid encoding issues
void BopomofoScheme::InitializeBopomofoMapping()
{
    m_bopomofoToPinyin[L"\u3105"] = L"b";   // ㄅ
    m_bopomofoToPinyin[L"\u3106"] = L"p";   // ㄆ
    m_bopomofoToPinyin[L"\u3107"] = L"m";   // ㄇ
    m_bopomofoToPinyin[L"\u3108"] = L"f";   // ㄈ
    m_bopomofoToPinyin[L"\u3109"] = L"d";   // ㄉ
    m_bopomofoToPinyin[L"\u310A"] = L"t";   // ㄊ
    m_bopomofoToPinyin[L"\u310B"] = L"n";   // ㄋ
    m_bopomofoToPinyin[L"\u310C"] = L"l";   // ㄌ
    m_bopomofoToPinyin[L"\u310D"] = L"g";   // ㄍ
    m_bopomofoToPinyin[L"\u310E"] = L"k";   // ㄎ
    m_bopomofoToPinyin[L"\u310F"] = L"h";   // ㄏ
    m_bopomofoToPinyin[L"\u3110"] = L"j";   // ㄐ
    m_bopomofoToPinyin[L"\u3111"] = L"q";   // ㄑ
    m_bopomofoToPinyin[L"\u3112"] = L"x";   // ㄒ
    m_bopomofoToPinyin[L"\u3113"] = L"zh";  // ㄓ
    m_bopomofoToPinyin[L"\u3114"] = L"ch";  // ㄔ
    m_bopomofoToPinyin[L"\u3115"] = L"sh";  // ㄕ
    m_bopomofoToPinyin[L"\u3116"] = L"r";   // ㄖ
    m_bopomofoToPinyin[L"\u3117"] = L"z";   // ㄗ
    m_bopomofoToPinyin[L"\u3118"] = L"c";   // ㄘ
    m_bopomofoToPinyin[L"\u3119"] = L"s";   // ㄙ
    m_bopomofoToPinyin[L"\u3127"] = L"i";   // ㄧ
    m_bopomofoToPinyin[L"\u3128"] = L"u";   // ㄨ
    m_bopomofoToPinyin[L"\u3129"] = L"v";   // ㄩ
    m_bopomofoToPinyin[L"\u311A"] = L"a";   // ㄚ
    m_bopomofoToPinyin[L"\u311B"] = L"o";   // ㄛ
    m_bopomofoToPinyin[L"\u311C"] = L"e";   // ㄜ
    m_bopomofoToPinyin[L"\u311D"] = L"e";   // ㄝ
    m_bopomofoToPinyin[L"\u311E"] = L"ai";  // ㄞ
    m_bopomofoToPinyin[L"\u311F"] = L"ei";  // ㄟ
    m_bopomofoToPinyin[L"\u3120"] = L"ao";  // ㄠ
    m_bopomofoToPinyin[L"\u3121"] = L"ou";  // ㄡ
    m_bopomofoToPinyin[L"\u3122"] = L"an";  // ㄢ
    m_bopomofoToPinyin[L"\u3123"] = L"en";  // ㄣ
    m_bopomofoToPinyin[L"\u3124"] = L"ang"; // ㄤ
    m_bopomofoToPinyin[L"\u3125"] = L"eng"; // ㄥ
    m_bopomofoToPinyin[L"\u3126"] = L"er";  // ㄦ
    m_bopomofoToPinyin[L"\u02C7"] = L"3";   // ˇ
    m_bopomofoToPinyin[L"\u02CA"] = L"2";   // ˊ
    m_bopomofoToPinyin[L"\u02CB"] = L"4";   // ˋ
    m_bopomofoToPinyin[L"\u02D9"] = L"5";   // ˙
}

// Parse bopomofo input
std::wstring BopomofoScheme::ParseBopomofoInput(const std::wstring& input) const
{
    std::wstring result;
    for (wchar_t ch : input)
    {
        if (ch != L' ')
        {
            result += ch;
        }
    }
    return result;
}

// Convert bopomofo to pinyin
std::wstring BopomofoScheme::ConvertBopomofoToPinyin(const std::wstring& bopomofo) const
{
    std::wstring pinyin;
    for (size_t i = 0; i < bopomofo.length(); ++i) {
        wchar_t ch = bopomofo[i];
        wchar_t next_ch = (i + 1 < bopomofo.length()) ? bopomofo[i+1] : L'\0';
        
        // Special rules
        if (ch == L'\u3127' && next_ch == L'\u3122') { // ㄧㄢ
            pinyin += L"ian";
            i++; continue;
        }
        if (ch == L'\u3128' && next_ch == L'\u3122') { // ㄨㄢ
            pinyin += L"uan";
            i++; continue;
        }
        if (ch == L'\u3129' && next_ch == L'\u3122') { // ㄩㄢ
            pinyin += L"uan";
            i++; continue;
        }
        
        std::wstring bopomofoChar(1, ch);
        auto it = m_bopomofoToPinyin.find(bopomofoChar);
        if (it != m_bopomofoToPinyin.end()) {
            pinyin += it->second;
        } else {
            pinyin += bopomofoChar;
        }
    }
    return pinyin;
}

// Validate bopomofo input
bool BopomofoScheme::IsValidBopomofoInput(const std::wstring& input) const
{
    if (input.empty()) return false;
    for (wchar_t ch : input)
    {
        if (ch == L' ') continue;
        std::wstring bopomofoChar(1, ch);
        if (m_bopomofoToPinyin.find(bopomofoChar) != m_bopomofoToPinyin.end())
        {
            return true;
        }
    }
    return false;
}
