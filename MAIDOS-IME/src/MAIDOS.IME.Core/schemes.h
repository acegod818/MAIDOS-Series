#pragma once

#include "pch.h"
#include <string>
#include <vector>
#include <memory>
#include <map>

class PinyinParser;

// Base class for input schemes
class InputScheme {
public:
    // Candidate structure
    struct Candidate {
        std::wstring character;
        int frequency;
        std::vector<std::wstring> tags;
    };

    // Virtual destructor
    virtual ~InputScheme() = default;

    // Process input
    virtual std::vector<Candidate> ProcessInput(const std::wstring& input) = 0;

    // Get candidates
    virtual std::vector<Candidate> GetCandidates(const std::wstring& input) = 0;

    // Add word
    virtual void AddWord(const std::wstring& word, int frequency = 0) = 0;

    // Remove word
    virtual void RemoveWord(const std::wstring& word) = 0;
};

// Pinyin input scheme
class PinyinScheme : public InputScheme {
public:
    PinyinScheme() : m_parser(nullptr) {}

    // Set the PinyinParser to delegate candidate lookup
    void SetParser(PinyinParser* parser) { m_parser = parser; }

    // Process input
    std::vector<Candidate> ProcessInput(const std::wstring& input) override;

    // Get candidates
    std::vector<Candidate> GetCandidates(const std::wstring& input) override;

    // Add word
    void AddWord(const std::wstring& word, int frequency = 0) override;

    // Remove word
    void RemoveWord(const std::wstring& word) override;

private:
    PinyinParser* m_parser;
    std::map<std::wstring, int> m_userWords;
};

// Cangjie input scheme
class CangjieScheme : public InputScheme {
public:
    // Process input
    std::vector<Candidate> ProcessInput(const std::wstring& input) override;

    // Get candidates
    std::vector<Candidate> GetCandidates(const std::wstring& input) override;

    // Add word
    void AddWord(const std::wstring& word, int frequency = 0) override;

    // Remove word
    void RemoveWord(const std::wstring& word) override;

private:
    std::map<std::wstring, int> m_userWords;
};

// Scheme factory
class SchemeFactory {
public:
    // Create scheme
    static std::unique_ptr<InputScheme> CreateScheme(const std::wstring& schemeName);
};