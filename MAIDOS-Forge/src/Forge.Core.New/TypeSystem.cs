// MAIDOS-Forge FFI Type System
// UEP v1.7B Compliant - Zero Technical Debt

using System.Text.Json;
using System.Text.Json.Serialization;

namespace Forge.Core.FFI;

/// <summary>
/// 類型種類
/// </summary>
public enum TypeKind
{
    Void,
    Bool,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    ISize, USize,
    Ptr,
    Array,
    Struct,
    FnPtr
}

/// <summary>
/// 類型引用基類
/// </summary>
/// <impl>
/// APPROACH: 使用多態表示各種類型，支援 JSON 序列化
/// CALLS: N/A (純資料)
/// EDGES: Kind 決定子類型
/// </impl>
[JsonPolymorphic(TypeDiscriminatorPropertyName = "kind")]
[JsonDerivedType(typeof(PrimitiveType), "primitive")]
[JsonDerivedType(typeof(PointerType), "ptr")]
[JsonDerivedType(typeof(ArrayType), "array")]
[JsonDerivedType(typeof(StructType), "struct")]
[JsonDerivedType(typeof(FunctionPointerType), "fn_ptr")]
public abstract class TypeRef
{
    public abstract TypeKind Kind { get; }

    /// <summary>
    /// 取得類型的 C 表示
    /// </summary>
    public abstract string ToCType();

    /// <summary>
    /// 取得類型的 C# 表示
    /// </summary>
    public abstract string ToCSharpType();

    /// <summary>
    /// 取得類型的 Rust 表示
    /// </summary>
    public abstract string ToRustType();

    /// <summary>
    /// 從 JSON 字串解析類型
    /// </summary>
    /// <impl>
    /// APPROACH: 根據 kind 欄位判斷類型並解析
    /// CALLS: JsonDocument.Parse()
    /// EDGES: 未知類型返回 null
    /// </impl>
    public static TypeRef? FromJson(string json)
    {
        using var doc = JsonDocument.Parse(json);
        return FromJsonElement(doc.RootElement);
    }

    /// <summary>
    /// 從 JsonElement 解析類型
    /// </summary>
    /// <impl>
    /// APPROACH: 根據 kind 欄位建立對應類型實例
    /// CALLS: JsonElement.GetProperty()
    /// EDGES: 未知 kind 返回 null, 缺少必填欄位返回 null
    /// </impl>
    public static TypeRef? FromJsonElement(JsonElement element)
    {
        if (!element.TryGetProperty("kind", out var kindProp))
        {
            return null;
        }

        var kindStr = kindProp.GetString()?.ToLowerInvariant();
        if (string.IsNullOrEmpty(kindStr))
        {
            return null;
        }

        return kindStr switch
        {
            "void" => PrimitiveType.Void,
            "bool" => PrimitiveType.Bool,
            "i8" => PrimitiveType.I8,
            "i16" => PrimitiveType.I16,
            "i32" => PrimitiveType.I32,
            "i64" => PrimitiveType.I64,
            "u8" => PrimitiveType.U8,
            "u16" => PrimitiveType.U16,
            "u32" => PrimitiveType.U32,
            "u64" => PrimitiveType.U64,
            "f32" => PrimitiveType.F32,
            "f64" => PrimitiveType.F64,
            "isize" => PrimitiveType.ISize,
            "usize" => PrimitiveType.USize,
            "ptr" => ParsePointerType(element),
            "array" => ParseArrayType(element),
            "struct" => ParseStructType(element),
            "fn_ptr" => ParseFunctionPointerType(element),
            _ => null
        };
    }

    private static PointerType? ParsePointerType(JsonElement element)
    {
        if (!element.TryGetProperty("pointee", out var pointeeProp))
        {
            return null;
        }

        var pointee = FromJsonElement(pointeeProp);
        if (pointee is null)
        {
            return null;
        }

        var nullable = true;
        var mutable = true;

        if (element.TryGetProperty("nullable", out var nullableProp))
        {
            nullable = nullableProp.GetBoolean();
        }

        if (element.TryGetProperty("mutable", out var mutableProp))
        {
            mutable = mutableProp.GetBoolean();
        }

        return new PointerType(pointee, nullable, mutable);
    }

