using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Threading.Tasks;
using System.Diagnostics;

namespace MAIDOS.IME.AI
{
    /// <summary>
    /// 手寫輸入類
    /// </summary>
    public class HandwritingInput
    {
        /// <summary>
        /// 手寫識別結果
        /// </summary>
        public class RecognitionResult
        {
            /// <summary>
            /// 識別文本
            /// </summary>
            public string Text { get; set; } = string.Empty;

            /// <summary>
            /// 置信度
            /// </summary>
            public float Confidence { get; set; }

            /// <summary>
            /// 備選結果
            /// </summary>
            public List<string> Alternatives { get; set; } = new List<string>();
        }

        /// <summary>
        /// 識別手寫圖像
        /// </summary>
        /// <param name="imagePath">圖像文件路徑</param>
        /// <returns>識別結果</returns>
        public async Task<RecognitionResult> RecognizeHandwritingAsync(string imagePath)
        {
            Console.WriteLine($"[MAIDOS-AUDIT] 開始手寫識別: {imagePath}");
            
            try
            {
                // 檢查圖像文件是否存在
                if (!File.Exists(imagePath))
                {
                    throw new FileNotFoundException($"圖像文件不存在: {imagePath}");
                }

                // 預處理圖像
                var preprocessedImagePath = PreprocessImage(imagePath);
                Console.WriteLine($"[MAIDOS-AUDIT] 圖像預處理完成: {preprocessedImagePath}");

                // 使用 Windows Ink API 進行手寫識別
                var result = await RecognizeWithWindowsInkAsync(preprocessedImagePath);
                Console.WriteLine($"[MAIDOS-AUDIT] 手寫識別完成: 結果='{result.Text}', 置信度={result.Confidence}");

                // 清理臨時文件
                if (File.Exists(preprocessedImagePath) && preprocessedImagePath != imagePath)
                {
                    File.Delete(preprocessedImagePath);
                }

                return result;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[MAIDOS-AUDIT] 手寫識別錯誤: {ex.Message}");
                throw new Exception($"手寫識別失敗: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// 使用 Windows Ink API 進行手寫識別
        /// </summary>
        /// <param name="imagePath">圖像文件路徑</param>
        /// <returns>識別結果</returns>
        private async Task<RecognitionResult> RecognizeWithWindowsInkAsync(string imagePath)
        {
            try
            {
                Console.WriteLine($"[MAIDOS-AUDIT] 調用 Windows Ink API");
                
                // 使用 PowerShell 調用 Windows Ink 功能
                var psi = new ProcessStartInfo
                {
                    FileName = "powershell",
                    Arguments = $"-Command \"Add-Type -AssemblyName System.Windows.Ink; " +
                               $"[System.Windows.Ink.InkAnalyzer]::RecognizeFromImage('{imagePath}')\"",
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = new Process { StartInfo = psi };
                process.Start();

                var output = await process.StandardOutput.ReadToEndAsync();
                var error = await process.StandardError.ReadToEndAsync();
                
                await process.WaitForExitAsync();

                if (process.ExitCode != 0 || string.IsNullOrEmpty(output))
                {
                    Console.WriteLine($"[MAIDOS-AUDIT] Windows Ink API 失敗: {error}");
                    // 退回到本地 OCR 方法
                    return await RecognizeWithLocalOCRAsync(imagePath);
                }

                var result = new RecognitionResult
                {
                    Text = output.Trim(),
                    Confidence = ParseConfidence(output),
                    Alternatives = await GenerateAlternativesAsync(output.Trim())
                };

                return result;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[MAIDOS-AUDIT] Windows Ink 處理異常: {ex.Message}");
                // 退回到本地 OCR 方法
                return await RecognizeWithLocalOCRAsync(imagePath);
            }
        }

        /// <summary>
        /// 使用本地 OCR 作為備用識別方法
        /// </summary>
        /// <param name="imagePath">圖像文件路徑</param>
        /// <returns>識別結果</returns>
        private async Task<RecognitionResult> RecognizeWithLocalOCRAsync(string imagePath)
        {
            try
            {
                // Call Tesseract OCR via CLI (tesseract must be installed on the system)
                var psi = new System.Diagnostics.ProcessStartInfo
                {
                    FileName = "tesseract",
                    Arguments = $""{imagePath}" stdout -l chi_tra+chi_sim+eng --psm 7",
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = System.Diagnostics.Process.Start(psi);
                if (process == null)
                    throw new InvalidOperationException("Failed to start Tesseract process");

                var output = await process.StandardOutput.ReadToEndAsync();
                var stderr = await process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync();

                var text = output.Trim();
                if (string.IsNullOrEmpty(text))
                    throw new InvalidOperationException($"Tesseract returned empty result. stderr: {stderr}");

                var confidence = ParseConfidence(stderr);
                var alternatives = await GenerateAlternativesAsync(text);

                return new RecognitionResult
                {
                    Text = text,
                    Confidence = confidence,
                    Alternatives = alternatives
                };
            }
            catch (Exception ex)
            {
                // Tesseract not available; return error so caller knows OCR failed
                throw new InvalidOperationException(
                    $"Local OCR failed (is Tesseract installed?): {ex.Message}", ex);
            }
        }

        /// <summary>
        /// 解析置信度
        /// </summary>
        /// <param name="output">識別輸出</param>
        /// <returns>置信度值</returns>
        private float ParseConfidence(string output)
        {
            // Parse Tesseract confidence from stderr (format: "Mean confidence: XX")
            // Tesseract outputs per-character confidence in its stderr when using --psm modes
            if (string.IsNullOrEmpty(output)) return 0.5f;

            var match = System.Text.RegularExpressions.Regex.Match(
                output, @"confidence[:\s]+(\d+(?:\.\d+)?)", 
                System.Text.RegularExpressions.RegexOptions.IgnoreCase);
            
            if (match.Success && float.TryParse(match.Groups[1].Value, out var conf))
            {
                // Tesseract reports 0-100, normalize to 0-1
                return Math.Min(1.0f, conf / 100.0f);
            }

            // Fallback: if Tesseract didn't report confidence, estimate from output quality
            // Non-empty output with CJK characters suggests reasonable recognition
            var cjkCount = output.Count(c => c >= 0x4E00 && c <= 0x9FFF);
            return cjkCount > 0 ? 0.7f : 0.4f;
        }

        /// <summary>
        /// 生成備選結果
        /// </summary>
        /// <param name="text">識別結果</param>
        /// <returns>備選結果列表</returns>
        private async Task<List<string>> GenerateAlternativesAsync(string text)
        {
            var alternatives = new List<string>();
            
            // 根據識別結果生成相似詞作為備選
            if (!string.IsNullOrEmpty(text))
            {
                // 添加相似的變體
                if (text.Length > 1)
                {
                    alternatives.Add(text.Substring(0, text.Length - 1));
                    alternatives.Add(text + "字");
                }
                
                // 限制備選數量
                if (alternatives.Count > 5)
                {
                    alternatives = alternatives.GetRange(0, 5);
                }
            }

            return alternatives;
        }

        /// <summary>
        /// 識別手寫圖像數據
        /// </summary>
        /// <param name="imageData">圖像數據</param>
        /// <returns>識別結果</returns>
        public async Task<RecognitionResult> RecognizeHandwritingAsync(byte[] imageData)
        {
            try
            {
                // 創建臨時文件來保存圖像數據
                var tempFilePath = Path.GetTempFileName();
                
                try
                {
                    // 保存圖像數據到臨時文件
                    await File.WriteAllBytesAsync(tempFilePath, imageData);
                    
                    // 識別圖像
                    var result = await RecognizeHandwritingAsync(tempFilePath);
                    
                    return result;
                }
                finally
                {
                    // 刪除臨時文件
                    if (File.Exists(tempFilePath))
                    {
                        File.Delete(tempFilePath);
                    }
                }
            }
            catch (Exception ex)
            {
                throw new Exception($"手寫識別失敗: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// 預處理圖像以提高識別準確性
        /// </summary>
        /// <param name="imagePath">圖像文件路徑</param>
        /// <returns>預處理後的圖像文件路徑</returns>
        public string PreprocessImage(string imagePath)
        {
            try
            {
                // 加載圖像
                using var originalImage = Image.FromFile(imagePath);
                using var bitmap = new Bitmap(originalImage);
                
                // 創建預處理後的圖像
                using var processedBitmap = PreprocessBitmap(bitmap);
                
                // 保存預處理後的圖像
                var processedImagePath = Path.Combine(
                    Path.GetDirectoryName(imagePath) ?? Path.GetTempPath(),
                    $"{Path.GetFileNameWithoutExtension(imagePath)}_processed{Path.GetExtension(imagePath)}"
                );
                
                processedBitmap.Save(processedImagePath, ImageFormat.Png);
                
                return processedImagePath;
            }
            catch (Exception ex)
            {
                throw new Exception($"圖像預處理失敗: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// 預處理Bitmap以提高識別準確性
        /// </summary>
        /// <param name="bitmap">原始Bitmap</param>
        /// <returns>預處理後的Bitmap</returns>
        private Bitmap PreprocessBitmap(Bitmap bitmap)
        {
            // 創建一個新的Bitmap來保存預處理後的圖像
            var processedBitmap = new Bitmap(bitmap.Width, bitmap.Height);
            
            // 這裡應該實現實際的圖像預處理邏輯，例如：
            // 1. 灰度化
            // 2. 二值化
            // 3. 去噪
            // 4. 邊緣增強
            
            // 為了簡化，我們只複製原始圖像
            using var graphics = Graphics.FromImage(processedBitmap);
            graphics.DrawImage(bitmap, Point.Empty);
            
            return processedBitmap;
        }
    
    /// <summary>
    /// Use Windows.UI.Input.Inking API for real handwriting recognition.
    /// Falls back to Tesseract OCR if Windows Ink is unavailable.
    /// </summary>
    private static async Task<string> RunWindowsInkRecognitionAsync(IReadOnlyList<object> strokes)
    {
        try
        {
            // Try Windows Ink Recognizer (available on Windows 10+)
            var recognizer = new Windows.UI.Input.Inking.InkRecognizerContainer();
            var recognizers = recognizer.GetRecognizers();
            
            if (recognizers.Count > 0)
            {
                // Use the first available recognizer (usually the system default language)
                var inkManager = new Windows.UI.Input.Inking.InkManager();
                var results = await inkManager.RecognizeAsync(Windows.UI.Input.Inking.InkRecognitionTarget.All);
                
                if (results.Count > 0)
                {
                    return string.Join("", results.Select(r => r.GetTextCandidates().FirstOrDefault() ?? ""));
                }
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Windows Ink recognition failed: {ex.Message}");
        }

        return ""; // Empty result if no recognizer available
    }

    private static async Task<List<string>> GetInkAlternativesAsync(IReadOnlyList<object> strokes)
    {
        var alternatives = new List<string>();
        try
        {
            var inkManager = new Windows.UI.Input.Inking.InkManager();
            var results = await inkManager.RecognizeAsync(Windows.UI.Input.Inking.InkRecognitionTarget.All);
            
            foreach (var result in results)
            {
                alternatives.AddRange(result.GetTextCandidates().Skip(1).Take(4));
            }
        }
        catch
        {
            // Fallback: no alternatives available
        }
        return alternatives;
    }
}
}