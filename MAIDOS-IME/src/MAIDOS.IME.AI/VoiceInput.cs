using System;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading.Tasks;

namespace MAIDOS.IME.AI
{
    /// <summary>
    /// 語音輸入類
    /// </summary>
    public class VoiceInput
    {
        private readonly string _whisperCliPath;
        private readonly string _modelPath;

        /// <summary>
        /// 構造函數
        /// </summary>
        /// <param name="whisperCliPath">Whisper CLI路徑</param>
        /// <param name="modelPath">模型路徑</param>
        public VoiceInput(string whisperCliPath = "whisper.exe", string modelPath = "models/ggml-base.en.bin")
        {
            _whisperCliPath = whisperCliPath;
            _modelPath = modelPath;
        }

        /// <summary>
        /// 轉錄音頻文件
        /// </summary>
        /// <param name="audioFilePath">音頻文件路徑</param>
        /// <returns>轉錄文本</returns>
        public async Task<string> TranscribeAudioAsync(string audioFilePath)
        {
            try
            {
                // 檢查音頻文件是否存在
                if (!File.Exists(audioFilePath))
                {
                    throw new FileNotFoundException($"音頻文件不存在: {audioFilePath}");
                }

                // 構建Whisper CLI命令
                var arguments = $"-m \"{_modelPath}\" -f \"{audioFilePath}\"";

                // 創建進程來調用Whisper CLI
                var processStartInfo = new ProcessStartInfo
                {
                    FileName = _whisperCliPath,
                    Arguments = arguments,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = new Process { StartInfo = processStartInfo };
                process.Start();

                // 讀取輸出
                var output = await process.StandardOutput.ReadToEndAsync();
                var error = await process.StandardError.ReadToEndAsync();

                await process.WaitForExitAsync();

                // 檢查是否有錯誤
                if (process.ExitCode != 0)
                {
                    throw new Exception($"Whisper轉錄失敗，退出代碼: {process.ExitCode}, 錯誤: {error}");
                }

                // 返回轉錄文本
                return output.Trim();
            }
            catch (Exception ex)
            {
                throw new Exception($"語音轉錄失敗: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// 記錄音頻並轉錄
        /// </summary>
        /// <param name="durationSeconds">錄音持續時間（秒）</param>
        /// <param name="outputFilePath">輸出音頻文件路徑</param>
        /// <returns>轉錄文本</returns>
        public async Task<string> RecordAndTranscribeAsync(int durationSeconds, string outputFilePath = "temp_recording.wav")
        {
            Console.WriteLine($"[MAIDOS-AUDIT] 開始錄音轉錄: 持續時間={durationSeconds}秒");
            
            try
            {
                // 檢查錄音權限和資源
                if (!await CheckRecordingPermissionsAsync())
                {
                    throw new Exception("缺少錄音權限或錄音設備不可用");
                }

                // 記錄音頻
                await RecordAudioAsync(durationSeconds, outputFilePath);

                // 驗證音頻文件
                if (!File.Exists(outputFilePath) || new FileInfo(outputFilePath).Length < 100)
                {
                    throw new Exception("音頻文件創建失敗");
                }

                Console.WriteLine($"[MAIDOS-AUDIT] 音頻文件創建成功: {outputFilePath}");
                
                // 轉錄音頻
                var transcription = await TranscribeAudioAsync(outputFilePath);
                Console.WriteLine($"[MAIDOS-AUDIT] 語音轉錄完成: 文字長度={transcription.Length}");

                // 刪除臨時音頻文件
                if (File.Exists(outputFilePath))
                {
                    File.Delete(outputFilePath);
                    Console.WriteLine($"[MAIDOS-AUDIT] 臨時文件已刪除: {outputFilePath}");
                }

                return transcription;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[MAIDOS-AUDIT] 語音輸入錯誤: {ex.Message}");
                
                // 刪除臨時音頻文件（如果存在）
                if (File.Exists(outputFilePath))
                {
                    File.Delete(outputFilePath);
                }

                throw new Exception($"錄音並轉錄失敗: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// 記錄音頻（使用系統錄音功能）
        /// </summary>
        /// <param name="durationSeconds">錄音持續時間（秒）</param>
        /// <param name="outputFilePath">輸出音頻文件路徑</param>
        private async Task RecordAudioAsync(int durationSeconds, string outputFilePath)
        {
            try
            {
                Console.WriteLine($"[MAIDOS-AUDIT] 開始錄音: 持續時間={durationSeconds}秒");
                
                // 使用系統指令錄音（Windows powershell）
                var psi = new ProcessStartInfo
                {
                    FileName = "powershell",
                    Arguments = $"-Command \"Add-Type -AssemblyName System.Speech; " +
                               $"$rec = New-Object System.Speech.Recognition.SpeechRecognitionEngine; " +
                               $"$rec.SetInputToDefaultAudioDevice(); " +
                               $"$stream = New-Object System.IO.MemoryStream; " +
                               $"$rec.RecognizeAsyncCancel(); " +
                               $"Start-Sleep {durationSeconds}; " +
                               $"$stream.ToArray() | Set-Content '{outputFilePath}' -Encoding Byte\"",
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

                if (process.ExitCode != 0)
                {
                    Console.WriteLine($"[MAIDOS-AUDIT] PowerShell錄音失敗: {error}");
                    throw new Exception($"系統錄音失敗: {error}");
                }

                Console.WriteLine($"[MAIDOS-AUDIT] 錄音完成: {outputFilePath}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[MAIDOS-AUDIT] 錄音過程錯誤: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// 檢查錄音權限
        /// </summary>
        /// <returns>是否具有錄音權限</returns>
        private async Task<bool> CheckRecordingPermissionsAsync()
        {
            try
            {
                Console.WriteLine("[MAIDOS-AUDIT] 檢查錄音權限");
                
                // 檢查音頻設備是否存在
                var checkProcess = new Process
                {
                    StartInfo = new ProcessStartInfo
                    {
                        FileName = "powershell",
                        Arguments = "-Command \"Get-AudioDevice -List | Where-Object {$_.Default -eq $true}\"",
                        RedirectStandardOutput = true,
                        RedirectStandardError = true,
                        UseShellExecute = false,
                        CreateNoWindow = true
                    }
                };

                checkProcess.Start();
                var output = await checkProcess.StandardOutput.ReadToEndAsync();
                var error = await checkProcess.StandardError.ReadToEndAsync();
                
                await checkProcess.WaitForExitAsync();

                if (!string.IsNullOrEmpty(output))
                {
                    Console.WriteLine($"[MAIDOS-AUDIT] 音頻設備檢測成功: {output}");
                    return true;
                }
                else
                {
                    Console.WriteLine($"[MAIDOS-AUDIT] 未檢測到音頻設備: {error}");
                    return false;
                }
            }
            catch
            {
                Console.WriteLine("[MAIDOS-AUDIT] 權限檢查失敗");
                return false;
            }
        }
    }
}