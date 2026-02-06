#include "pch.h"
#include "dictionary.h"
#include <fstream>
#include <sstream>
#include <iomanip>
#include <ctime>
#include <cwctype>

namespace {

void SkipWs(const std::wstring& s, size_t& i)
{
    while (i < s.size() && iswspace(s[i]))
    {
        ++i;
    }
}

bool Consume(const std::wstring& s, size_t& i, wchar_t ch)
{
    SkipWs(s, i);
    if (i >= s.size() || s[i] != ch)
    {
        return false;
    }
    ++i;
    return true;
}

bool ParseString(const std::wstring& s, size_t& i, std::wstring& out)
{
    SkipWs(s, i);
    if (i >= s.size() || s[i] != L'"')
    {
        return false;
    }

    ++i;
    std::wstring result;

    while (i < s.size())
    {
        wchar_t ch = s[i++];
        if (ch == L'"')
        {
            out = std::move(result);
            return true;
        }

        if (ch == L'\\')
        {
            if (i >= s.size())
            {
                return false;
            }

            wchar_t esc = s[i++];
            switch (esc)
            {
            case L'"': result.push_back(L'"'); break;
            case L'\\': result.push_back(L'\\'); break;
            case L'/': result.push_back(L'/'); break;
            case L'b': result.push_back(L'\b'); break;
            case L'f': result.push_back(L'\f'); break;
            case L'n': result.push_back(L'\n'); break;
            case L'r': result.push_back(L'\r'); break;
            case L't': result.push_back(L'\t'); break;
            case L'u':
            {
                // Minimal \uXXXX support.
                if (i + 4 > s.size())
                {
                    return false;
                }

                unsigned int code = 0;
                for (int k = 0; k < 4; k++)
                {
                    wchar_t h = s[i++];
                    code <<= 4;

                    if (h >= L'0' && h <= L'9')
                    {
                        code |= static_cast<unsigned int>(h - L'0');
                    }
                    else if (h >= L'a' && h <= L'f')
                    {
                        code |= static_cast<unsigned int>(h - L'a' + 10);
                    }
                    else if (h >= L'A' && h <= L'F')
                    {
                        code |= static_cast<unsigned int>(h - L'A' + 10);
                    }
                    else
                    {
                        return false;
                    }
                }

                result.push_back(static_cast<wchar_t>(code));
                break;
            }
            default:
                // Unknown escape sequence; keep best-effort.
                result.push_back(esc);
                break;
            }

            continue;
        }

        result.push_back(ch);
    }

    return false;
}

bool ParseUnsigned(const std::wstring& s, size_t& i, unsigned int& out)
{
    SkipWs(s, i);
    if (i >= s.size() || !iswdigit(s[i]))
    {
        return false;
    }

    unsigned long long value = 0;
    while (i < s.size() && iswdigit(s[i]))
    {
        value = (value * 10ULL) + static_cast<unsigned long long>(s[i] - L'0');
        if (value > 0xFFFFFFFFULL)
        {
            value = 0xFFFFFFFFULL;
        }
        ++i;
    }

    out = static_cast<unsigned int>(value);
    return true;
}

bool SkipValue(const std::wstring& s, size_t& i);

bool SkipObject(const std::wstring& s, size_t& i)
{
    if (!Consume(s, i, L'{'))
    {
        return false;
    }

    while (i < s.size())
    {
        SkipWs(s, i);
        if (i >= s.size())
        {
            return false;
        }

        if (s[i] == L'}')
        {
            ++i;
            return true;
        }

        std::wstring key;
        if (!ParseString(s, i, key))
        {
            return false;
        }
        if (!Consume(s, i, L':'))
        {
            return false;
        }
        if (!SkipValue(s, i))
        {
            return false;
        }

        SkipWs(s, i);
        if (i < s.size() && s[i] == L',')
        {
            ++i;
        }
    }

    return false;
}

bool SkipArray(const std::wstring& s, size_t& i)
{
    if (!Consume(s, i, L'['))
    {
        return false;
    }

    while (i < s.size())
    {
        SkipWs(s, i);
        if (i >= s.size())
        {
            return false;
        }

        if (s[i] == L']')
        {
            ++i;
            return true;
        }

        if (!SkipValue(s, i))
        {
            return false;
        }

        SkipWs(s, i);
        if (i < s.size() && s[i] == L',')
        {
            ++i;
        }
    }

    return false;
}

bool SkipValue(const std::wstring& s, size_t& i)
{
    SkipWs(s, i);
    if (i >= s.size())
    {
        return false;
    }

    if (s[i] == L'"')
    {
        std::wstring tmp;
        return ParseString(s, i, tmp);
    }

    if (s[i] == L'{')
    {
        return SkipObject(s, i);
    }

    if (s[i] == L'[')
    {
        return SkipArray(s, i);
    }

    // number / true / false / null
    size_t start = i;
    while (i < s.size())
    {
        wchar_t ch = s[i];
        if (ch == L',' || ch == L'}' || ch == L']' || iswspace(ch))
        {
            break;
        }
        ++i;
    }

    return i > start;
}

bool ParseTags(const std::wstring& s, size_t& i, std::vector<std::wstring>& tags)
{
    tags.clear();
    if (!Consume(s, i, L'['))
    {
        return false;
    }

    while (i < s.size())
    {
        SkipWs(s, i);
        if (i >= s.size())
        {
            return false;
        }

        if (s[i] == L']')
        {
            ++i;
            return true;
        }

        std::wstring tag;
        if (!ParseString(s, i, tag))
        {
            return false;
        }
        tags.push_back(std::move(tag));

        SkipWs(s, i);
        if (i < s.size() && s[i] == L',')
        {
            ++i;
        }
    }

    return false;
}

bool ParseEntryObject(const std::wstring& s, size_t& i, Dictionary::DictEntry& entry)
{
    entry = Dictionary::DictEntry{};

    if (!Consume(s, i, L'{'))
    {
        return false;
    }

    while (i < s.size())
    {
        SkipWs(s, i);
        if (i >= s.size())
        {
            return false;
        }

        if (s[i] == L'}')
        {
            ++i;
            return true;
        }

        std::wstring key;
        if (!ParseString(s, i, key))
        {
            return false;
        }
        if (!Consume(s, i, L':'))
        {
            return false;
        }

        if (key == L"word")
        {
            if (!ParseString(s, i, entry.word))
            {
                return false;
            }
        }
        else if (key == L"frequency")
        {
            if (!ParseUnsigned(s, i, entry.frequency))
            {
                return false;
            }
        }
        else if (key == L"pronunciation")
        {
            if (!ParseString(s, i, entry.pronunciation))
            {
                return false;
            }
        }
        else if (key == L"tags")
        {
            if (!ParseTags(s, i, entry.tags))
            {
                return false;
            }
        }
        else
        {
            if (!SkipValue(s, i))
            {
                return false;
            }
        }

        SkipWs(s, i);
        if (i < s.size() && s[i] == L',')
        {
            ++i;
        }
    }

    return false;
}

} // namespace

