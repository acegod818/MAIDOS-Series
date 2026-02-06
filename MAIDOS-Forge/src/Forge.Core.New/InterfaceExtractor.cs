// MAIDOS-Forge Interface Extractor
// UEP v1.7B Compliant - Zero Technical Debt

using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Forge.Core.Platform;

namespace Forge.Core.FFI;

/// <summary>
/// 接口提取結果
/// </summary>
/// <impl>
/// APPROACH: 封裝提取結果，包含成功/失敗狀態
/// CALLS: N/A (純資料)
/// EDGES: IsSuccess 為 false 時 Error 非空
/// </impl>
public sealed class ExtractResult
{
    public bool IsSuccess { get; }
    public string Error { get; }
    public ModuleInterface? Interface { get; }

    private ExtractResult(bool isSuccess, string error, ModuleInterface? iface)
    {
        IsSuccess = isSuccess;
        Error = error;
        Interface = iface;
    }

    public static ExtractResult Success(ModuleInterface iface)
        => new(true, string.Empty, iface);

    public static ExtractResult Failure(string error)
        => new(false, error, null);
}

/// <summary>
/// 接口提取器 - 從編譯產物提取 FFI 接口
/// </summary>
/// <impl>
/// APPROACH: 根據檔案類型選擇對應的提取策略
/// CALLS: ExtractFromCSharpAssembly(), ExtractFromNativeLibrary()
/// EDGES: 不支援的檔案類型返回失敗
/// </impl>
public static class InterfaceExtractor
{
    /// <summary>
    /// 從編譯產物提取接口
    /// </summary>
    /// <impl>
    /// APPROACH: 根據副檔名選擇提取方法
    /// CALLS: ExtractFromCSharpAssembly(), ExtractFromNativeLibrary()
    /// EDGES: 檔案不存在返回失敗, 不支援的類型返回失敗
    /// </impl>
    public static async Task<ExtractResult> ExtractAsync(
        string artifactPath,
        string moduleName,
        string sourceLanguage,
        CancellationToken ct = default)
    {
        if (!File.Exists(artifactPath))
        {
            return ExtractResult.Failure($"Artifact not found: {artifactPath}");
        }

        var ext = Path.GetExtension(artifactPath).ToLowerInvariant();

        return ext switch
        {
            ".dll" when sourceLanguage.Equals("csharp", StringComparison.OrdinalIgnoreCase)
                => ExtractFromCSharpAssembly(artifactPath, moduleName),
            ".dll" or ".so" or ".dylib" or ".a" or ".rlib"
                => await ExtractFromNativeLibraryAsync(artifactPath, moduleName, sourceLanguage, ct),
            _ => ExtractResult.Failure($"Unsupported artifact type: {ext}")
        };
    }

    /// <summary>
    /// 從 C# 程序集提取接口
    /// </summary>
    /// <impl>
    /// APPROACH: 使用 System.Reflection.Metadata 讀取方法簽名
    /// CALLS: PEReader, MetadataReader
    /// EDGES: 無公開方法返回空接口, 讀取失敗返回錯誤
    /// </impl>
    private static ExtractResult ExtractFromCSharpAssembly(string dllPath, string moduleName)
    {
        try
        {
            var exports = new List<ExportedFunction>();

            using var stream = File.OpenRead(dllPath);
            using var peReader = new PEReader(stream);

            if (!peReader.HasMetadata)
            {
                return ExtractResult.Failure("Assembly has no metadata");
            }

            var reader = peReader.GetMetadataReader();

            foreach (var typeHandle in reader.TypeDefinitions)
            {
                var typeDef = reader.GetTypeDefinition(typeHandle);
                var typeName = reader.GetString(typeDef.Name);
                var typeNamespace = reader.GetString(typeDef.Namespace);

                // 只處理公開類型
                if (!typeDef.Attributes.HasFlag(TypeAttributes.Public))
                {
                    continue;
                }

                foreach (var methodHandle in typeDef.GetMethods())
                {
                    var methodDef = reader.GetMethodDefinition(methodHandle);

                    // 只處理公開靜態方法
                    if (!methodDef.Attributes.HasFlag(MethodAttributes.Public) ||
                        !methodDef.Attributes.HasFlag(MethodAttributes.Static))
                    {
                        continue;
                    }

                    var methodName = reader.GetString(methodDef.Name);

                    // 跳過特殊方法
                    if (methodName.StartsWith('<') || methodName.StartsWith('.'))
                    {
                        continue;
                    }

                    // 解析方法簽名
                    var export = ParseCSharpMethod(reader, methodDef, $"{typeNamespace}.{typeName}");
                    if (export is not null)
                    {
                        exports.Add(export);
                    }
                }
            }

            var moduleInterface = new ModuleInterface
            {
                Version = "1.0",
                ModuleName = moduleName,
                ModuleVersion = "1.0.0",
                Language = "csharp",
                Abi = "clr",
                Mode = "clr",
                Exports = exports
            };

            return ExtractResult.Success(moduleInterface);
        }
        catch (Exception ex)
        {
            return ExtractResult.Failure($"Failed to extract interface: {ex.Message}");
        }
    }

