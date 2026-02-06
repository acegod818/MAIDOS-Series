// MAIDOS-Forge Linker Manager
// UEP v1.7B Compliant - Zero Technical Debt

namespace Forge.Core.Linker;

/// <summary>
/// 鏈接器管理器 - 選擇並執行平台鏈接器
/// </summary>
/// <impl>
/// APPROACH: 維護可用鏈接器列表，根據目標平台選擇合適的鏈接器
/// CALLS: IPlatformLinker.LinkAsync()
/// EDGES: 無可用鏈接器時返回錯誤
/// </impl>
public sealed class LinkerManager
{
    private readonly List<IPlatformLinker> _linkers = new();
    private IPlatformLinker? _preferredLinker;

    /// <summary>
    /// 建立鏈接器管理器並註冊預設鏈接器
    /// </summary>
    /// <impl>
    /// APPROACH: 根據當前平台註冊對應的鏈接器
    /// CALLS: Register()
    /// EDGES: N/A
    /// </impl>
    public LinkerManager()
    {
        // 根據平台註冊鏈接器（優先順序由上到下）
        if (OperatingSystem.IsWindows())
        {
            Register(new MsvcLinker());
            Register(new LldLinker());
        }
        else if (OperatingSystem.IsMacOS())
        {
            Register(new AppleLinker());
            Register(new LldLinker());
        }
        else
        {
            // Linux / FreeBSD / Others
            Register(new LldLinker());
            Register(new GnuLinker());
        }
    }

    /// <summary>
    /// 註冊鏈接器
    /// </summary>
    /// <impl>
    /// APPROACH: 將鏈接器加入列表
    /// CALLS: N/A
    /// EDGES: N/A
    /// </impl>
    public void Register(IPlatformLinker linker)
    {
        _linkers.Add(linker);
    }

    /// <summary>
    /// 設定偏好的鏈接器
    /// </summary>
    /// <impl>
    /// APPROACH: 根據名稱查找並設定偏好鏈接器
    /// CALLS: N/A
    /// EDGES: 名稱不匹配時忽略
    /// </impl>
    public void SetPreferred(string linkerName)
    {
        _preferredLinker = _linkers.FirstOrDefault(l =>
            l.Name.Contains(linkerName, StringComparison.OrdinalIgnoreCase));
    }

    /// <summary>
    /// 取得可用的鏈接器
    /// </summary>
    /// <impl>
    /// APPROACH: 遍歷所有鏈接器，檢查可用性
    /// CALLS: IPlatformLinker.CheckAvailabilityAsync()
    /// EDGES: 無可用鏈接器返回 null
    /// </impl>
    public async Task<IPlatformLinker?> GetAvailableLinkerAsync(
        TargetPlatform target,
        CancellationToken ct = default)
    {
        // 優先使用偏好鏈接器
        if (_preferredLinker is not null)
        {
            var (available, _) = await _preferredLinker.CheckAvailabilityAsync(ct);
            if (available && _preferredLinker.SupportedPlatforms.Contains(target.Os))
            {
                return _preferredLinker;
            }
        }

        // 遍歷所有鏈接器
        foreach (var linker in _linkers)
        {
            if (!linker.SupportedPlatforms.Contains(target.Os) &&
                !linker.SupportedPlatforms.Contains("native"))
            {
                continue;
            }

            var (available, _) = await linker.CheckAvailabilityAsync(ct);
            if (available)
            {
                return linker;
            }
        }

        return null;
    }

    /// <summary>
    /// 驗證所有鏈接器
    /// </summary>
    /// <impl>
    /// APPROACH: 並行檢查所有鏈接器可用性
    /// CALLS: IPlatformLinker.CheckAvailabilityAsync()
    /// EDGES: N/A
    /// </impl>
    public async Task<IReadOnlyDictionary<string, (bool Available, string Message)>> ValidateAllAsync(
        CancellationToken ct = default)
    {
        var results = new Dictionary<string, (bool, string)>();

        foreach (var linker in _linkers)
        {
            var result = await linker.CheckAvailabilityAsync(ct);
            results[linker.Name] = result;
        }

        return results;
    }

    /// <summary>
    /// 執行鏈接
    /// </summary>
    /// <impl>
    /// APPROACH: 選擇合適的鏈接器並執行鏈接
    /// CALLS: GetAvailableLinkerAsync(), IPlatformLinker.LinkAsync()
    /// EDGES: 無可用鏈接器返回錯誤, 輸入為空返回錯誤
    /// </impl>
    public async Task<LinkResult> LinkAsync(
        IReadOnlyList<LinkInput> inputs,
        LinkConfig config,
        CancellationToken ct = default)
    {
        if (inputs.Count == 0)
        {
            return LinkResult.Failure("No inputs provided");
        }

        // 分類輸入
        var nativeInputs = inputs.Where(i => i.Type != LinkInputType.DotNetAssembly).ToList();
        var clrInputs = inputs.Where(i => i.Type == LinkInputType.DotNetAssembly).ToList();

        // 如果全是 CLR 程序集，不需要原生鏈接
        if (nativeInputs.Count == 0 && clrInputs.Count > 0)
        {
            return HandleClrOnlyLink(clrInputs, config);
        }

        // 選擇鏈接器
        var linker = await GetAvailableLinkerAsync(config.Target, ct);
        if (linker is null)
        {
            return LinkResult.Failure(
                $"No linker available for target platform: {config.Target.ToTriple()}");
        }

        // 執行鏈接
        return await linker.LinkAsync(inputs, config, ct);
    }