// Constructor
Dictionary::Dictionary() :
    m_version(L"1.0.0"),
    m_createdAt(L"2026-01-25T00:00:00Z"),
    m_updatedAt(L"2026-01-25T00:00:00Z")
{
}

// Destructor
Dictionary::~Dictionary()
{
}

// Load dictionary from file
bool Dictionary::LoadFromFile(const std::wstring& filePath)
{
    try
    {
        m_entries.clear();

        std::wifstream file(filePath);
        if (!file.is_open())
        {
            return false;
        }

        // JSON dictionaries are UTF-8.
        file.imbue(std::locale(std::locale(), new std::codecvt_utf8_utf16<wchar_t>));

        std::wstringstream buffer;
        buffer << file.rdbuf();
        std::wstring content = buffer.str();
        file.close();

        size_t pos = content.find(L"\"entries\"");
        if (pos == std::wstring::npos)
        {
            return false;
        }

        pos = content.find(L':', pos);
        if (pos == std::wstring::npos)
        {
            return false;
        }
        ++pos;

        if (!Consume(content, pos, L'{'))
        {
            return false;
        }

        while (pos < content.size())
        {
            SkipWs(content, pos);
            if (pos >= content.size())
            {
                return false;
            }

            if (content[pos] == L'}')
            {
                ++pos;
                break;
            }

            std::wstring pronKey;
            if (!ParseString(content, pos, pronKey))
            {
                return false;
            }

            if (!Consume(content, pos, L':'))
            {
                return false;
            }

            if (!Consume(content, pos, L'['))
            {
                return false;
            }

            SkipWs(content, pos);
            if (pos < content.size() && content[pos] != L']')
            {
                while (pos < content.size())
                {
                    DictEntry entry;
                    if (!ParseEntryObject(content, pos, entry))
                    {
                        return false;
                    }

                    if (entry.pronunciation.empty())
                    {
                        entry.pronunciation = pronKey;
                    }

                    m_entries[pronKey].push_back(std::move(entry));

                    SkipWs(content, pos);
                    if (pos < content.size() && content[pos] == L',')
                    {
                        ++pos;
                        continue;
                    }
                    break;
                }
            }

            if (!Consume(content, pos, L']'))
            {
                return false;
            }

            SkipWs(content, pos);
            if (pos < content.size() && content[pos] == L',')
            {
                ++pos;
            }
        }

        // Update timestamp.
        std::time_t now = std::time(nullptr);
        std::tm tm_buf{};
        localtime_s(&tm_buf, &now);

        std::wstringstream ss;
        ss << std::put_time(&tm_buf, L"%Y-%m-%dT%H:%M:%SZ");
        m_updatedAt = ss.str();

        // If nothing loaded, let caller fallback.
        return !m_entries.empty();
    }
    catch (...)
    {
        return false;
    }
}

