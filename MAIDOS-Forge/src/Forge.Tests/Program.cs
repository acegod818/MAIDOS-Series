// MAIDOS-Forge Tests
// UEP v1.7B Compliant - Zero Technical Debt

using System.Text.Json;
using Forge.Core.Build;
using Forge.Core.Config;

namespace Forge.Tests;

/// <summary>
/// 簡易測試框架
/// </summary>
/// <impl>
/// APPROACH: 收集測試方法，執行並報告結果
/// CALLS: Reflection, Action delegates
/// EDGES: 測試失敗拋 Exception, 捕獲並記錄
/// </impl>
public static class TestRunner
{
    private static int _passed = 0;
    private static int _failed = 0;

    public static void Main()
    {
        Console.WriteLine("MAIDOS-Forge Test Suite");
        Console.WriteLine("=======================");
        Console.WriteLine();

        // 執行所有測試
        RunTestClass<ConfigParserTests>();
        RunTestClass<DependencyAnalyzerTests>();
        RunTestClass<BuildSchedulerTests>();
        RunTestClass<PluginHostTests>();
        RunTestClass<CSharpPluginTests>();
        RunTestClass<RustPluginTests>();
        RunTestClass<CPluginTests>();
        RunTestClass<TypeSystemTests>();
        RunTestClass<GlueGeneratorTests>();
        RunTestClass<ModuleInterfaceTests>();
        RunTestClass<TargetPlatformTests>();
        RunTestClass<LinkerManagerTests>();
        RunTestClass<BuildOrchestratorTests>();
        RunTestClass<IncrementalBuildTests>();
        RunTestClass<PluginMetadataTests>();
        RunTestClass<PluginLoaderTests>();
        RunTestClass<PluginManagerTests>();
        RunTestClass<ForgeDirectoriesTests>();

        // 報告結果
        Console.WriteLine();
        Console.WriteLine("=======================");
        Console.WriteLine($"Total: {_passed + _failed}, Passed: {_passed}, Failed: {_failed}");

        Environment.Exit(_failed > 0 ? 1 : 0);
    }

    private static void RunTestClass<T>() where T : new()
    {
        var testClass = new T();
        var type = typeof(T);
        
        Console.WriteLine($"[{type.Name}]");

        foreach (var method in type.GetMethods())
        {
            if (method.Name.StartsWith("Test_") && method.GetParameters().Length == 0)
            {
                try
                {
                    method.Invoke(testClass, null);
                    Console.WriteLine($"  ✓ {method.Name}");
                    _passed++;
                }
                catch (Exception ex)
                {
                    var inner = ex.InnerException ?? ex;
                    Console.WriteLine($"  ✗ {method.Name}");
                    Console.WriteLine($"    {inner.Message}");
                    _failed++;
                }
            }
        }
        
        Console.WriteLine();
    }
}

/// <summary>
/// Assert 輔助類
/// </summary>
/// <impl>
/// APPROACH: 提供斷言方法，失敗時拋異常
/// CALLS: N/A
/// EDGES: 條件不成立拋 Exception
/// </impl>
public static class Assert
{
    public static void True(bool condition, string message = "Expected true")
    {
        if (!condition) throw new Exception(message);
    }

    public static void False(bool condition, string message = "Expected false")
    {
        if (condition) throw new Exception(message);
    }

    public static void Equal<T>(T expected, T actual, string? message = null)
    {
        if (!Equals(expected, actual))
        {
            throw new Exception(message ?? $"Expected: {expected}, Actual: {actual}");
        }
    }

    public static void NotNull<T>(T? value, string message = "Expected non-null")
    {
        if (value is null) throw new Exception(message);
    }

    public static void Null<T>(T? value, string message = "Expected null")
    {
        if (value is not null) throw new Exception(message);
    }

    public static void Contains(string expectedSubstring, string actualString, string message = "String does not contain expected substring")
    {
        if (actualString is null || !actualString.Contains(expectedSubstring))
        {
            throw new Exception(message);
        }
    }
}

/// <summary>
/// ConfigParser 測試
/// </summary>
public class ConfigParserTests
{
    private readonly string _testDir;

    public ConfigParserTests()
    {
        _testDir = Path.Combine(Path.GetTempPath(), $"forge-test-{Guid.NewGuid():N}");
        Directory.CreateDirectory(_testDir);
    }

    public void Test_ParseProject_NotFound()
    {
        var result = ConfigParser.ParseProject("/nonexistent/path");
        Assert.False(result.IsSuccess);
        Assert.True(result.Error.Contains("not found"));
    }

    public void Test_ParseProject_MissingForgeJson()
    {
        var result = ConfigParser.ParseProject(_testDir);
        Assert.False(result.IsSuccess);
        Assert.True(result.Error.Contains("forge.json"));
    }

    public void Test_ParseProject_ValidConfig()
    {
        // 建立測試配置
        var forgeJson = new
        {
            name = "test-project",
            version = "1.0.0",
            modules = Array.Empty<string>()
        };
        File.WriteAllText(
            Path.Combine(_testDir, "forge.json"),
            JsonSerializer.Serialize(forgeJson));

        var result = ConfigParser.ParseProject(_testDir);
        Assert.True(result.IsSuccess, result.Error);
        Assert.Equal("test-project", result.Value!.Config.Name);
    }

