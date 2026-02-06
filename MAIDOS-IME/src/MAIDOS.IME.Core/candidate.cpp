#include "pch.h"
#include "ime_engine.h"
#include "pinyin_parser.h"
#include <algorithm>
#include <iostream>

// Constructor
CandidateManager::CandidateManager(PinyinParser& parser) : m_parser(parser)
{
    std::wcout << L"[MAIDOS-AUDIT] CandidateManager initialized" << std::endl;
}

// Destructor
CandidateManager::~CandidateManager()
{
    std::wcout << L"[MAIDOS-AUDIT] CandidateManager destroyed" << std::endl;
}

// Get candidates for pinyin input
std::vector<std::wstring> CandidateManager::GetCandidates(const std::wstring& pinyinInput)
{
    std::wcout << L"[MAIDOS-AUDIT] Get candidates for pinyin: " << pinyinInput << std::endl;
    
    auto parseResult = m_parser.ParseContinuousPinyin(pinyinInput);
    
    std::vector<std::wstring> candidates;
    candidates.reserve(parseResult.candidates.size());
    
    for (const auto& candidate : parseResult.candidates)
    {
        candidates.push_back(candidate);
    }
    
    std::wcout << L"[MAIDOS-AUDIT] Retrieving " << candidates.size() << " candidates" << std::endl;
    return candidates;
}

// Get candidates with priority (AI-enhanced)
std::vector<std::wstring> CandidateManager::GetSmartCandidates(const std::wstring& pinyinInput, const std::wstring& context)
{
    std::wcout << L"[MAIDOS-AUDIT] Get smart candidates for: " << pinyinInput << " with context: " << context << std::endl;
    
    // First get standard candidates
    auto candidates = GetCandidates(pinyinInput);
    
    // If context is empty or candidates are few, return standard candidates
    if (context.empty() || candidates.size() <= 1)
    {
        std::wcout << L"[MAIDOS-AUDIT] Using standard candidates (no context or single candidate)" << std::endl;
        return candidates;
    }
    
    // Simple context-aware reordering based on common patterns
    // In a real implementation, this would use AI/ML for smarter ranking
    std::wcout << L"[MAIDOS-AUDIT] Applying context-aware reordering" << std::endl;
    
    // Simple pattern: prefer shorter words at sentence beginning, longer words elsewhere
    if (context.empty() || context.length() < 5)
    {
        // Beginning of sentence - prefer shorter words
        std::sort(candidates.begin(), candidates.end(),
                  [](const std::wstring& a, const std::wstring& b) {
                      return a.length() < b.length();
                  });
    }
    else
    {
        // Middle of sentence - prefer longer words
        std::sort(candidates.begin(), candidates.end(),
                  [](const std::wstring& a, const std::wstring& b) {
                      return a.length() > b.length();
                  });
    }
    
    // Limit to top 10 candidates for smart suggestions
    if (candidates.size() > 10)
    {
        candidates.resize(10);
    }
    
    std::wcout << L"[MAIDOS-AUDIT] Smart candidates generated: " << candidates.size() << std::endl;
    return candidates;
}

// Get candidate frequency
unsigned int CandidateManager::GetCandidateFrequency(const std::wstring& candidate)
{
    std::wcout << L"[MAIDOS-AUDIT] Get frequency for candidate: " << candidate << std::endl;
    
    // In a real implementation, this would query the dictionary or statistical model
    // For now, return a dummy frequency based on string length
    return static_cast<unsigned int>(1000 / (1 + candidate.length()));
}

// Select candidate by index
bool CandidateManager::SelectCandidate(int index, const std::vector<std::wstring>& candidates)
{
    std::wcout << L"[MAIDOS-AUDIT] Select candidate at index: " << index << std::endl;
    
    if (index < 0 || index >= static_cast<int>(candidates.size()))
    {
        std::wcout << L"[MAIDOS-AUDIT] Invalid candidate index" << std::endl;
        return false;
    }
    
    m_selectedCandidate = candidates[index];
    std::wcout << L"[MAIDOS-AUDIT] Selected candidate: " << m_selectedCandidate << std::endl;
    return true;
}

// Get selected candidate
std::wstring CandidateManager::GetSelectedCandidate() const
{
    return m_selectedCandidate;
}

// Clear selection
void CandidateManager::ClearSelection()
{
    std::wcout << L"[MAIDOS-AUDIT] Clearing candidate selection" << std::endl;
    m_selectedCandidate.clear();
}

// Check if selection is valid
bool CandidateManager::HasValidSelection() const
{
    return !m_selectedCandidate.empty();
}

// Reset candidate state
void CandidateManager::Reset()
{
    std::wcout << L"[MAIDOS-AUDIT] Resetting CandidateManager" << std::endl;
    ClearSelection();
    m_parser.ClearCache();
}

// Add user preference for candidate
void CandidateManager::AddUserPreference(const std::wstring& pinyin, const std::wstring& candidate, int preferenceBoost)
{
    std::wcout << L"[MAIDOS-AUDIT] Adding user preference: " << pinyin << " -> " << candidate 
               << " (boost: " << preferenceBoost << ")" << std::endl;
    
    // In a real implementation, this would update the user dictionary or preference database
    // For now, just log the preference
    m_userPreferences[pinyin][candidate] += preferenceBoost;
}

// Get candidate suggestions based on usage history
std::vector<std::wstring> CandidateManager::GetSmartSuggestions(const std::wstring& pinyinInput)
{
    std::wcout << L"[MAIDOS-AUDIT] Getting smart suggestions for: " << pinyinInput << std::endl;
    
    auto candidates = GetCandidates(pinyinInput);
    
    // Apply user preferences if available
    auto preferenceIt = m_userPreferences.find(pinyinInput);
    if (preferenceIt != m_userPreferences.end())
    {
        std::wcout << L"[MAIDOS-AUDIT] Applying user preferences" << std::endl;
        
        // Create a weighted list based on user preferences
        std::vector<std::pair<std::wstring, int>> weightedCandidates;
        for (const auto& candidate : candidates)
        {
            int weight = 0;
            auto candidatePreference = preferenceIt->second.find(candidate);
            if (candidatePreference != preferenceIt->second.end())
            {
                weight = candidatePreference->second;
            }
            weightedCandidates.emplace_back(candidate, weight);
        }
        
        // Sort by weight (descending)
        std::sort(weightedCandidates.begin(), weightedCandidates.end(),
                  [](const auto& a, const auto& b) {
                      return a.second > b.second;
                  });
        
        // Extract just the candidate strings
        candidates.clear();
        for (const auto& weightedCandidate : weightedCandidates)
        {
            candidates.push_back(weightedCandidate.first);
        }
    }
    
    return candidates;
}