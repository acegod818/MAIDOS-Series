using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace MAIDOS.IME.AI
{
    /// <summary>
    /// AI管理器類
    /// </summary>
    public class AiManager
    {
        private readonly SmartPredict _smartPredict;
        private readonly VoiceInput _voiceInput;
        private readonly HandwritingInput _handwritingInput;

        /// <summary>
        /// 構造函數
        /// </summary>
        /// <param name="model">AI 模型名稱 (預設 local；由 SmartPredict 內部決定使用本地/外部推理)</param>
        /// <param name="whisperCliPath">Whisper CLI路徑</param>
        /// <param name="modelPath">模型路徑</param>
        public AiManager(
            string model = "local",
            string whisperCliPath = "whisper.exe",
            string modelPath = "models/ggml-base.en.bin")
        {
            _smartPredict = new SmartPredict(model);
            _voiceInput = new VoiceInput(whisperCliPath, modelPath);
            _handwritingInput = new HandwritingInput();
        }

        /// <summary>
        /// 選擇字符
        /// </summary>
        /// <param name="context">上下文</param>
        /// <param name="candidates">候選字列表</param>
        /// <returns>選中的字符</returns>
        public async Task<char> SelectCharacterAsync(string context, List<char> candidates)
        {
            return await _smartPredict.SelectCharacterAsync(context, candidates);
        }

        /// <summary>
        /// 自動糾錯
        /// </summary>
        /// <param name="text">需要糾錯的文本</param>
        /// <returns>糾錯後的文本</returns>
        public async Task<string> AutoCorrectAsync(string text)
        {
            return await _smartPredict.AutoCorrectAsync(text);
        }

        /// <summary>
        /// 智慧聯想
        /// </summary>
        /// <param name="text">需要聯想的文本</param>
        /// <returns>聯想建議列表</returns>
        public async Task<List<string>> SmartSuggestionsAsync(string text)
        {
            return await _smartPredict.SmartSuggestionsAsync(text);
        }

        /// <summary>
        /// 轉錄音頻文件
        /// </summary>
        /// <param name="audioFilePath">音頻文件路徑</param>
        /// <returns>轉錄文本</returns>
        public async Task<string> TranscribeAudioAsync(string audioFilePath)
        {
            return await _voiceInput.TranscribeAudioAsync(audioFilePath);
        }

        /// <summary>
        /// 記錄音頻並轉錄
        /// </summary>
        /// <param name="durationSeconds">錄音持續時間（秒）</param>
        /// <param name="outputFilePath">輸出音頻文件路徑</param>
        /// <returns>轉錄文本</returns>
        public async Task<string> RecordAndTranscribeAsync(int durationSeconds, string outputFilePath = "temp_recording.wav")
        {
            return await _voiceInput.RecordAndTranscribeAsync(durationSeconds, outputFilePath);
        }

        /// <summary>
        /// 識別手寫圖像
        /// </summary>
        /// <param name="imagePath">圖像文件路徑</param>
        /// <returns>識別結果</returns>
        public async Task<HandwritingInput.RecognitionResult> RecognizeHandwritingAsync(string imagePath)
        {
            return await _handwritingInput.RecognizeHandwritingAsync(imagePath);
        }

        /// <summary>
        /// 識別手寫圖像數據
        /// </summary>
        /// <param name="imageData">圖像數據</param>
        /// <returns>識別結果</returns>
        public async Task<HandwritingInput.RecognitionResult> RecognizeHandwritingAsync(byte[] imageData)
        {
            return await _handwritingInput.RecognizeHandwritingAsync(imageData);
        }

        /// <summary>
        /// 預處理圖像以提高識別準確性
        /// </summary>
        /// <param name="imagePath">圖像文件路徑</param>
        /// <returns>預處理後的圖像文件路徑</returns>
        public string PreprocessImage(string imagePath)
        {
            return _handwritingInput.PreprocessImage(imagePath);
        }
    }
}
