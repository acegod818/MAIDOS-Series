#include "pch.h"
#include "ime_engine.h"
#include <iostream>
#include <string>

int main()
{
    // Instance of IME Engine
    ImeEngine engine;

    // Initialize
    std::wstring configPath = L"../../src/config/maidos.toml";
    if (!engine.Initialize(configPath))
    {
        std::wcerr << L"Failed to initialize engine" << std::endl;
        return 1;
    }

    std::wcout << L"MAIDOS IME Core Engine Test" << std::endl;
    std::wcout << L"============================" << std::endl;

    // Test pinyin
    std::wstring input = L"nihao";
    std::wcout << L"Input: " << input << std::endl;

    auto candidates = engine.ProcessInput(input);

    std::wcout << L"Candidates:" << std::endl;
    for (size_t i = 0; i < candidates.size(); ++i)
    {
        std::wcout << L"  " << (i + 1) << L". " << candidates[i].character 
                  << L" (Frequency: " << candidates[i].frequency << L")" << std::endl;
    }

    // Auto correct
    std::wstring textToCorrect = L"hello";
    std::wcout << L"\nAuto Correct Test:" << std::endl;
    std::wcout << L"Original: " << textToCorrect << std::endl;
    std::wstring correctedText = engine.AutoCorrect(textToCorrect);
    std::wcout << L"Corrected: " << correctedText << std::endl;

    // Smart suggestions
    std::wstring textForSuggestions = L"how are";
    std::wcout << L"\nSmart Suggestions Test:" << std::endl;
    std::wcout << L"Input: " << textForSuggestions << std::endl;
    auto suggestions = engine.SmartSuggestions(textForSuggestions);
    std::wcout << L"Suggestions:" << std::endl;
    for (size_t i = 0; i < suggestions.size(); ++i)
    {
        std::wcout << L"  " << (i + 1) << L". " << suggestions[i] << std::endl;
    }

    std::wcout << L"\nTest Completed!" << std::endl;

    return 0;
}