    public void Test_ParseProject_EmptyName()
    {
        var forgeJson = new { name = "", version = "1.0.0" };
        File.WriteAllText(
            Path.Combine(_testDir, "forge.json"),
            JsonSerializer.Serialize(forgeJson));

        var result = ConfigParser.ParseProject(_testDir);
        Assert.False(result.IsSuccess);
        Assert.True(result.Error.Contains("name"));
    }

    public void Test_ParseModule_ValidConfig()
    {
        var moduleDir = Path.Combine(_testDir, "test-module");
        Directory.CreateDirectory(moduleDir);

        var moduleJson = new
        {
            name = "test-module",
            language = "csharp",
            type = "library",
            dependencies = Array.Empty<string>()
        };
        File.WriteAllText(
            Path.Combine(moduleDir, "module.json"),
            JsonSerializer.Serialize(moduleJson));

        var result = ConfigParser.ParseModuleConfig(moduleDir);
        Assert.True(result.IsSuccess, result.Error);
        Assert.Equal("test-module", result.Value!.Config.Name);
        Assert.Equal("csharp", result.Value.Config.Language);
    }

    public void Test_ParseModule_UnsupportedLanguage()
    {
        var moduleDir = Path.Combine(_testDir, "bad-module");
        Directory.CreateDirectory(moduleDir);

        var moduleJson = new
        {
            name = "bad-module",
            language = "brainfuck"  // 不支援
        };
        File.WriteAllText(
            Path.Combine(moduleDir, "module.json"),
            JsonSerializer.Serialize(moduleJson));

        var result = ConfigParser.ParseModuleConfig(moduleDir);
        Assert.False(result.IsSuccess);
        Assert.True(result.Error.Contains("unsupported language"));
    }
}

/// <summary>
/// DependencyAnalyzer 測試
/// </summary>
public class DependencyAnalyzerTests
{
    private readonly string _testDir;

    public DependencyAnalyzerTests()
    {
        _testDir = Path.Combine(Path.GetTempPath(), $"forge-test-{Guid.NewGuid():N}");
        Directory.CreateDirectory(_testDir);
        Directory.CreateDirectory(Path.Combine(_testDir, "modules"));
    }

    private ValidatedForgeConfig CreateConfig(params (string name, string lang, string[] deps)[] modules)
    {
        var forgeJson = new
        {
            name = "test-project",
            modules = modules.Select(m => m.name).ToArray()
        };
        File.WriteAllText(
            Path.Combine(_testDir, "forge.json"),
            JsonSerializer.Serialize(forgeJson));

        foreach (var (name, lang, deps) in modules)
        {
            var moduleDir = Path.Combine(_testDir, "modules", name);
            Directory.CreateDirectory(moduleDir);

            var moduleJson = new
            {
                name = name,
                language = lang,
                dependencies = deps
            };
            File.WriteAllText(
                Path.Combine(moduleDir, "module.json"),
                JsonSerializer.Serialize(moduleJson));
        }

        var result = ConfigParser.ParseProject(_testDir);
        if (!result.IsSuccess) throw new Exception(result.Error);
        return result.Value!;
    }

    public void Test_Analyze_EmptyProject()
    {
        var forgeJson = new { name = "empty", modules = Array.Empty<string>() };
        File.WriteAllText(
            Path.Combine(_testDir, "forge.json"),
            JsonSerializer.Serialize(forgeJson));

        var config = ConfigParser.ParseProject(_testDir).Value!;
        var result = DependencyAnalyzer.Analyze(config);

        Assert.False(result.HasCycle);
        Assert.Equal(0, result.Graph.NodeCount);
    }

    public void Test_Analyze_NoDependencies()
    {
        var config = CreateConfig(
            ("core", "csharp", Array.Empty<string>()),
            ("utils", "csharp", Array.Empty<string>())
        );

        var result = DependencyAnalyzer.Analyze(config);
        Assert.False(result.HasCycle);
        Assert.Equal(2, result.Graph.NodeCount);
    }

    public void Test_Analyze_LinearDependency()
    {
        var config = CreateConfig(
            ("core", "csharp", Array.Empty<string>()),
            ("engine", "csharp", new[] { "core" }),
            ("app", "csharp", new[] { "engine" })
        );

        var result = DependencyAnalyzer.Analyze(config);
        Assert.False(result.HasCycle);
        Assert.Equal(3, result.Graph.NodeCount);
    }

    public void Test_Analyze_CyclicDependency()
    {
        var config = CreateConfig(
            ("a", "csharp", new[] { "c" }),
            ("b", "csharp", new[] { "a" }),
            ("c", "csharp", new[] { "b" })
        );

        var result = DependencyAnalyzer.Analyze(config);
        Assert.True(result.HasCycle);
        Assert.True(result.CycleChain.Count > 0);
    }

    public void Test_Analyze_MissingDependency()
    {
        var config = CreateConfig(
            ("app", "csharp", new[] { "nonexistent" })
        );

        var result = DependencyAnalyzer.Analyze(config);
        Assert.True(result.Error.Contains("does not exist"));
    }
}

/// <summary>
/// BuildScheduler 測試
/// </summary>
public class BuildSchedulerTests
{
    private readonly string _testDir;

    public BuildSchedulerTests()
    {
        _testDir = Path.Combine(Path.GetTempPath(), $"forge-test-{Guid.NewGuid():N}");
        Directory.CreateDirectory(_testDir);
        Directory.CreateDirectory(Path.Combine(_testDir, "modules"));
    }