    private static ArrayType? ParseArrayType(JsonElement element)
    {
        if (!element.TryGetProperty("element", out var elementProp))
        {
            return null;
        }

        var elementType = FromJsonElement(elementProp);
        if (elementType is null)
        {
            return null;
        }

        int? length = null;
        if (element.TryGetProperty("length", out var lengthProp) && 
            lengthProp.ValueKind == JsonValueKind.Number)
        {
            length = lengthProp.GetInt32();
        }

        return new ArrayType(elementType, length);
    }

    private static StructType? ParseStructType(JsonElement element)
    {
        if (!element.TryGetProperty("name", out var nameProp))
        {
            return null;
        }

        var name = nameProp.GetString();
        if (string.IsNullOrEmpty(name))
        {
            return null;
        }

        return new StructType(name);
    }

    private static FunctionPointerType? ParseFunctionPointerType(JsonElement element)
    {
        if (!element.TryGetProperty("signature", out var sigProp))
        {
            return null;
        }

        var sig = FunctionSignature.FromJsonElement(sigProp);
        return sig is not null ? new FunctionPointerType(sig) : null;
    }
}

/// <summary>
/// 原始類型
/// </summary>
/// <impl>
/// APPROACH: 使用單例模式表示基本類型
/// CALLS: N/A
/// EDGES: N/A
/// </impl>
public sealed class PrimitiveType : TypeRef
{
    public static readonly PrimitiveType Void = new(TypeKind.Void);
    public static readonly PrimitiveType Bool = new(TypeKind.Bool);
    public static readonly PrimitiveType I8 = new(TypeKind.I8);
    public static readonly PrimitiveType I16 = new(TypeKind.I16);
    public static readonly PrimitiveType I32 = new(TypeKind.I32);
    public static readonly PrimitiveType I64 = new(TypeKind.I64);
    public static readonly PrimitiveType U8 = new(TypeKind.U8);
    public static readonly PrimitiveType U16 = new(TypeKind.U16);
    public static readonly PrimitiveType U32 = new(TypeKind.U32);
    public static readonly PrimitiveType U64 = new(TypeKind.U64);
    public static readonly PrimitiveType F32 = new(TypeKind.F32);
    public static readonly PrimitiveType F64 = new(TypeKind.F64);
    public static readonly PrimitiveType ISize = new(TypeKind.ISize);
    public static readonly PrimitiveType USize = new(TypeKind.USize);

    private readonly TypeKind _kind;
    public override TypeKind Kind => _kind;

    private PrimitiveType(TypeKind kind) => _kind = kind;

    public override string ToCType() => _kind switch
    {
        TypeKind.Void => "void",
        TypeKind.Bool => "_Bool",
        TypeKind.I8 => "int8_t",
        TypeKind.I16 => "int16_t",
        TypeKind.I32 => "int32_t",
        TypeKind.I64 => "int64_t",
        TypeKind.U8 => "uint8_t",
        TypeKind.U16 => "uint16_t",
        TypeKind.U32 => "uint32_t",
        TypeKind.U64 => "uint64_t",
        TypeKind.F32 => "float",
        TypeKind.F64 => "double",
        TypeKind.ISize => "intptr_t",
        TypeKind.USize => "size_t",
        _ => "void"
    };

    public override string ToCSharpType() => _kind switch
    {
        TypeKind.Void => "void",
        TypeKind.Bool => "bool",
        TypeKind.I8 => "sbyte",
        TypeKind.I16 => "short",
        TypeKind.I32 => "int",
        TypeKind.I64 => "long",
        TypeKind.U8 => "byte",
        TypeKind.U16 => "ushort",
        TypeKind.U32 => "uint",
        TypeKind.U64 => "ulong",
        TypeKind.F32 => "float",
        TypeKind.F64 => "double",
        TypeKind.ISize => "nint",
        TypeKind.USize => "nuint",
        _ => "void"
    };

