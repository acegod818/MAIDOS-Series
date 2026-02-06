// MAIDOS-Forge Configuration Models
// UEP v1.7B Compliant - Zero Technical Debt

using System.Text.Json.Serialization;

namespace Forge.Core.Config;

/// <summary>
/// 專案配置 (forge.json)
/// </summary>
/// <remarks>
/// <para>
/// 這個類別代表整個專案的配置，對應於專案根目錄下的 forge.json 檔案。
/// 它包含了專案的基本資訊、輸出設定、目標平台以及模組列表。
/// </para>
/// <para>
/// APPROACH: POCO 模型，映射 forge.json 根結構
/// CALLS: System.Text.Json 反序列化
/// EDGES: 所有欄位可空並有預設值，允許漸進式配置
/// </para>
/// </remarks>
/// <example>
/// <code>
/// {
///   "name": "my-project",
///   "version": "1.0.0",
///   "output": {
///     "dir": "build",
///     "artifact_name": "myapp"
///   },
///   "target": {
///     "default": "native"
///   },
///   "modules": ["core", "utils"]
/// }
/// </code>
/// </example>
public sealed class ForgeConfig
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("version")]
    public string Version { get; set; } = "0.1.0";

    [JsonPropertyName("output")]
    public OutputConfig Output { get; set; } = new();

    [JsonPropertyName("target")]
    public TargetConfig Target { get; set; } = new();

    [JsonPropertyName("modules")]
    public List<string> Modules { get; set; } = new();
}

/// <summary>
/// 輸出配置
/// </summary>
/// <remarks>
/// <para>
/// 定義編譯輸出的目錄結構和產物名稱。
/// </para>
/// <para>
/// APPROACH: 定義 build 輸出目錄結構
/// CALLS: N/A (純資料)
/// EDGES: 預設值確保最小可用配置
/// </para>
/// </remarks>
/// <example>
/// <code>
/// {
///   "dir": "build",
///   "artifact_name": "myapp"
/// }
/// </code>
/// </example>
public sealed class OutputConfig
{
    [JsonPropertyName("dir")]
    public string Dir { get; set; } = "build";

    [JsonPropertyName("artifact_name")]
    public string ArtifactName { get; set; } = string.Empty;
}

/// <summary>
/// 目標平台配置
/// </summary>
/// <remarks>
/// <para>
/// 定義編譯的目標平台，包括默認平台和交叉編譯選項。
/// </para>
/// <para>
/// APPROACH: 定義編譯目標平台與交叉編譯選項
/// CALLS: N/A (純資料)
/// EDGES: 預設當前平台，sysroot 為 auto
/// </para>
/// </remarks>
/// <example>
/// <code>
/// {
///   "default": "native",
///   "cross_compile": {
///     "sysroot": "auto"
///   }
/// }
/// </code>
/// </example>
public sealed class TargetConfig
{
    [JsonPropertyName("default")]
    public string Default { get; set; } = "native";

    [JsonPropertyName("cross_compile")]
    public CrossCompileConfig? CrossCompile { get; set; }
}

/// <summary>
/// 交叉編譯配置
/// </summary>
/// <remarks>
/// <para>
/// 定義交叉編譯時使用的 sysroot 來源。
/// </para>
/// <para>
/// APPROACH: 定義 sysroot 來源
/// CALLS: N/A (純資料)
/// EDGES: auto = 自動下載, generate = 本地生成, 或自訂路徑
/// </para>
/// </remarks>
/// <example>
/// <code>
/// {
///   "sysroot": "auto"
/// }
/// </code>
/// </example>
public sealed class CrossCompileConfig
{
    [JsonPropertyName("sysroot")]
    public string Sysroot { get; set; } = "auto";
}