    /// <summary>
    /// 解析 C# 方法
    /// </summary>
    /// <impl>
    /// APPROACH: 從 MethodDefinition 讀取參數和返回類型
    /// CALLS: MetadataReader.GetBlobReader()
    /// EDGES: 複雜類型簡化為 void*
    /// </impl>
    private static ExportedFunction? ParseCSharpMethod(
        MetadataReader reader,
        MethodDefinition methodDef,
        string declaringType)
    {
        var methodName = reader.GetString(methodDef.Name);

        try
        {
            var signature = methodDef.DecodeSignature(new TypeProvider(), genericContext: null);

            var parameters = new List<FunctionParameter>();
            var paramIndex = 0;

            foreach (var paramHandle in methodDef.GetParameters())
            {
                var param = reader.GetParameter(paramHandle);
                var paramName = reader.GetString(param.Name);

                if (string.IsNullOrEmpty(paramName))
                {
                    paramName = $"arg{paramIndex}";
                }

                // 簡化類型映射
                TypeRef paramType = PrimitiveType.Void;
                if (paramIndex < signature.ParameterTypes.Length)
                {
                    paramType = MapCSharpType(signature.ParameterTypes[paramIndex]);
                }

                var direction = ParameterDirection.In;
                if (param.Attributes.HasFlag(ParameterAttributes.Out))
                {
                    direction = ParameterDirection.Out;
                }

                parameters.Add(new FunctionParameter(paramName, paramType, direction));
                paramIndex++;
            }

            var returnType = MapCSharpType(signature.ReturnType);

            var funcSignature = new FunctionSignature(parameters, returnType, CallingConvention.Cdecl);

            var attributes = new List<string>();
            // 檢查 unsafe 屬性
            foreach (var attrHandle in methodDef.GetCustomAttributes())
            {
                var attr = reader.GetCustomAttribute(attrHandle);
                var ctorHandle = attr.Constructor;

                if (ctorHandle.Kind == HandleKind.MemberReference)
                {
                    var memberRef = reader.GetMemberReference((MemberReferenceHandle)ctorHandle);
                    var attrTypeName = GetTypeName(reader, memberRef.Parent);

                    if (attrTypeName.Contains("UnmanagedCallersOnly"))
                    {
                        attributes.Add("export");
                    }
                }
            }

            return new ExportedFunction(methodName, funcSignature, attributes);
        }
        catch
        {
            // 無法解析的方法，返回簡化版本
            return new ExportedFunction(
                methodName,
                new FunctionSignature(Array.Empty<FunctionParameter>(), PrimitiveType.Void),
                Array.Empty<string>());
        }
    }

    private static string GetTypeName(MetadataReader reader, EntityHandle handle)
    {
        return handle.Kind switch
        {
            HandleKind.TypeReference => reader.GetString(
                reader.GetTypeReference((TypeReferenceHandle)handle).Name),
            HandleKind.TypeDefinition => reader.GetString(
                reader.GetTypeDefinition((TypeDefinitionHandle)handle).Name),
            _ => string.Empty
        };
    }