    public override string ToRustType() => _kind switch
    {
        TypeKind.Void => "()",
        TypeKind.Bool => "bool",
        TypeKind.I8 => "i8",
        TypeKind.I16 => "i16",
        TypeKind.I32 => "i32",
        TypeKind.I64 => "i64",
        TypeKind.U8 => "u8",
        TypeKind.U16 => "u16",
        TypeKind.U32 => "u32",
        TypeKind.U64 => "u64",
        TypeKind.F32 => "f32",
        TypeKind.F64 => "f64",
        TypeKind.ISize => "isize",
        TypeKind.USize => "usize",
        _ => "()"
    };

    /// <summary>
    /// 從字串解析原始類型
    /// </summary>
    /// <impl>
    /// APPROACH: 字串匹配
    /// CALLS: N/A
    /// EDGES: 未知類型返回 null
    /// </impl>
    public static PrimitiveType? FromString(string s) => s.ToLowerInvariant() switch
    {
        "void" or "()" => Void,
        "bool" or "_bool" => Bool,
        "i8" or "sbyte" or "int8_t" => I8,
        "i16" or "short" or "int16_t" => I16,
        "i32" or "int" or "int32_t" => I32,
        "i64" or "long" or "int64_t" => I64,
        "u8" or "byte" or "uint8_t" => U8,
        "u16" or "ushort" or "uint16_t" => U16,
        "u32" or "uint" or "uint32_t" => U32,
        "u64" or "ulong" or "uint64_t" => U64,
        "f32" or "float" => F32,
        "f64" or "double" => F64,
        "isize" or "nint" or "intptr_t" => ISize,
        "usize" or "nuint" or "size_t" => USize,
        _ => null
    };
}

/// <summary>
/// 指標類型
/// </summary>
/// <impl>
/// APPROACH: 封裝指標的目標類型、可空性、可變性
/// CALLS: Pointee.ToXxxType()
/// EDGES: N/A
/// </impl>
public sealed class PointerType : TypeRef
{
    public override TypeKind Kind => TypeKind.Ptr;
    public TypeRef Pointee { get; }
    public bool Nullable { get; }
    public bool Mutable { get; }

    public PointerType(TypeRef pointee, bool nullable = true, bool mutable = true)
    {
        Pointee = pointee;
        Nullable = nullable;
        Mutable = mutable;
    }

    public override string ToCType()
    {
        var constPrefix = Mutable ? "" : "const ";
        return $"{constPrefix}{Pointee.ToCType()}*";
    }

    public override string ToCSharpType()
    {
        return $"{Pointee.ToCSharpType()}*";
    }

    public override string ToRustType()
    {
        var mutability = Mutable ? "*mut" : "*const";
        return $"{mutability} {Pointee.ToRustType()}";
    }
}

/// <summary>
/// 陣列類型
/// </summary>
/// <impl>
/// APPROACH: 封裝元素類型和可選的固定長度
/// CALLS: Element.ToXxxType()
/// EDGES: Length 為 null 表示動態陣列
/// </impl>
public sealed class ArrayType : TypeRef
{
    public override TypeKind Kind => TypeKind.Array;
    public TypeRef Element { get; }
    public int? Length { get; }

    public ArrayType(TypeRef element, int? length = null)
    {
        Element = element;
        Length = length;
    }

    public override string ToCType()
    {
        return Length.HasValue
            ? $"{Element.ToCType()}[{Length}]"
            : $"{Element.ToCType()}*";
    }

    public override string ToCSharpType()
    {
        return Length.HasValue
            ? $"{Element.ToCSharpType()}*"  // 固定陣列用指標
            : $"{Element.ToCSharpType()}[]";
    }

    public override string ToRustType()
    {
        return Length.HasValue
            ? $"[{Element.ToRustType()}; {Length}]"
            : $"*mut {Element.ToRustType()}";
    }
}

/// <summary>
/// 結構類型引用
/// </summary>
/// <impl>
/// APPROACH: 只儲存結構名稱，實際定義在別處
/// CALLS: N/A
/// EDGES: N/A
/// </impl>
public sealed class StructType : TypeRef
{
    public override TypeKind Kind => TypeKind.Struct;
    public string Name { get; }