    private DependencyAnalysisResult CreateAnalysis(params (string name, string lang, string[] deps)[] modules)
    {
        var forgeJson = new
        {
            name = "test-project",
            modules = modules.Select(m => m.name).ToArray()
        };
        File.WriteAllText(
            Path.Combine(_testDir, "forge.json"),
            JsonSerializer.Serialize(forgeJson));

        foreach (var (name, lang, deps) in modules)
        {
            var moduleDir = Path.Combine(_testDir, "modules", name);
            Directory.CreateDirectory(moduleDir);

            var moduleJson = new
            {
                name = name,
                language = lang,
                dependencies = deps
            };
            File.WriteAllText(
                Path.Combine(moduleDir, "module.json"),
                JsonSerializer.Serialize(moduleJson));
        }

        var config = ConfigParser.ParseProject(_testDir).Value!;
        return DependencyAnalyzer.Analyze(config);
    }

    public void Test_Schedule_EmptyGraph()
    {
        var result = BuildScheduler.CreateSchedule(
            new DependencyGraph(new Dictionary<string, DependencyNode>()));

        Assert.True(result.IsSuccess);
        Assert.Equal(0, result.Schedule!.TotalModules);
    }

    public void Test_Schedule_SingleModule()
    {
        var analysis = CreateAnalysis(("core", "csharp", Array.Empty<string>()));
        var result = BuildScheduler.CreateSchedule(analysis);

        Assert.True(result.IsSuccess);
        Assert.Equal(1, result.Schedule!.TotalModules);
        Assert.Equal(1, result.Schedule.Layers.Count);
    }

    public void Test_Schedule_ParallelModules()
    {
        var analysis = CreateAnalysis(
            ("a", "csharp", Array.Empty<string>()),
            ("b", "rust", Array.Empty<string>()),
            ("c", "c", Array.Empty<string>())
        );
        var result = BuildScheduler.CreateSchedule(analysis);

        Assert.True(result.IsSuccess);
        Assert.Equal(3, result.Schedule!.TotalModules);
        Assert.Equal(1, result.Schedule.Layers.Count);  // 全部在 Layer 0
        Assert.Equal(3, result.Schedule.MaxParallelism);
    }

    public void Test_Schedule_LinearDependency()
    {
        var analysis = CreateAnalysis(
            ("core", "csharp", Array.Empty<string>()),
            ("engine", "csharp", new[] { "core" }),
            ("app", "csharp", new[] { "engine" })
        );
        var result = BuildScheduler.CreateSchedule(analysis);

        Assert.True(result.IsSuccess);
        Assert.Equal(3, result.Schedule!.TotalModules);
        Assert.Equal(3, result.Schedule.Layers.Count);  // 三個層級
        Assert.Equal(1, result.Schedule.MaxParallelism);
    }

    public void Test_Schedule_DiamondDependency()
    {
        // Diamond: D depends on B and C, which both depend on A
        var analysis = CreateAnalysis(
            ("a", "csharp", Array.Empty<string>()),
            ("b", "csharp", new[] { "a" }),
            ("c", "csharp", new[] { "a" }),
            ("d", "csharp", new[] { "b", "c" })
        );
        var result = BuildScheduler.CreateSchedule(analysis);

        Assert.True(result.IsSuccess);
        Assert.Equal(4, result.Schedule!.TotalModules);
        Assert.Equal(3, result.Schedule.Layers.Count);

        // Layer 0: a
        // Layer 1: b, c (parallel)
        // Layer 2: d
        Assert.Equal(1, result.Schedule.Layers[0].Modules.Count);
        Assert.Equal(2, result.Schedule.Layers[1].Modules.Count);
        Assert.Equal(1, result.Schedule.Layers[2].Modules.Count);
    }
}

/// <summary>
/// PluginHost 測試
/// </summary>
public class PluginHostTests
{
    public void Test_RegisterBuiltinPlugins()
    {
        var host = new Forge.Core.Plugin.PluginHost();
        host.RegisterBuiltinPlugins();

        Assert.True(host.HasPlugin("csharp"));
        Assert.True(host.HasPlugin("rust"));
        Assert.True(host.HasPlugin("c"));
        // Note: 有多個內置插件，不只是3個
        Assert.True(host.RegisteredLanguages.Count >= 3);
    }

    public void Test_GetPlugin_CSharp()
    {
        var host = new Forge.Core.Plugin.PluginHost();
        host.RegisterBuiltinPlugins();

        var plugin = host.GetPlugin("csharp");
        Assert.NotNull(plugin);

        var caps = plugin!.GetCapabilities();
        Assert.Equal("csharp", caps.LanguageName);
        Assert.True(caps.SupportedExtensions.Contains(".cs"));
    }

    public void Test_GetPlugin_Rust()
    {
        var host = new Forge.Core.Plugin.PluginHost();
        host.RegisterBuiltinPlugins();

        var plugin = host.GetPlugin("rust");
        Assert.NotNull(plugin);

        var caps = plugin!.GetCapabilities();
        Assert.Equal("rust", caps.LanguageName);
        Assert.True(caps.SupportedExtensions.Contains(".rs"));
    }

    public void Test_GetPlugin_NotFound()
    {
        var host = new Forge.Core.Plugin.PluginHost();
        var plugin = host.GetPlugin("python");
        Assert.Null(plugin);
    }

