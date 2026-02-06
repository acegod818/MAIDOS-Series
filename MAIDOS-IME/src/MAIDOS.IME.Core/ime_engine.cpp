#include "pch.h"
#include "ime_engine.h"
#include <fstream>
#include <sstream>
#include <algorithm>
#include <random>

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

std::wstring ResolveDictPath(const wchar_t* fileName)
{
    // Soft-config: allow overriding dictionary directory.
    // Example: set MAIDOS_IME_DICT_DIR=F:\MAIDOS_PORTABLE\dist
    const std::wstring dictDir = GetEnvVarW(L"MAIDOS_IME_DICT_DIR");
    if (!dictDir.empty())
    {
        const std::wstring p1 = JoinPathW(dictDir, fileName);
        if (FileExistsW(p1)) return p1;
        const std::wstring p2 = JoinPathW(dictDir, JoinPathW(L"dicts", fileName));
        if (FileExistsW(p2)) return p2;
    }

    const std::wstring exeDir = GetExeDirW();
    if (!exeDir.empty())
    {
        const std::wstring p1 = JoinPathW(exeDir, fileName);
        if (FileExistsW(p1)) return p1;
        const std::wstring p2 = JoinPathW(exeDir, JoinPathW(L"dicts", fileName));
        if (FileExistsW(p2)) return p2;
        // When running from repo tree, the process dir may be ...\\src\\core; try walking up once.
        const std::wstring p3 = JoinPathW(exeDir, JoinPathW(L"..\\dicts", fileName));
        if (FileExistsW(p3)) return p3;
    }

    // Repo-relative fallbacks.
    const std::wstring p4 = JoinPathW(L"src\\dicts", fileName);
    if (FileExistsW(p4)) return p4;
    const std::wstring p5 = JoinPathW(L"dicts", fileName);
    if (FileExistsW(p5)) return p5;

    return L"";
}

} // namespace

// Constructor
ImeEngine::ImeEngine() :
    m_aiSelectionEnabled(false),
    m_autoCorrectionEnabled(false),
    m_smartSuggestionsEnabled(false),
    m_defaultScheme(L"pinyin"),
    m_charset(L"Traditional")
{
}

// Destructor
ImeEngine::~ImeEngine()
{
}

// Initialize engine
bool ImeEngine::Initialize(const std::wstring& configPath)
{
    try
    {
        // Load configuration
        LoadConfiguration(configPath);

        // Initialize dictionary
        m_dictionary = std::make_unique<Dictionary>();
        
        // Load dictionary from file
        std::wstring dictPath = ResolveDictPath(L"pinyin.dict.json");
        if (dictPath.empty() || !m_dictionary->LoadFromFile(dictPath))
        {
            // Fallback entries (ASCII placeholders for now to avoid encoding issues)
            m_dictionary->AddEntry(L"ni hao", Dictionary::DictEntry{ L"NiHao", 1000, L"ni hao", {L"greeting", L"common"} });
            m_dictionary->AddEntry(L"shi jie", Dictionary::DictEntry{ L"ShiJie", 800, L"shi jie", {L"noun", L"common"} });
            m_dictionary->AddEntry(L"xie xie", Dictionary::DictEntry{ L"XieXie", 950, L"xie xie", {L"greeting", L"common"} });
            m_dictionary->AddEntry(L"jin tian", Dictionary::DictEntry{ L"JinTian", 900, L"jin tian", {L"time", L"common"} });
            m_dictionary->AddEntry(L"ming tian", Dictionary::DictEntry{ L"MingTian", 700, L"ming tian", {L"time", L"common"} });
            m_dictionary->AddEntry(L"ai", Dictionary::DictEntry{ L"Ai", 600, L"ai", {L"emotion", L"common"} });
        }

        // Initialize pinyin parser
        m_pinyinParser = std::make_unique<PinyinParser>(*m_dictionary);

        // Initialize converter
        m_converter = std::make_unique<CharsetConverter>();

        // Initialize schemes
        auto pinyinScheme = std::make_unique<PinyinScheme>();
        pinyinScheme->SetParser(m_pinyinParser.get());
        m_schemes[L"pinyin"] = std::move(pinyinScheme);
        m_schemes[L"bopomofo"] = std::make_unique<BopomofoScheme>();

        return true;
    }
    catch (...)
    {
        return false;
    }
}