// Save dictionary to file
bool Dictionary::SaveToFile(const std::wstring& filePath) const
{
    try
    {
        std::wofstream file(filePath);
        if (!file.is_open())
        {
            return false;
        }

        file << L"{" << std::endl;
        file << L"  \"version\": \"" << m_version << L"\"," << std::endl;
        file << L"  \"created_at\": \"" << m_createdAt << L"\"," << std::endl;
        file << L"  \"updated_at\": \"" << m_updatedAt << L"\"," << std::endl;
        file << L"  \"entries\": {" << std::endl;

        bool firstEntry = true;
        for (const auto& pair : m_entries)
        {
            if (!firstEntry)
            {
                file << L"," << std::endl;
            }
            firstEntry = false;

            file << L"    \"" << pair.first << L"\": [" << std::endl;

            bool firstWord = true;
            for (const auto& entry : pair.second)
            {
                if (!firstWord)
                {
                    file << L"," << std::endl;
                }
                firstWord = false;

                file << L"      {" << std::endl;
                file << L"        \"word\": \"" << entry.word << L"\"," << std::endl;
                file << L"        \"frequency\": " << entry.frequency << L"," << std::endl;
                file << L"        \"pronunciation\": \"" << entry.pronunciation << L"\"," << std::endl;
                file << L"        \"tags\": [" << std::endl;

                bool firstTag = true;
                for (const auto& tag : entry.tags)
                {
                    if (!firstTag)
                    {
                        file << L"," << std::endl;
                    }
                    firstTag = false;
                    file << L"          \"" << tag << L"\"";
                }
                file << std::endl << L"        ]" << std::endl;
                file << L"      }";
            }
            file << std::endl << L"    ]";
        }

        file << std::endl << L"  }" << std::endl;
        file << L"}" << std::endl;

        file.close();
        return true;
    }
    catch (...)
    {
        return false;
    }
}

// Lookup entry
std::vector<Dictionary::DictEntry> Dictionary::Lookup(const std::wstring& pronunciation) const
{
    auto it = m_entries.find(pronunciation);
    if (it != m_entries.end())
    {
        return it->second;
    }
    return {};
}

// Add entry
void Dictionary::AddEntry(const std::wstring& pronunciation, const DictEntry& entry)
{
    m_entries[pronunciation].push_back(entry);
}

// Get all entries
const std::map<std::wstring, std::vector<Dictionary::DictEntry>>& Dictionary::GetAllEntries() const
{
    return m_entries;
}
