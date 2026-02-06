#pragma once

#include "pch.h"
#include <string>
#include <vector>
#include <map>
#include <memory>

// Dictionary class
class Dictionary {
public:
    // Dictionary entry structure
    struct DictEntry {
        std::wstring word;              // word
        unsigned int frequency;         // frequency
        std::wstring pronunciation;     // pronunciation
        std::vector<std::wstring> tags; // tags
    };

    // Constructor
    Dictionary();

    // Destructor
    ~Dictionary();

    // Load dictionary from file
    bool LoadFromFile(const std::wstring& filePath);

    // Save dictionary to file
    bool SaveToFile(const std::wstring& filePath) const;

    // Lookup entry
    std::vector<DictEntry> Lookup(const std::wstring& pronunciation) const;

    // Add entry
    void AddEntry(const std::wstring& pronunciation, const DictEntry& entry);

    // Get all entries
    const std::map<std::wstring, std::vector<DictEntry>>& GetAllEntries() const;

private:
    // Entry mapping (pronunciation -> entry list)
    std::map<std::wstring, std::vector<DictEntry>> m_entries;
    
    // Version information
    std::wstring m_version;
    std::wstring m_createdAt;
    std::wstring m_updatedAt;
};