    /// <summary>
    /// 映射 C# 類型到 FFI 類型
    /// </summary>
    /// <impl>
    /// APPROACH: 根據類型代碼映射到原始類型
    /// CALLS: N/A
    /// EDGES: 未知類型映射為 void*
    /// </impl>
    private static TypeRef MapCSharpType(string typeName)
    {
        return typeName.ToLowerInvariant() switch
        {
            "void" or "system.void" => PrimitiveType.Void,
            "bool" or "system.boolean" => PrimitiveType.Bool,
            "sbyte" or "system.sbyte" => PrimitiveType.I8,
            "short" or "system.int16" => PrimitiveType.I16,
            "int" or "system.int32" => PrimitiveType.I32,
            "long" or "system.int64" => PrimitiveType.I64,
            "byte" or "system.byte" => PrimitiveType.U8,
            "ushort" or "system.uint16" => PrimitiveType.U16,
            "uint" or "system.uint32" => PrimitiveType.U32,
            "ulong" or "system.uint64" => PrimitiveType.U64,
            "float" or "system.single" => PrimitiveType.F32,
            "double" or "system.double" => PrimitiveType.F64,
            "nint" or "system.intptr" => PrimitiveType.ISize,
            "nuint" or "system.uintptr" => PrimitiveType.USize,
            _ when typeName.EndsWith("*") => new PointerType(PrimitiveType.Void),
            _ when typeName.EndsWith("[]") => new ArrayType(PrimitiveType.U8),
            _ => new PointerType(PrimitiveType.Void) // 複雜類型作為指標
        };
    }

    /// <summary>
    /// 從原生庫提取接口
    /// </summary>
    /// <impl>
    /// APPROACH: 使用 nm 命令列出符號表
    /// CALLS: ProcessRunner.RunAsync()
    /// EDGES: nm 不可用時返回空接口
    /// </impl>
    private static async Task<ExtractResult> ExtractFromNativeLibraryAsync(
        string libPath,
        string moduleName,
        string sourceLanguage,
        CancellationToken ct)
    {
        var exports = new List<ExportedFunction>();

        // 使用 nm 列出符號
        var nmCommand = OperatingSystem.IsMacOS() ? "nm" : "nm";
        var nmArgs = OperatingSystem.IsMacOS() ? $"-g \"{libPath}\"" : $"-D --defined-only \"{libPath}\"";

        var result = await ProcessRunner.RunAsync(
            nmCommand, nmArgs,
            new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

        if (result.IsSuccess && !string.IsNullOrEmpty(result.Stdout))
        {
            foreach (var line in result.Stdout.Split('\n', StringSplitOptions.RemoveEmptyEntries))
            {
                var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);

                // 格式: address type name
                if (parts.Length < 2) continue;

                string symbolType, symbolName;

                if (parts.Length >= 3)
                {
                    symbolType = parts[1];
                    symbolName = parts[2];
                }
                else
                {
                    // 有時 nm 輸出沒有地址
                    symbolType = parts[0];
                    symbolName = parts[1];
                }

                // T = text section (函數), D/B = data
                if (symbolType != "T" && symbolType != "t")
                {
                    continue;
                }

                // 過濾 Rust mangled 符號
                if (symbolName.StartsWith("_ZN") || symbolName.Contains("$"))
                {
                    continue;
                }

                // 移除前導下劃線
                var funcName = symbolName.TrimStart('_');

                // 跳過常見的系統符號
                if (IsSystemSymbol(funcName))
                {
                    continue;
                }

                // 建立基本函數描述
                var funcSignature = new FunctionSignature(
                    Array.Empty<FunctionParameter>(),
                    PrimitiveType.I32,  // 預設返回 i32
                    CallingConvention.Cdecl);

                exports.Add(new ExportedFunction(funcName, funcSignature, Array.Empty<string>()));
            }
        }

        // 如果 nm 失敗，嘗試使用 objdump（Linux）
        if (exports.Count == 0 && !OperatingSystem.IsMacOS())
        {
            var objdumpResult = await ProcessRunner.RunAsync(
                "objdump", $"-T \"{libPath}\"",
                new ProcessConfig { Timeout = TimeSpan.FromSeconds(30) }, ct);

            if (objdumpResult.IsSuccess && !string.IsNullOrEmpty(objdumpResult.Stdout))
            {
                foreach (var line in objdumpResult.Stdout.Split('\n'))
                {
                    // objdump 格式較複雜，提取 .text 區段的符號
                    if (!line.Contains(".text") && !line.Contains("DF"))
                    {
                        continue;
                    }

                    var parts = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                    if (parts.Length < 1) continue;

                    var funcName = parts[^1].TrimStart('_');

                    if (IsSystemSymbol(funcName)) continue;

                    var funcSignature = new FunctionSignature(
                        Array.Empty<FunctionParameter>(),
                        PrimitiveType.I32,
                        CallingConvention.Cdecl);

                    exports.Add(new ExportedFunction(funcName, funcSignature, Array.Empty<string>()));
                }
            }
        }

        var abi = sourceLanguage.ToLowerInvariant() switch
        {
            "rust" => "c",
            "c" => "c",
            "csharp" => "c", // NativeAOT
            _ => "c"
        };

        var moduleInterface = new ModuleInterface
        {
            Version = "1.0",
            ModuleName = moduleName,
            ModuleVersion = "1.0.0",
            Language = sourceLanguage,
            Abi = abi,
            Mode = "native",
            Exports = exports
        };

        return ExtractResult.Success(moduleInterface);
    }