/// <summary>
/// 模組配置 (module.json)
/// </summary>
/// <remarks>
/// <para>
/// 這個類別代表單一模組的配置，對應於模組目錄下的 module.json 檔案。
/// 它包含了模組的基本資訊、語言設定、依賴關係以及語言特定的配置。
/// </para>
/// <para>
/// APPROACH: POCO 模型，映射 module.json 根結構
/// CALLS: System.Text.Json 反序列化
/// EDGES: language 必填（無預設），其他欄位有預設值
/// </para>
/// </remarks>
/// <example>
/// <code>
/// {
///   "name": "core",
///   "language": "csharp",
///   "type": "library",
///   "dependencies": ["utils"],
///   "csharp": {
///     "mode": "clr",
///     "framework": "net8.0"
///   }
/// }
/// </code>
/// </example>
public sealed class ModuleConfig
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("language")]
    public string Language { get; set; } = string.Empty;

    [JsonPropertyName("type")]
    public string Type { get; set; } = "library";

    [JsonPropertyName("dependencies")]
    public List<string> Dependencies { get; set; } = new();

    [JsonPropertyName("exports")]
    public List<ExportConfig> Exports { get; set; } = new();

    [JsonPropertyName("csharp")]
    public CSharpConfig? CSharp { get; set; }

    [JsonPropertyName("rust")]
    public RustConfig? Rust { get; set; }

    [JsonPropertyName("c")]
    public CConfig? C { get; set; }

    [JsonPropertyName("cpp")]
    public CppConfig? Cpp { get; set; }

    [JsonPropertyName("go")]
    public GoConfig? Go { get; set; }

    [JsonPropertyName("typescript")]
    public TypeScriptConfig? TypeScript { get; set; }

    [JsonPropertyName("python")]
    public PythonConfig? Python { get; set; }

    [JsonPropertyName("asm")]
    public AsmConfig? Asm { get; set; }
}

/// <summary>
/// 導出接口配置
/// </summary>
/// <remarks>
/// <para>
/// 定義模組對外暴露的函數接口，包括函數名稱和調用約定。
/// </para>
/// <para>
/// APPROACH: 定義模組對外暴露的符號
/// CALLS: N/A (純資料)
/// EDGES: name 必填，calling_convention 預設 cdecl
/// </para>
/// </remarks>
/// <example>
/// <code>
/// {
///   "name": "process_data",
///   "calling_convention": "cdecl"
/// }
/// </code>
/// </example>
public sealed class ExportConfig
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("calling_convention")]
    public string CallingConvention { get; set; } = "cdecl";
}

/// <summary>
/// C# 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 C# 編譯選項（CLR vs NativeAOT）
/// CALLS: N/A (純資料)
/// EDGES: mode 預設 clr，framework 預設 net8.0
/// </impl>
public sealed class CSharpConfig
{
    [JsonPropertyName("mode")]
    public string Mode { get; set; } = "clr";

    [JsonPropertyName("framework")]
    public string Framework { get; set; } = "net8.0";

    [JsonPropertyName("nullable")]
    public string Nullable { get; set; } = "enable";

    [JsonPropertyName("implicit_usings")]
    public bool ImplicitUsings { get; set; } = true;

    [JsonPropertyName("output_type")]
    public string? OutputType { get; set; }
}

/// <summary>
/// Rust 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 Rust 編譯選項
/// CALLS: N/A (純資料)
/// EDGES: edition 預設 2021，profile 預設 release
/// </impl>
public sealed class RustConfig
{
    [JsonPropertyName("edition")]
    public string Edition { get; set; } = "2021";

    [JsonPropertyName("profile")]
    public string Profile { get; set; } = "release";

    [JsonPropertyName("features")]
    public List<string> Features { get; set; } = new();
}

/// <summary>
/// C 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 C 編譯選項
/// CALLS: N/A (純資料)
/// EDGES: compiler 預設空（自動偵測），standard 預設 c17
/// </impl>
public sealed class CConfig
{
    [JsonPropertyName("compiler")]
    public string Compiler { get; set; } = string.Empty;

    [JsonPropertyName("standard")]
    public string Standard { get; set; } = "c17";