    public StructType(string name) => Name = name;

    public override string ToCType() => $"struct {Name}";
    public override string ToCSharpType() => Name;
    public override string ToRustType() => Name;
}

/// <summary>
/// 函數指標類型
/// </summary>
/// <impl>
/// APPROACH: 封裝函數簽名
/// CALLS: Signature.ToXxxType()
/// EDGES: N/A
/// </impl>
public sealed class FunctionPointerType : TypeRef
{
    public override TypeKind Kind => TypeKind.FnPtr;
    public FunctionSignature Signature { get; }

    public FunctionPointerType(FunctionSignature signature) => Signature = signature;

    public override string ToCType()
    {
        var ret = Signature.ReturnType.ToCType();
        var parms = string.Join(", ", Signature.Parameters.Select(p => p.Type.ToCType()));
        return $"{ret} (*fn)({parms})";
    }

    public override string ToCSharpType()
    {
        var ret = Signature.ReturnType.ToCSharpType();
        var parms = string.Join(", ", Signature.Parameters.Select(p => p.Type.ToCSharpType()));
        return $"delegate* unmanaged[Cdecl]<{parms}, {ret}>";
    }

    public override string ToRustType()
    {
        var ret = Signature.ReturnType.ToRustType();
        var parms = string.Join(", ", Signature.Parameters.Select(p => p.Type.ToRustType()));
        return $"extern \"C\" fn({parms}) -> {ret}";
    }
}

/// <summary>
/// 函數參數
/// </summary>
/// <impl>
/// APPROACH: 封裝參數名稱、類型、方向
/// CALLS: N/A
/// EDGES: Direction 預設 In
/// </impl>
public sealed class FunctionParameter
{
    public string Name { get; }
    public TypeRef Type { get; }
    public ParameterDirection Direction { get; }

    public FunctionParameter(string name, TypeRef type, ParameterDirection direction = ParameterDirection.In)
    {
        Name = name;
        Type = type;
        Direction = direction;
    }

    /// <summary>
    /// 從 JsonElement 解析參數
    /// </summary>
    /// <impl>
    /// APPROACH: 提取 name, type, direction 欄位
    /// CALLS: TypeRef.FromJsonElement()
    /// EDGES: 缺少必填欄位返回 null
    /// </impl>
    public static FunctionParameter? FromJsonElement(JsonElement element)
    {
        if (!element.TryGetProperty("name", out var nameProp) ||
            !element.TryGetProperty("type", out var typeProp))
        {
            return null;
        }

        var name = nameProp.GetString();
        if (string.IsNullOrEmpty(name))
        {
            return null;
        }

        var type = TypeRef.FromJsonElement(typeProp);
        if (type is null)
        {
            return null;
        }

        var direction = ParameterDirection.In;
        if (element.TryGetProperty("direction", out var dirProp))
        {
            direction = dirProp.GetString()?.ToLowerInvariant() switch
            {
                "out" => ParameterDirection.Out,
                "inout" => ParameterDirection.InOut,
                _ => ParameterDirection.In
            };
        }

        return new FunctionParameter(name, type, direction);
    }
}

/// <summary>
/// 參數方向
/// </summary>
public enum ParameterDirection
{
    In,
    Out,
    InOut
}

/// <summary>
/// 呼叫約定
/// </summary>
public enum CallingConvention
{
    Cdecl,
    Stdcall,
    Fastcall,
    Thiscall
}

/// <summary>
/// 函數簽名
/// </summary>
/// <impl>
/// APPROACH: 封裝參數列表、返回類型、呼叫約定
/// CALLS: N/A
/// EDGES: Convention 預設 Cdecl
/// </impl>
public sealed class FunctionSignature
{
    public IReadOnlyList<FunctionParameter> Parameters { get; }
    public TypeRef ReturnType { get; }
    public CallingConvention Convention { get; }

    public FunctionSignature(
        IReadOnlyList<FunctionParameter> parameters,
        TypeRef returnType,
        CallingConvention convention = CallingConvention.Cdecl)
    {
        Parameters = parameters;
        ReturnType = returnType;
        Convention = convention;
    }