    /// <summary>
    /// 檢查是否為系統符號
    /// </summary>
    /// <impl>
    /// APPROACH: 黑名單比對
    /// CALLS: N/A
    /// EDGES: N/A
    /// </impl>
    private static bool IsSystemSymbol(string name)
    {
        var systemPrefixes = new[]
        {
            "___", "__", "GCC_", "_init", "_fini", "_start", "_end",
            "__libc", "__cxa", "__gxx", "__dso", "_ITM", "_Jv",
            "rust_", "core::", "std::", "alloc::"
        };

        return systemPrefixes.Any(prefix => name.StartsWith(prefix, StringComparison.Ordinal));
    }

    /// <summary>
    /// 類型提供者 - 用於解碼方法簽名
    /// </summary>
    private sealed class TypeProvider : ISignatureTypeProvider<string, object?>
    {
        public string GetPrimitiveType(PrimitiveTypeCode typeCode) => typeCode switch
        {
            PrimitiveTypeCode.Void => "void",
            PrimitiveTypeCode.Boolean => "bool",
            PrimitiveTypeCode.Char => "char",
            PrimitiveTypeCode.SByte => "sbyte",
            PrimitiveTypeCode.Byte => "byte",
            PrimitiveTypeCode.Int16 => "short",
            PrimitiveTypeCode.UInt16 => "ushort",
            PrimitiveTypeCode.Int32 => "int",
            PrimitiveTypeCode.UInt32 => "uint",
            PrimitiveTypeCode.Int64 => "long",
            PrimitiveTypeCode.UInt64 => "ulong",
            PrimitiveTypeCode.Single => "float",
            PrimitiveTypeCode.Double => "double",
            PrimitiveTypeCode.IntPtr => "nint",
            PrimitiveTypeCode.UIntPtr => "nuint",
            PrimitiveTypeCode.String => "string",
            PrimitiveTypeCode.Object => "object",
            _ => "void"
        };

        public string GetTypeFromDefinition(MetadataReader reader, TypeDefinitionHandle handle, byte rawTypeKind)
        {
            var typeDef = reader.GetTypeDefinition(handle);
            return reader.GetString(typeDef.Name);
        }

        public string GetTypeFromReference(MetadataReader reader, TypeReferenceHandle handle, byte rawTypeKind)
        {
            var typeRef = reader.GetTypeReference(handle);
            return reader.GetString(typeRef.Name);
        }

        public string GetSZArrayType(string elementType) => $"{elementType}[]";
        public string GetPointerType(string elementType) => $"{elementType}*";
        public string GetByReferenceType(string elementType) => $"ref {elementType}";
        public string GetGenericInstantiation(string genericType, System.Collections.Immutable.ImmutableArray<string> typeArguments)
            => $"{genericType}<{string.Join(", ", typeArguments)}>";
        public string GetArrayType(string elementType, ArrayShape shape) => $"{elementType}[{new string(',', shape.Rank - 1)}]";
        public string GetTypeFromSpecification(MetadataReader reader, object? genericContext, TypeSpecificationHandle handle, byte rawTypeKind)
            => "object";
        public string GetFunctionPointerType(MethodSignature<string> signature)
            => $"delegate*<{string.Join(", ", signature.ParameterTypes)}, {signature.ReturnType}>";
        public string GetGenericMethodParameter(object? genericContext, int index) => $"T{index}";
        public string GetGenericTypeParameter(object? genericContext, int index) => $"T{index}";
        public string GetModifiedType(string modifier, string unmodifiedType, bool isRequired) => unmodifiedType;
        public string GetPinnedType(string elementType) => elementType;
    }
}