    public void Test_GetPluginByExtension()
    {
        var host = new Forge.Core.Plugin.PluginHost();
        host.RegisterBuiltinPlugins();

        var csPlugin = host.GetPluginByExtension(".cs");
        Assert.NotNull(csPlugin);
        Assert.Equal("csharp", csPlugin!.GetCapabilities().LanguageName);

        var rsPlugin = host.GetPluginByExtension(".rs");
        Assert.NotNull(rsPlugin);
        Assert.Equal("rust", rsPlugin!.GetCapabilities().LanguageName);
    }
}

/// <summary>
/// CSharpPlugin 測試
/// </summary>
public class CSharpPluginTests
{
    public void Test_GetCapabilities()
    {
        var plugin = new Forge.Core.Plugin.CSharpPlugin();
        var caps = plugin.GetCapabilities();

        Assert.Equal("csharp", caps.LanguageName);
        Assert.True(caps.SupportsNativeCompilation);
        Assert.True(caps.SupportsCrossCompilation);
        Assert.True(caps.SupportedTargets.Count > 0);
    }

    public void Test_GenerateGlue_Rust()
    {
        var plugin = new Forge.Core.Plugin.CSharpPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "test", Version = "1.0.0" },
            Language = new Forge.Core.Plugin.InterfaceLanguage { Name = "csharp", Abi = "c" },
            Exports = new[]
            {
                new Forge.Core.Plugin.ExportedFunction
                {
                    Name = "add",
                    ReturnType = "i32",
                    Parameters = new[]
                    {
                        new Forge.Core.Plugin.FunctionParameter { Name = "a", Type = "i32" },
                        new Forge.Core.Plugin.FunctionParameter { Name = "b", Type = "i32" }
                    }
                }
            }
        };

        var result = plugin.GenerateGlue(iface, "rust");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("extern \"C\""));
        Assert.True(result.SourceCode.Contains("pub fn add"));
    }

    public void Test_GenerateGlue_Unsupported()
    {
        var plugin = new Forge.Core.Plugin.CSharpPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription();
        
        var result = plugin.GenerateGlue(iface, "python");
        Assert.False(result.IsSuccess);
        Assert.True(result.Error.Contains("Unsupported"));
    }
}

/// <summary>
/// RustPlugin 測試
/// </summary>
public class RustPluginTests
{
    public void Test_GetCapabilities()
    {
        var plugin = new Forge.Core.Plugin.RustPlugin();
        var caps = plugin.GetCapabilities();

        Assert.Equal("rust", caps.LanguageName);
        Assert.True(caps.SupportsNativeCompilation);
        Assert.True(caps.SupportsCrossCompilation);
        Assert.True(caps.SupportedTargets.Count > 0);
    }

    public void Test_GenerateGlue_CSharp()
    {
        var plugin = new Forge.Core.Plugin.RustPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "crypto", Version = "1.0.0" },
            Language = new Forge.Core.Plugin.InterfaceLanguage { Name = "rust", Abi = "c" },
            Exports = new[]
            {
                new Forge.Core.Plugin.ExportedFunction
                {
                    Name = "hash_sha256",
                    ReturnType = "u32",
                    Parameters = new[]
                    {
                        new Forge.Core.Plugin.FunctionParameter { Name = "data", Type = "u8" },
                        new Forge.Core.Plugin.FunctionParameter { Name = "len", Type = "usize" }
                    }
                }
            }
        };

        var result = plugin.GenerateGlue(iface, "csharp");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("DllImport"));
        Assert.True(result.SourceCode.Contains("hash_sha256"));
    }

    public void Test_GenerateGlue_C()
    {
        var plugin = new Forge.Core.Plugin.RustPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "crypto", Version = "1.0.0" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "c");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("#ifndef"));
        Assert.True(result.FileName.EndsWith(".h"));
    }
}

/// <summary>
/// FFI TypeSystem 測試
/// </summary>
public class TypeSystemTests
{
    public void Test_PrimitiveType_FromString()
    {
        Assert.Equal(Forge.Core.FFI.TypeKind.I32, 
            Forge.Core.FFI.PrimitiveType.FromString("int")!.Kind);
        Assert.Equal(Forge.Core.FFI.TypeKind.U8,
            Forge.Core.FFI.PrimitiveType.FromString("byte")!.Kind);
        Assert.Equal(Forge.Core.FFI.TypeKind.F64,
            Forge.Core.FFI.PrimitiveType.FromString("double")!.Kind);
        Assert.Null(Forge.Core.FFI.PrimitiveType.FromString("unknown"));
    }

    public void Test_PrimitiveType_ToCType()
    {
        Assert.Equal("int32_t", Forge.Core.FFI.PrimitiveType.I32.ToCType());
        Assert.Equal("uint8_t", Forge.Core.FFI.PrimitiveType.U8.ToCType());
        Assert.Equal("void", Forge.Core.FFI.PrimitiveType.Void.ToCType());
    }

    public void Test_PrimitiveType_ToCSharpType()
    {
        Assert.Equal("int", Forge.Core.FFI.PrimitiveType.I32.ToCSharpType());
        Assert.Equal("byte", Forge.Core.FFI.PrimitiveType.U8.ToCSharpType());
        Assert.Equal("nint", Forge.Core.FFI.PrimitiveType.ISize.ToCSharpType());
    }

    public void Test_PrimitiveType_ToRustType()
    {
        Assert.Equal("i32", Forge.Core.FFI.PrimitiveType.I32.ToRustType());
        Assert.Equal("u8", Forge.Core.FFI.PrimitiveType.U8.ToRustType());
        Assert.Equal("()", Forge.Core.FFI.PrimitiveType.Void.ToRustType());
    }