    /// <summary>
    /// 從 JsonElement 解析簽名
    /// </summary>
    /// <impl>
    /// APPROACH: 提取 params, return, convention 欄位
    /// CALLS: FunctionParameter.FromJsonElement(), TypeRef.FromJsonElement()
    /// EDGES: 缺少必填欄位返回 null
    /// </impl>
    public static FunctionSignature? FromJsonElement(JsonElement element)
    {
        var parameters = new List<FunctionParameter>();

        if (element.TryGetProperty("params", out var paramsProp) &&
            paramsProp.ValueKind == JsonValueKind.Array)
        {
            foreach (var paramElem in paramsProp.EnumerateArray())
            {
                var param = FunctionParameter.FromJsonElement(paramElem);
                if (param is not null)
                {
                    parameters.Add(param);
                }
            }
        }

        TypeRef returnType = PrimitiveType.Void;
        if (element.TryGetProperty("return", out var returnProp))
        {
            var parsed = TypeRef.FromJsonElement(returnProp);
            if (parsed is not null)
            {
                returnType = parsed;
            }
        }

        var convention = CallingConvention.Cdecl;
        if (element.TryGetProperty("convention", out var convProp))
        {
            convention = convProp.GetString()?.ToLowerInvariant() switch
            {
                "stdcall" => CallingConvention.Stdcall,
                "fastcall" => CallingConvention.Fastcall,
                "thiscall" => CallingConvention.Thiscall,
                _ => CallingConvention.Cdecl
            };
        }

        return new FunctionSignature(parameters, returnType, convention);
    }
}

/// <summary>
/// 導出函數
/// </summary>
/// <impl>
/// APPROACH: 封裝函數名稱、簽名、屬性
/// CALLS: N/A
/// EDGES: Attributes 可為空
/// </impl>
public sealed class ExportedFunction
{
    public string Name { get; }
    public FunctionSignature Signature { get; }
    public IReadOnlyList<string> Attributes { get; }

    public ExportedFunction(
        string name,
        FunctionSignature signature,
        IReadOnlyList<string>? attributes = null)
    {
        Name = name;
        Signature = signature;
        Attributes = attributes ?? Array.Empty<string>();
    }

    /// <summary>
    /// 從 JsonElement 解析導出函數
    /// </summary>
    /// <impl>
    /// APPROACH: 提取 name, signature, attributes 欄位
    /// CALLS: FunctionSignature.FromJsonElement()
    /// EDGES: 缺少必填欄位返回 null
    /// </impl>
    public static ExportedFunction? FromJsonElement(JsonElement element)
    {
        if (!element.TryGetProperty("name", out var nameProp) ||
            !element.TryGetProperty("signature", out var sigProp))
        {
            return null;
        }

        var name = nameProp.GetString();
        if (string.IsNullOrEmpty(name))
        {
            return null;
        }

        var signature = FunctionSignature.FromJsonElement(sigProp);
        if (signature is null)
        {
            return null;
        }

        var attributes = new List<string>();
        if (element.TryGetProperty("attributes", out var attrProp) &&
            attrProp.ValueKind == JsonValueKind.Array)
        {
            foreach (var attr in attrProp.EnumerateArray())
            {
                var attrStr = attr.GetString();
                if (!string.IsNullOrEmpty(attrStr))
                {
                    attributes.Add(attrStr);
                }
            }
        }

        return new ExportedFunction(name, signature, attributes);
    }
}

/// <summary>
/// 模組接口描述
/// </summary>
/// <impl>
/// APPROACH: 完整的接口描述，包含模組資訊、導出、導入、類型定義
/// CALLS: N/A
/// EDGES: 所有集合可為空但不為 null
/// </impl>
public sealed record ModuleInterface
{
    public string Version { get; init; } = "1.0";
    public string ModuleName { get; init; } = string.Empty;
    public string ModuleVersion { get; init; } = "1.0.0";
    public string Language { get; init; } = string.Empty;
    public string Abi { get; init; } = "c";
    public string Mode { get; init; } = "native";
    public IReadOnlyList<ExportedFunction> Exports { get; init; } = Array.Empty<ExportedFunction>();
    public IReadOnlyList<string> Imports { get; init; } = Array.Empty<string>();