// Process input
std::vector<ImeEngine::Candidate> ImeEngine::ProcessInput(const std::wstring& input, const std::wstring& context)
{
    std::vector<Candidate> candidates = GetCandidatesFromScheme(input, m_defaultScheme);

    if (m_aiSelectionEnabled && !candidates.empty())
    {
        std::sort(candidates.begin(), candidates.end(), 
                  [](const Candidate& a, const Candidate& b) {
                      return a.frequency > b.frequency;
                  });
    }

    return candidates;
}

// Select character
wchar_t ImeEngine::SelectCharacter(const std::wstring& context, const std::vector<wchar_t>& candidates)
{
    if (candidates.empty())
        return L'\0';

    if (m_aiSelectionEnabled && candidates.size() > 1)
    {
        std::random_device rd;
        std::mt19937 gen(rd());
        std::uniform_int_distribution<> dis(0, static_cast<int>(candidates.size()) - 1);
        return candidates[dis(gen)];
    }

    return candidates[0];
}

// Auto correct
std::wstring ImeEngine::AutoCorrect(const std::wstring& text)
{
    if (!m_autoCorrectionEnabled)
        return text;

    return text;
}

// Smart suggestions
std::vector<std::wstring> ImeEngine::SmartSuggestions(const std::wstring& text)
{
    if (!m_smartSuggestionsEnabled)
        return {};

    std::vector<std::wstring> suggestions = {
        text + L",",
        text + L"!",
        text + L"?"
    };

    return suggestions;
}

// Process cross input
std::wstring ImeEngine::ProcessCrossInput(const std::wstring& input, const std::wstring& context,
    const std::wstring& scheme, const std::wstring& charset)
{
    std::vector<Candidate> candidates = ProcessInput(input, context);

    std::wstring result;
    if (!candidates.empty())
    {
        result = candidates[0].character;
    }
    else
    {
        result = input;
    }

    return m_converter->Convert(result, m_charset, charset);
}

// Get cross candidates
std::vector<ImeEngine::Candidate> ImeEngine::GetCrossCandidates(const std::wstring& input,
    const std::wstring& scheme, const std::wstring& charset)
{
    return GetCandidatesFromScheme(input, scheme);
}

// Load configuration
void ImeEngine::LoadConfiguration(const std::wstring& configPath)
{
    m_aiSelectionEnabled = true;
    m_autoCorrectionEnabled = true;
    m_smartSuggestionsEnabled = true;
    m_defaultScheme = L"pinyin";
    m_charset = L"Traditional";
}

// Get candidates from scheme
std::vector<ImeEngine::Candidate> ImeEngine::GetCandidatesFromScheme(const std::wstring& input, const std::wstring& schemeName)
{
    std::vector<Candidate> candidates;

    auto it = m_schemes.find(schemeName);
    if (it != m_schemes.end())
    {
        auto schemeCandidates = it->second->GetCandidates(input);
        
        for (const auto& candidate : schemeCandidates)
        {
            candidates.push_back({
                candidate.character,
                candidate.frequency,
                candidate.tags
                });
        }
    }
    else if (schemeName == L"pinyin")
    {
        auto result = m_pinyinParser->ParseContinuousPinyin(input);
        
        for (size_t i = 0; i < result.candidates.size() && i < result.frequencies.size(); ++i)
        {
            candidates.push_back({
                result.candidates[i],
                static_cast<int>(result.frequencies[i]),
                {}
                });
        }
    }

    return candidates;
}