    public void Test_PointerType_Conversion()
    {
        var ptr = new Forge.Core.FFI.PointerType(Forge.Core.FFI.PrimitiveType.U8, true, false);
        Assert.Equal("const uint8_t*", ptr.ToCType());
        Assert.Equal("byte*", ptr.ToCSharpType());
        Assert.Equal("*const u8", ptr.ToRustType());
    }

    public void Test_ArrayType_Conversion()
    {
        var arr = new Forge.Core.FFI.ArrayType(Forge.Core.FFI.PrimitiveType.I32, 10);
        Assert.Equal("int32_t[10]", arr.ToCType());
        Assert.Equal("[i32; 10]", arr.ToRustType());
    }
}

/// <summary>
/// GlueGenerator 測試
/// </summary>
public class GlueGeneratorTests
{
    public void Test_Generate_CSharp()
    {
        var iface = new Forge.Core.FFI.ModuleInterface
        {
            ModuleName = "crypto",
            Language = "rust",
            Exports = new[]
            {
                new Forge.Core.FFI.ExportedFunction(
                    "hash_sha256",
                    new Forge.Core.FFI.FunctionSignature(
                        new[]
                        {
                            new Forge.Core.FFI.FunctionParameter("data", 
                                new Forge.Core.FFI.PointerType(Forge.Core.FFI.PrimitiveType.U8)),
                            new Forge.Core.FFI.FunctionParameter("len", Forge.Core.FFI.PrimitiveType.USize)
                        },
                        Forge.Core.FFI.PrimitiveType.I32
                    )
                )
            }
        };

        var result = Forge.Core.FFI.GlueGenerator.Generate(iface, "csharp");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("DllImport"));
        Assert.True(result.SourceCode.Contains("hash_sha256"));
        Assert.True(result.SourceCode.Contains("byte*"));
        Assert.True(result.FileName.EndsWith(".cs"));
    }

    public void Test_Generate_Rust()
    {
        var iface = new Forge.Core.FFI.ModuleInterface
        {
            ModuleName = "core",
            Language = "csharp",
            Exports = new[]
            {
                new Forge.Core.FFI.ExportedFunction(
                    "add",
                    new Forge.Core.FFI.FunctionSignature(
                        new[]
                        {
                            new Forge.Core.FFI.FunctionParameter("a", Forge.Core.FFI.PrimitiveType.I32),
                            new Forge.Core.FFI.FunctionParameter("b", Forge.Core.FFI.PrimitiveType.I32)
                        },
                        Forge.Core.FFI.PrimitiveType.I32
                    )
                )
            }
        };

        var result = Forge.Core.FFI.GlueGenerator.Generate(iface, "rust");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("extern \"C\""));
        Assert.True(result.SourceCode.Contains("pub fn add"));
        Assert.True(result.SourceCode.Contains("i32"));
        Assert.True(result.FileName.EndsWith(".rs"));
    }

    public void Test_Generate_C()
    {
        var iface = new Forge.Core.FFI.ModuleInterface
        {
            ModuleName = "crypto",
            Language = "rust",
            Exports = new[]
            {
                new Forge.Core.FFI.ExportedFunction(
                    "encrypt",
                    new Forge.Core.FFI.FunctionSignature(
                        Array.Empty<Forge.Core.FFI.FunctionParameter>(),
                        Forge.Core.FFI.PrimitiveType.Void
                    )
                )
            }
        };

        var result = Forge.Core.FFI.GlueGenerator.Generate(iface, "c");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("#ifndef"));
        Assert.True(result.SourceCode.Contains("void encrypt(void);"));
        Assert.True(result.FileName.EndsWith(".h"));
    }

    public void Test_Generate_Unsupported()
    {
        var iface = new Forge.Core.FFI.ModuleInterface { ModuleName = "test" };
        var result = Forge.Core.FFI.GlueGenerator.Generate(iface, "python");
        Assert.False(result.IsSuccess);
        Assert.True(result.Error.Contains("Unsupported"));
    }
}

/// <summary>
/// ModuleInterface JSON 測試
/// </summary>
public class ModuleInterfaceTests
{
    public void Test_ToJson_Basic()
    {
        var iface = new Forge.Core.FFI.ModuleInterface
        {
            ModuleName = "test",
            Language = "rust",
            Exports = Array.Empty<Forge.Core.FFI.ExportedFunction>()
        };

        var json = iface.ToJson();
        Assert.True(json.Contains("\"name\": \"test\""));
        Assert.True(json.Contains("\"exports\": []"));
    }

    public void Test_ToJson_WithExports()
    {
        var iface = new Forge.Core.FFI.ModuleInterface
        {
            ModuleName = "crypto",
            Language = "rust",
            Exports = new[]
            {
                new Forge.Core.FFI.ExportedFunction(
                    "hash",
                    new Forge.Core.FFI.FunctionSignature(
                        new[] { new Forge.Core.FFI.FunctionParameter("data", Forge.Core.FFI.PrimitiveType.U8) },
                        Forge.Core.FFI.PrimitiveType.I32
                    ),
                    new[] { "pure" }
                )
            }
        };

        var json = iface.ToJson();
        Assert.True(json.Contains("\"name\": \"hash\""));
        Assert.True(json.Contains("\"pure\""));
    }
}

