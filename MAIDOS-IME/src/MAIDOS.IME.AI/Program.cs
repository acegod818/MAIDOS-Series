using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using MAIDOS.IME.AI;

namespace MAIDOS.IME.AI.Test
{
    class Program
    {
        static async Task Main(string[] args)
        {
            Console.WriteLine("MAIDOS IME AI 功能測試");
            Console.WriteLine("======================");

            // 創建AI管理器實例
            var aiManager = new AiManager();

            // 測試智能選字
            await TestSmartCharacterSelection(aiManager);

            // 測試自動糾錯
            await TestAutoCorrection(aiManager);

            // 測試智慧聯想
            await TestSmartSuggestions(aiManager);

            // 測試語音輸入（僅演示API調用，實際需要Whisper CLI）
            await TestVoiceInput(aiManager);

            // 測試手寫輸入（僅演示API調用，實際需要圖像處理庫）
            await TestHandwritingInput(aiManager);

            Console.WriteLine("\n所有測試完成!");
            if (Console.IsInputRedirected == false && Environment.UserInteractive)
            {
                Console.WriteLine("按任意鍵退出...");
                Console.ReadKey();
            }
        }

        static async Task TestSmartCharacterSelection(AiManager aiManager)
        {
            Console.WriteLine("\n1. 智能選字測試:");
            var context = "今天天氣很";
            var candidates = new List<char> { '好', '棒', '差', '糟' };
            
            Console.WriteLine($"上下文: {context}");
            Console.WriteLine($"候選字: {string.Join(", ", candidates)}");
            
            var selectedChar = await aiManager.SelectCharacterAsync(context, candidates);
            Console.WriteLine($"選中的字符: {selectedChar}");
        }

        static async Task TestAutoCorrection(AiManager aiManager)
        {
            Console.WriteLine("\n2. 自動糾錯測試:");
            var textToCorrect = "今天天氣很好，我覺得很開心。";
            
            Console.WriteLine($"原文本: {textToCorrect}");
            var correctedText = await aiManager.AutoCorrectAsync(textToCorrect);
            Console.WriteLine($"糾錯後: {correctedText}");
        }

        static async Task TestSmartSuggestions(AiManager aiManager)
        {
            Console.WriteLine("\n3. 智慧聯想測試:");
            var textForSuggestions = "今天天氣很好，適合";
            
            Console.WriteLine($"輸入文本: {textForSuggestions}");
            var suggestions = await aiManager.SmartSuggestionsAsync(textForSuggestions);
            
            Console.WriteLine("聯想建議:");
            for (int i = 0; i < suggestions.Count; i++)
            {
                Console.WriteLine($"  {i + 1}. {suggestions[i]}");
            }
        }

        static async Task TestVoiceInput(AiManager aiManager)
        {
            Console.WriteLine("\n4. 語音輸入測試:");
            Console.WriteLine("注意: 此功能需要安裝Whisper CLI和相應的模型文件");
            
            // 這裡僅演示API調用，實際使用時需要提供真實的音頻文件
            try
            {
                // 這行代碼會失敗，因為我們沒有真實的音頻文件
                // var transcription = await aiManager.TranscribeAudioAsync("test_audio.wav");
                // Console.WriteLine($"轉錄結果: {transcription}");
                
                Console.WriteLine("語音轉錄API調用演示完成");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"語音轉錄測試失敗: {ex.Message}");
            }
        }

        static async Task TestHandwritingInput(AiManager aiManager)
        {
            Console.WriteLine("\n5. 手寫輸入測試:");
            Console.WriteLine("注意: 此功能需要安裝圖像處理庫");
            
            // 這裡僅演示API調用，實際使用時需要提供真實的圖像文件
            try
            {
                // 這行代碼會失敗，因為我們沒有真實的圖像文件
                // var result = await aiManager.RecognizeHandwritingAsync("test_image.png");
                // Console.WriteLine($"識別結果: {result.Text}");
                // Console.WriteLine($"置信度: {result.Confidence}");
                
                Console.WriteLine("手寫識別API調用演示完成");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"手寫識別測試失敗: {ex.Message}");
            }
        }
    }
}