    /// <summary>
    /// 處理純 CLR 鏈接（實際上是複製）
    /// </summary>
    /// <impl>
    /// APPROACH: CLR 程序集不需要原生鏈接，直接複製到輸出目錄
    /// CALLS: File.Copy()
    /// EDGES: N/A
    /// </impl>
    private static LinkResult HandleClrOnlyLink(
        IReadOnlyList<LinkInput> clrInputs,
        LinkConfig config)
    {
        var startTime = DateTime.UtcNow;
        var logs = new List<string>();

        try
        {
            Directory.CreateDirectory(config.OutputDir);

            // 找出主程序集（可能是入口點）
            var mainAssembly = clrInputs.FirstOrDefault(i => !i.IsGlue);
            if (mainAssembly is null)
            {
                mainAssembly = clrInputs.First();
            }

            var outputPath = Path.Combine(
                config.OutputDir,
                config.OutputName + ".dll");

            // 複製主程序集
            File.Copy(mainAssembly.Path, outputPath, overwrite: true);
            logs.Add($"Copied: {mainAssembly.Path} -> {outputPath}");

            // 複製其他程序集
            foreach (var input in clrInputs.Where(i => i != mainAssembly))
            {
                var destPath = Path.Combine(
                    config.OutputDir,
                    Path.GetFileName(input.Path));

                File.Copy(input.Path, destPath, overwrite: true);
                logs.Add($"Copied: {input.Path} -> {destPath}");
            }

            // 複製 deps.json 和 runtimeconfig.json（如果存在）
            var mainDir = Path.GetDirectoryName(mainAssembly.Path);
            if (!string.IsNullOrEmpty(mainDir))
            {
                foreach (var ext in new[] { ".deps.json", ".runtimeconfig.json" })
                {
                    var srcFile = Path.Combine(mainDir,
                        Path.GetFileNameWithoutExtension(mainAssembly.Path) + ext);

                    if (File.Exists(srcFile))
                    {
                        var destFile = Path.Combine(config.OutputDir,
                            config.OutputName + ext);
                        File.Copy(srcFile, destFile, overwrite: true);
                        logs.Add($"Copied: {srcFile}");
                    }
                }
            }

            var duration = DateTime.UtcNow - startTime;
            return LinkResult.Success(outputPath, duration, logs);
        }
        catch (Exception ex)
        {
            var duration = DateTime.UtcNow - startTime;
            return LinkResult.Failure($"Failed to copy CLR assemblies: {ex.Message}", duration, logs);
        }
    }

    /// <summary>
    /// 收集編譯產物作為鏈接輸入
    /// </summary>
    /// <impl>
    /// APPROACH: 掃描編譯輸出目錄，收集可鏈接的檔案
    /// CALLS: Directory.GetFiles()
    /// EDGES: 目錄不存在返回空列表
    /// </impl>
    public static IReadOnlyList<LinkInput> CollectInputs(
        string buildDir,
        IReadOnlyList<(string Name, string Language)> modules)
    {
        var inputs = new List<LinkInput>();

        foreach (var (moduleName, language) in modules)
        {
            var moduleDir = Path.Combine(buildDir, moduleName);
            if (!Directory.Exists(moduleDir))
            {
                continue;
            }

            // 根據語言確定要收集的檔案類型
            var patterns = language.ToLowerInvariant() switch
            {
                "csharp" => new[] { "*.dll" },
                "rust" => new[] { "*.rlib", "*.a", "*.so", "*.dylib" },
                "c" => new[] { "*.o", "*.a" },
                "asm" => new[] { "*.o" },
                _ => new[] { "*.o", "*.a" }
            };

            foreach (var pattern in patterns)
            {
                foreach (var file in Directory.GetFiles(moduleDir, pattern))
                {
                    var inputType = DetermineInputType(file, language);
                    inputs.Add(new LinkInput
                    {
                        Path = file,
                        ModuleName = moduleName,
                        Language = language,
                        Type = inputType,
                        IsGlue = file.Contains(".glue.") || file.Contains("_ffi.")
                    });
                }
            }
        }

        return inputs;
    }

    /// <summary>
    /// 判斷輸入類型
    /// </summary>
    /// <impl>
    /// APPROACH: 根據副檔名和語言判斷輸入類型
    /// CALLS: N/A
    /// EDGES: N/A
    /// </impl>
    private static LinkInputType DetermineInputType(string filePath, string language)
    {
        var ext = Path.GetExtension(filePath).ToLowerInvariant();

        return ext switch
        {
            ".o" => LinkInputType.Object,
            ".obj" => LinkInputType.Object,
            ".a" => LinkInputType.StaticLib,
            ".lib" => LinkInputType.StaticLib,
            ".rlib" => LinkInputType.RustLib,
            ".so" => LinkInputType.SharedLib,
            ".dylib" => LinkInputType.SharedLib,
            ".dll" when language.Equals("csharp", StringComparison.OrdinalIgnoreCase)
                => LinkInputType.DotNetAssembly,
            ".dll" => LinkInputType.SharedLib,
            _ => LinkInputType.Object
        };
    }
}