/// <summary>
/// TargetPlatform 測試
/// </summary>
public class TargetPlatformTests
{
    public void Test_Current_Platform()
    {
        var platform = Forge.Core.Linker.TargetPlatform.Current;
        Assert.NotNull(platform.Os);
        Assert.NotNull(platform.Arch);
        Assert.False(platform.Os == "unknown");
    }

    public void Test_GetSharedLibExtension()
    {
        var linux = new Forge.Core.Linker.TargetPlatform { Os = "linux" };
        Assert.Equal(".so", linux.GetSharedLibExtension());

        var windows = new Forge.Core.Linker.TargetPlatform { Os = "windows" };
        Assert.Equal(".dll", windows.GetSharedLibExtension());

        var macos = new Forge.Core.Linker.TargetPlatform { Os = "macos" };
        Assert.Equal(".dylib", macos.GetSharedLibExtension());
    }

    public void Test_GetExecutableExtension()
    {
        var linux = new Forge.Core.Linker.TargetPlatform { Os = "linux" };
        Assert.Equal("", linux.GetExecutableExtension());

        var windows = new Forge.Core.Linker.TargetPlatform { Os = "windows" };
        Assert.Equal(".exe", windows.GetExecutableExtension());
    }

    public void Test_ToTriple()
    {
        var platform = new Forge.Core.Linker.TargetPlatform { Os = "linux", Arch = "x86_64" };
        Assert.Equal("x86_64-linux", platform.ToTriple());
    }
}

/// <summary>
/// LinkerManager 測試
/// </summary>
public class LinkerManagerTests
{
    public void Test_CollectInputs_Empty()
    {
        var inputs = Forge.Core.Linker.LinkerManager.CollectInputs(
            "/nonexistent/path",
            new[] { ("test", "csharp") });
        Assert.Equal(0, inputs.Count);
    }

    public void Test_LinkConfig_Defaults()
    {
        var config = new Forge.Core.Linker.LinkConfig();
        Assert.Equal("output", config.OutputName);
        Assert.Equal("build", config.OutputDir);
        Assert.Equal(Forge.Core.Linker.OutputType.Executable, config.OutputType);
        Assert.False(config.EnableLto);
        Assert.False(config.StripSymbols);
    }

    public void Test_LinkInput_Defaults()
    {
        var input = new Forge.Core.Linker.LinkInput();
        Assert.Equal(string.Empty, input.Path);
        Assert.Equal(Forge.Core.Linker.LinkInputType.Object, input.Type);
        Assert.False(input.IsGlue);
    }

    public void Test_LinkerManager_Creation()
    {
        var manager = new Forge.Core.Linker.LinkerManager();
        // 應該不拋異常
        Assert.NotNull(manager);
    }
}

/// <summary>
/// CPlugin 測試
/// </summary>
public class CPluginTests
{
    public void Test_GetCapabilities()
    {
        var plugin = new Forge.Core.Plugin.CPlugin();
        var caps = plugin.GetCapabilities();

        Assert.Equal("c", caps.LanguageName);
        Assert.True(caps.SupportedExtensions.Contains(".c"));
        Assert.True(caps.SupportedExtensions.Contains(".h"));
        Assert.True(caps.SupportsNativeCompilation);
        Assert.True(caps.SupportsCrossCompilation);
    }

    public void Test_GenerateGlue_CSharp()
    {
        var plugin = new Forge.Core.Plugin.CPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "native", Version = "1.0.0" },
            Language = new Forge.Core.Plugin.InterfaceLanguage { Name = "c", Abi = "c" },
            Exports = new[]
            {
                new Forge.Core.Plugin.ExportedFunction
                {
                    Name = "compute",
                    ReturnType = "i32",
                    Parameters = Array.Empty<Forge.Core.Plugin.FunctionParameter>()
                }
            }
        };

        var result = plugin.GenerateGlue(iface, "csharp");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("DllImport"));
        Assert.True(result.SourceCode.Contains("compute"));
    }

    public void Test_GenerateGlue_Rust()
    {
        var plugin = new Forge.Core.Plugin.CPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "native", Version = "1.0.0" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "rust");
        Assert.True(result.IsSuccess);
        Assert.True(result.SourceCode.Contains("extern \"C\""));
    }
}

/// <summary>
/// BuildOrchestrator 測試
/// </summary>
public class BuildOrchestratorTests
{
    public void Test_BuildOptions_Defaults()
    {
        var options = new Forge.Core.Orchestration.BuildOptions();

        Assert.Equal("release", options.Profile);
        Assert.Equal(Forge.Core.Linker.OutputType.Executable, options.OutputType);
        Assert.False(options.CompileOnly);
        Assert.True(options.GenerateGlue);
        Assert.False(options.StripSymbols);
        Assert.False(options.Verbose);
        Assert.False(options.DryRun);
        Assert.True(options.Incremental);
        Assert.False(options.ForceRebuild);
    }

    public void Test_BuildResult_Success()
    {
        var result = Forge.Core.Orchestration.BuildResult.Success(
            "/build/output",
            TimeSpan.FromSeconds(5),
            Array.Empty<Forge.Core.Orchestration.ModuleBuildResult>());

        Assert.True(result.IsSuccess);
        Assert.Equal("/build/output", result.OutputPath);
    }

    public void Test_BuildResult_Failure()
    {
        var result = Forge.Core.Orchestration.BuildResult.Failure("Test error");

        Assert.False(result.IsSuccess);
        Assert.Equal("Test error", result.Error);
    }