    [JsonPropertyName("include_dirs")]
    public List<string> IncludeDirs { get; set; } = new();

    [JsonPropertyName("defines")]
    public List<string> Defines { get; set; } = new();
}

/// <summary>
/// C++ 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 C++ 編譯選項
/// CALLS: N/A (純資料)
/// EDGES: compiler 預設空（自動偵測），standard 預設 c++17
/// </impl>
public sealed class CppConfig
{
    [JsonPropertyName("compiler")]
    public string Compiler { get; set; } = string.Empty;

    [JsonPropertyName("standard")]
    public string Standard { get; set; } = "c++17";

    [JsonPropertyName("include_dirs")]
    public List<string> IncludeDirs { get; set; } = new();

    [JsonPropertyName("defines")]
    public List<string> Defines { get; set; } = new();

    [JsonPropertyName("libs")]
    public List<string> Libs { get; set; } = new();

    [JsonPropertyName("exceptions")]
    public bool Exceptions { get; set; } = true;

    [JsonPropertyName("rtti")]
    public bool Rtti { get; set; } = true;
}

/// <summary>
/// Go 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 Go 編譯選項
/// CALLS: N/A (純資料)
/// EDGES: buildmode 預設 c-shared 以支援 FFI
/// </impl>
public sealed class GoConfig
{
    [JsonPropertyName("buildmode")]
    public string BuildMode { get; set; } = "c-shared";

    [JsonPropertyName("cgo_enabled")]
    public bool CgoEnabled { get; set; } = true;

    [JsonPropertyName("ldflags")]
    public List<string> LdFlags { get; set; } = new();

    [JsonPropertyName("tags")]
    public List<string> Tags { get; set; } = new();
}

/// <summary>
/// TypeScript 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 TypeScript 編譯選項
/// CALLS: N/A (純資料)
/// EDGES: bundler 預設 esbuild，target 預設 es2020
/// </impl>
public sealed class TypeScriptConfig
{
    [JsonPropertyName("bundler")]
    public string Bundler { get; set; } = "esbuild";

    [JsonPropertyName("target")]
    public string Target { get; set; } = "es2020";

    [JsonPropertyName("module")]
    public string Module { get; set; } = "commonjs";

    [JsonPropertyName("declaration")]
    public bool Declaration { get; set; } = true;

    [JsonPropertyName("strict")]
    public bool Strict { get; set; } = true;
}

/// <summary>
/// Python 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 Python 編譯選項（Cython/mypyc/Nuitka）
/// CALLS: N/A (純資料)
/// EDGES: compiler 預設 cython
/// </impl>
public sealed class PythonConfig
{
    [JsonPropertyName("compiler")]
    public string Compiler { get; set; } = "cython";

    [JsonPropertyName("python_version")]
    public string PythonVersion { get; set; } = "3.11";

    [JsonPropertyName("include_dirs")]
    public List<string> IncludeDirs { get; set; } = new();

    [JsonPropertyName("optimize")]
    public int Optimize { get; set; } = 2;
}

/// <summary>
/// Assembly 語言特定配置
/// </summary>
/// <impl>
/// APPROACH: 定義 Assembly 編譯選項（NASM/GAS/MASM）
/// CALLS: N/A (純資料)
/// EDGES: assembler 預設空（自動偵測），format 依平台決定
/// </impl>
public sealed class AsmConfig
{
    [JsonPropertyName("assembler")]
    public string Assembler { get; set; } = string.Empty;

    [JsonPropertyName("format")]
    public string Format { get; set; } = string.Empty;

    [JsonPropertyName("arch")]
    public string Arch { get; set; } = "x86_64";

    [JsonPropertyName("includes")]
    public List<string> Includes { get; set; } = new();

    [JsonPropertyName("defines")]
    public List<string> Defines { get; set; } = new();

    [JsonPropertyName("debug")]
    public bool Debug { get; set; } = false;
}