    /// <summary>
    /// 從 JSON 字串解析
    /// </summary>
    /// <impl>
    /// APPROACH: 解析頂層欄位和 exports 陣列
    /// CALLS: JsonDocument.Parse(), ExportedFunction.FromJsonElement()
    /// EDGES: JSON 格式錯誤返回 null
    /// </impl>
    public static ModuleInterface? FromJson(string json)
    {
        try
        {
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            var moduleInterface = new ModuleInterface();
            var exports = new List<ExportedFunction>();

            if (root.TryGetProperty("version", out var versionProp))
            {
                moduleInterface = moduleInterface with { Version = versionProp.GetString() ?? "1.0" };
            }

            if (root.TryGetProperty("module", out var moduleProp))
            {
                if (moduleProp.TryGetProperty("name", out var nameProp))
                {
                    moduleInterface = moduleInterface with { ModuleName = nameProp.GetString() ?? "" };
                }
                if (moduleProp.TryGetProperty("version", out var verProp))
                {
                    moduleInterface = moduleInterface with { ModuleVersion = verProp.GetString() ?? "1.0.0" };
                }
            }

            if (root.TryGetProperty("language", out var langProp))
            {
                if (langProp.TryGetProperty("name", out var langNameProp))
                {
                    moduleInterface = moduleInterface with { Language = langNameProp.GetString() ?? "" };
                }
                if (langProp.TryGetProperty("abi", out var abiProp))
                {
                    moduleInterface = moduleInterface with { Abi = abiProp.GetString() ?? "c" };
                }
                if (langProp.TryGetProperty("mode", out var modeProp))
                {
                    moduleInterface = moduleInterface with { Mode = modeProp.GetString() ?? "native" };
                }
            }

            if (root.TryGetProperty("exports", out var exportsProp) &&
                exportsProp.ValueKind == JsonValueKind.Array)
            {
                foreach (var exportElem in exportsProp.EnumerateArray())
                {
                    var export = ExportedFunction.FromJsonElement(exportElem);
                    if (export is not null)
                    {
                        exports.Add(export);
                    }
                }
            }

            return moduleInterface with { Exports = exports };
        }
        catch
        {
            return null;
        }
    }

    /// <summary>
    /// 序列化為 JSON
    /// </summary>
    /// <impl>
    /// APPROACH: 手動構建 JSON 結構
    /// CALLS: JsonSerializer.Serialize()
    /// EDGES: N/A
    /// </impl>
    public string ToJson()
    {
        var options = new JsonSerializerOptions
        {
            WriteIndented = true,
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase
        };

        var obj = new
        {
            version = Version,
            module = new { name = ModuleName, version = ModuleVersion },
            language = new { name = Language, abi = Abi, mode = Mode },
            exports = Exports.Select(e => new
            {
                name = e.Name,
                signature = new
                {
                    @params = e.Signature.Parameters.Select(p => new
                    {
                        name = p.Name,
                        type = SerializeType(p.Type),
                        direction = p.Direction.ToString().ToLowerInvariant()
                    }),
                    @return = SerializeType(e.Signature.ReturnType),
                    convention = e.Signature.Convention.ToString().ToLowerInvariant()
                },
                attributes = e.Attributes
            })
        };

        return JsonSerializer.Serialize(obj, options);
    }

    private static object SerializeType(TypeRef type)
    {
        return type switch
        {
            PrimitiveType p => new { kind = p.Kind.ToString().ToLowerInvariant() },
            PointerType ptr => new
            {
                kind = "ptr",
                pointee = SerializeType(ptr.Pointee),
                nullable = ptr.Nullable,
                mutable = ptr.Mutable
            },
            ArrayType arr => new
            {
                kind = "array",
                element = SerializeType(arr.Element),
                length = arr.Length
            },
            StructType s => new { kind = "struct", name = s.Name },
            _ => new { kind = "void" }
        };
    }
}