    public void Test_Orchestrator_Creation()
    {
        var orchestrator = new Forge.Core.Orchestration.BuildOrchestrator();
        Assert.NotNull(orchestrator);
    }
}

/// <summary>
/// IncrementalBuildManager 測試
/// </summary>
public class IncrementalBuildTests
{
    public void Test_CacheManager_Creation()
    {
        var manager = new Forge.Core.Cache.IncrementalBuildManager("/tmp/test-project");
        Assert.NotNull(manager);
    }

    public void Test_LoadCache_Empty()
    {
        var manager = new Forge.Core.Cache.IncrementalBuildManager("/tmp/nonexistent-" + Guid.NewGuid());
        var cache = manager.LoadCache();

        Assert.NotNull(cache);
        Assert.Equal(0, cache.Modules.Count);
    }

    public void Test_CheckModule_NoCache()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), "forge-test-" + Guid.NewGuid());
        Directory.CreateDirectory(tempDir);

        try
        {
            var manager = new Forge.Core.Cache.IncrementalBuildManager(tempDir);
            manager.LoadCache();

            var result = manager.CheckModule(
                tempDir,
                "test-module",
                "csharp",
                "release",
                Array.Empty<string>());

            Assert.True(result.NeedsRebuild);
            Assert.Equal("No cache entry", result.Reason);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    public void Test_ModuleCacheEntry_Defaults()
    {
        var entry = new Forge.Core.Cache.ModuleCacheEntry();

        Assert.Equal(string.Empty, entry.ModuleName);
        Assert.Equal(string.Empty, entry.Language);
        Assert.Equal("release", entry.Profile);
        Assert.NotNull(entry.ArtifactPaths);
    }
}

// ============================================================================
// M7 Hot-Pluggable Plugin System Tests
// ============================================================================

/// <summary>
/// PluginMetadata 測試
/// </summary>
public class PluginMetadataTests
{
    public void Test_Metadata_Creation()
    {
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.test",
            Version = "1.0.0",
            Language = "test",
            Entry = "Test.dll",
            PluginClass = "Test.TestPlugin"
        };

        Assert.Equal("forge.plugin.test", metadata.Name);
        Assert.Equal("1.0.0", metadata.Version);
        Assert.Equal("test", metadata.Language);
    }

    public void Test_Metadata_Validate_Valid()
    {
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.test",
            Version = "1.0.0",
            Language = "test",
            Entry = "Test.dll",
            PluginClass = "Test.TestPlugin"
        };

        var (isValid, error) = metadata.Validate();
        Assert.True(isValid, $"Expected valid but got error: {error}");
    }

    public void Test_Metadata_Validate_MissingName()
    {
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Version = "1.0.0",
            Language = "test",
            Entry = "Test.dll",
            PluginClass = "Test.TestPlugin"
        };

        var (isValid, error) = metadata.Validate();
        Assert.False(isValid);
        Assert.Contains("name", error.ToLowerInvariant());
    }

    public void Test_Metadata_Validate_MissingLanguage()
    {
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.test",
            Version = "1.0.0",
            Entry = "Test.dll",
            PluginClass = "Test.TestPlugin"
        };

        var (isValid, error) = metadata.Validate();
        Assert.False(isValid);
        Assert.Contains("language", error.ToLowerInvariant());
    }

    public void Test_Metadata_ToJson()
    {
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.test",
            Version = "1.0.0",
            Language = "test",
            Entry = "Test.dll",
            PluginClass = "Test.TestPlugin"
        };

        var json = metadata.ToJson();
        Assert.Contains("forge.plugin.test", json);
        Assert.Contains("1.0.0", json);
    }

    public void Test_Metadata_LoadFromJson()
    {
        var json = """
            {
                "name": "forge.plugin.test",
                "version": "1.0.0",
                "language": "test",
                "entry": "Test.dll",
                "pluginClass": "Test.TestPlugin"
            }
            """;

        var metadata = Forge.Core.Plugin.PluginMetadata.LoadFromJson(json);
        Assert.NotNull(metadata);
        Assert.Equal("forge.plugin.test", metadata!.Name);
    }
}

/// <summary>
/// PluginLoader 測試
/// </summary>
public class PluginLoaderTests
{
    public void Test_Loader_Creation()
    {
        var loader = new Forge.Core.Plugin.PluginLoader();
        Assert.NotNull(loader);
        Assert.Equal(0, loader.LoadedPlugins.Count);
    }

    public void Test_Loader_LoadBuiltinPlugin()
    {
        var loader = new Forge.Core.Plugin.PluginLoader();
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.csharp",
            Version = "0.7.0",
            Language = "csharp",
            IsBuiltin = true,
            Entry = "Forge.Core.dll",
            PluginClass = "Forge.Core.Plugin.CSharpPlugin"
        };

        var plugin = new Forge.Core.Plugin.CSharpPlugin();
        var result = loader.LoadBuiltinPlugin(metadata, plugin);

        Assert.True(result.IsLoaded);
        Assert.NotNull(result.Instance);
    }

    public void Test_Loader_GetPlugin()
    {
        var loader = new Forge.Core.Plugin.PluginLoader();
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.csharp",
            Version = "0.7.0",
            Language = "csharp",
            IsBuiltin = true,
            Entry = "Forge.Core.dll",
            PluginClass = "Forge.Core.Plugin.CSharpPlugin"
        };

        loader.LoadBuiltinPlugin(metadata, new Forge.Core.Plugin.CSharpPlugin());
        
        var plugin = loader.GetPlugin("csharp");
        Assert.NotNull(plugin);
    }

    public void Test_Loader_IsLoaded()
    {
        var loader = new Forge.Core.Plugin.PluginLoader();
        Assert.False(loader.IsLoaded("csharp"));

        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.csharp",
            Version = "0.7.0",
            Language = "csharp",
            IsBuiltin = true,
            Entry = "Forge.Core.dll",
            PluginClass = "Forge.Core.Plugin.CSharpPlugin"
        };

        loader.LoadBuiltinPlugin(metadata, new Forge.Core.Plugin.CSharpPlugin());
        Assert.True(loader.IsLoaded("csharp"));
    }

    public void Test_Loader_UnloadBuiltinFails()
    {
        var loader = new Forge.Core.Plugin.PluginLoader();
        var metadata = new Forge.Core.Plugin.PluginMetadata
        {
            Name = "forge.plugin.csharp",
            Version = "0.7.0",
            Language = "csharp",
            IsBuiltin = true,
            Entry = "Forge.Core.dll",
            PluginClass = "Forge.Core.Plugin.CSharpPlugin"
        };

        loader.LoadBuiltinPlugin(metadata, new Forge.Core.Plugin.CSharpPlugin());
        var (success, message) = loader.UnloadPlugin("csharp");
        
        Assert.False(success);
        Assert.Contains("builtin", message.ToLowerInvariant());
    }
}

/// <summary>
/// PluginManager 測試
/// </summary>
public class PluginManagerTests
{
    public void Test_Manager_Creation()
    {
        using var manager = new Forge.Core.Plugin.PluginManager();
        Assert.NotNull(manager);
        Assert.NotNull(manager.Loader);
        Assert.NotNull(manager.Discovery);
    }

    public void Test_Manager_Initialize_LoadsBuiltins()
    {
        using var manager = new Forge.Core.Plugin.PluginManager();
        var result = manager.InitializeAsync(loadBuiltins: true).GetAwaiter().GetResult();

        Assert.True(result.SuccessCount >= 3, "Should load at least 3 builtin plugins");
        Assert.True(manager.Loader.IsLoaded("csharp"));
        Assert.True(manager.Loader.IsLoaded("rust"));
        Assert.True(manager.Loader.IsLoaded("c"));
    }

    public void Test_Manager_ListPlugins()
    {
        using var manager = new Forge.Core.Plugin.PluginManager();
        manager.InitializeAsync(loadBuiltins: true).GetAwaiter().GetResult();

        var plugins = manager.ListInstalledPlugins();
        Assert.True(plugins.Count >= 3);
    }

    public void Test_Manager_CreateTemplate()
    {
        using var manager = new Forge.Core.Plugin.PluginManager();
        var tempDir = Path.Combine(Path.GetTempPath(), $"forge-plugin-test-{Guid.NewGuid()}");

        try
        {
            var result = manager.CreatePluginTemplate(tempDir, "mylang", "forge.plugin.mylang");
            Assert.True(result.IsSuccess, result.Message);

            Assert.True(File.Exists(Path.Combine(tempDir, "plugin.json")));
            Assert.True(File.Exists(Path.Combine(tempDir, "forge.plugin.mylang.csproj")));
            Assert.True(File.Exists(Path.Combine(tempDir, "MylangPlugin.cs")));
        }
        finally
        {
            if (Directory.Exists(tempDir))
            {
                Directory.Delete(tempDir, true);
            }
        }
    }

    public void Test_Manager_RemoveBuiltinFails()
    {
        using var manager = new Forge.Core.Plugin.PluginManager();
        manager.InitializeAsync(loadBuiltins: true).GetAwaiter().GetResult();

        var result = manager.RemovePlugin("csharp");
        Assert.False(result.IsSuccess);
        Assert.Contains("builtin", result.Message.ToLowerInvariant());
    }
}

/// <summary>
/// ForgeDirectories 測試
/// </summary>
public class ForgeDirectoriesTests
{
    public void Test_ForgeHome_NotEmpty()
    {
        var home = Forge.Core.Plugin.ForgeDirectories.ForgeHome;
        Assert.False(string.IsNullOrEmpty(home));
        Assert.Contains(".forge", home);
    }

    public void Test_PluginsDir_Path()
    {
        var pluginsDir = Forge.Core.Plugin.ForgeDirectories.PluginsDir;
        Assert.Contains("plugins", pluginsDir);
        Assert.Contains(".forge", pluginsDir);
    }

    public void Test_CacheDir_Path()
    {
        var cacheDir = Forge.Core.Plugin.ForgeDirectories.CacheDir;
        Assert.Contains("cache", cacheDir);
        Assert.Contains(".forge", cacheDir);
    }

    public void Test_EnsureDirectories_Creates()
    {
        Forge.Core.Plugin.ForgeDirectories.EnsureDirectories();
        
        // 驗證目錄已建立
        Assert.True(Directory.Exists(Forge.Core.Plugin.ForgeDirectories.ForgeHome));
        Assert.True(Directory.Exists(Forge.Core.Plugin.ForgeDirectories.PluginsDir));
        Assert.True(Directory.Exists(Forge.Core.Plugin.ForgeDirectories.CacheDir));
    }
}