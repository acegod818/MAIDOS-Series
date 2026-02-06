// MAIDOS-Forge Studio - FFI Inspector ViewModel
// Code-QC v2.2B Compliant

using System.Collections.ObjectModel;
using System.Linq;
using System.Text;
using Avalonia.Media;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace Forge.Studio.ViewModels;

public partial class FfiInspectorViewModel : ViewModelBase
{
    [ObservableProperty]
    private string? _selectedModule;

    [ObservableProperty]
    private FfiInterface? _selectedInterface;

    [ObservableProperty]
    private string? _selectedTargetLanguage;

    [ObservableProperty]
    private string _generatedCode = "";

    [ObservableProperty]
    private ObservableCollection<string> _modules = new();

    [ObservableProperty]
    private ObservableCollection<FfiInterface> _interfaces = new();

    [ObservableProperty]
    private ObservableCollection<string> _targetLanguages = new();

    public bool HasSelectedInterface => SelectedInterface != null;

    public FfiInspectorViewModel()
    {
        InitializeModules();
        InitializeTargetLanguages();
        
        // Load demo interfaces
        LoadDemoInterfaces();
    }

    private void InitializeModules()
    {
        Modules.Add("core (Rust)");
        Modules.Add("bindings (C#)");
        Modules.Add("native (C)");
        Modules.Add("scripts (Python)");
        
        SelectedModule = Modules[0];
    }

    private void InitializeTargetLanguages()
    {
        TargetLanguages.Add("C");
        TargetLanguages.Add("C#");
        TargetLanguages.Add("Python");
        TargetLanguages.Add("Rust");
        TargetLanguages.Add("Go");
        
        SelectedTargetLanguage = TargetLanguages[0];
    }

    private void LoadDemoInterfaces()
    {
        Interfaces.Clear();

        // Demo functions
        Interfaces.Add(new FfiInterface
        {
            Name = "calculate_hash",
            Kind = "function",
            KindLabel = "fn",
            KindColor = new SolidColorBrush(Color.Parse("#00B894")),
            ReturnType = "uint64_t",
            CallingConvention = "cdecl",
            Signature = "(data: *const u8, len: usize) -> u64",
            Parameters = new ObservableCollection<FfiParameter>
            {
                new("data", "*const u8"),
                new("len", "usize")
            }
        });

        Interfaces.Add(new FfiInterface
        {
            Name = "create_context",
            Kind = "function",
            KindLabel = "fn",
            KindColor = new SolidColorBrush(Color.Parse("#00B894")),
            ReturnType = "*mut Context",
            CallingConvention = "cdecl",
            Signature = "() -> *mut Context",
            Parameters = new ObservableCollection<FfiParameter>()
        });

        Interfaces.Add(new FfiInterface
        {
            Name = "process_buffer",
            Kind = "function",
            KindLabel = "fn",
            KindColor = new SolidColorBrush(Color.Parse("#00B894")),
            ReturnType = "int32_t",
            CallingConvention = "cdecl",
            Signature = "(ctx: *mut Context, buf: *mut u8, size: usize) -> i32",
            Parameters = new ObservableCollection<FfiParameter>
            {
                new("ctx", "*mut Context"),
                new("buf", "*mut u8"),
                new("size", "usize")
            }
        });

        Interfaces.Add(new FfiInterface
        {
            Name = "Context",
            Kind = "struct",
            KindLabel = "struct",
            KindColor = new SolidColorBrush(Color.Parse("#3498DB")),
            ReturnType = "-",
            CallingConvention = "-",
            Signature = "{ handle: u64, flags: u32 }",
            Parameters = new ObservableCollection<FfiParameter>
            {
                new("handle", "u64"),
                new("flags", "u32")
            }
        });

        Interfaces.Add(new FfiInterface
        {
            Name = "ErrorCode",
            Kind = "enum",
            KindLabel = "enum",
            KindColor = new SolidColorBrush(Color.Parse("#9B59B6")),
            ReturnType = "-",
            CallingConvention = "-",
            Signature = "{ Success = 0, InvalidInput = 1, OutOfMemory = 2 }",
            Parameters = new ObservableCollection<FfiParameter>
            {
                new("Success", "0"),
                new("InvalidInput", "1"),
                new("OutOfMemory", "2")
            }
        });

        if (Interfaces.Count > 0)
        {
            SelectedInterface = Interfaces[0];
        }
    }

    partial void OnSelectedInterfaceChanged(FfiInterface? value)
    {
        OnPropertyChanged(nameof(HasSelectedInterface));
        GenerateBindingCode();
    }

    partial void OnSelectedTargetLanguageChanged(string? value)
    {
        GenerateBindingCode();
    }

    [RelayCommand]
    private void Analyze()
    {
        // In real implementation, this would parse source files
        LoadDemoInterfaces();
    }

    private void GenerateBindingCode()
    {
        if (SelectedInterface == null || string.IsNullOrEmpty(SelectedTargetLanguage))
        {
            GeneratedCode = "";
            return;
        }

        var sb = new StringBuilder();
        var iface = SelectedInterface;

        switch (SelectedTargetLanguage)
        {
            case "C":
                GenerateCBinding(sb, iface);
                break;
            case "C#":
                GenerateCSharpBinding(sb, iface);
                break;
            case "Python":
                GeneratePythonBinding(sb, iface);
                break;
            case "Rust":
                GenerateRustBinding(sb, iface);
                break;
            case "Go":
                GenerateGoBinding(sb, iface);
                break;
        }

        GeneratedCode = sb.ToString();
    }

    private static void GenerateCBinding(StringBuilder sb, FfiInterface iface)
    {
        sb.AppendLine("/* Auto-generated C binding */");
        sb.AppendLine("#include <stdint.h>");
        sb.AppendLine();

        if (iface.Kind == "struct")
        {
            sb.AppendLine($"typedef struct {iface.Name} {{");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {MapToCType(param.Type)} {param.Name};");
            }
            sb.AppendLine($"}} {iface.Name};");
        }
        else if (iface.Kind == "enum")
        {
            sb.AppendLine($"typedef enum {iface.Name} {{");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {iface.Name}_{param.Name} = {param.Type},");
            }
            sb.AppendLine($"}} {iface.Name};");
        }
        else
        {
            var returnType = MapToCType(iface.ReturnType);
            var paramsList = string.Join(", ", iface.Parameters.Select(p => $"{MapToCType(p.Type)} {p.Name}"));
            if (string.IsNullOrEmpty(paramsList)) paramsList = "void";
            
            sb.AppendLine($"{returnType} {iface.Name}({paramsList});");
        }
    }

    private static void GenerateCSharpBinding(StringBuilder sb, FfiInterface iface)
    {
        sb.AppendLine("// Auto-generated C# binding");
        sb.AppendLine("using System.Runtime.InteropServices;");
        sb.AppendLine();

        if (iface.Kind == "struct")
        {
            sb.AppendLine("[StructLayout(LayoutKind.Sequential)]");
            sb.AppendLine($"public struct {iface.Name}");
            sb.AppendLine("{");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    public {MapToCSharpType(param.Type)} {param.Name};");
            }
            sb.AppendLine("}");
        }
        else if (iface.Kind == "enum")
        {
            sb.AppendLine($"public enum {iface.Name}");
            sb.AppendLine("{");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {param.Name} = {param.Type},");
            }
            sb.AppendLine("}");
        }
        else
        {
            sb.AppendLine("[DllImport(\"native\", CallingConvention = CallingConvention.Cdecl)]");
            var returnType = MapToCSharpType(iface.ReturnType);
            var paramsList = string.Join(", ", iface.Parameters.Select(p => $"{MapToCSharpType(p.Type)} {p.Name}"));
            sb.AppendLine($"public static extern {returnType} {iface.Name}({paramsList});");
        }
    }

    private static void GeneratePythonBinding(StringBuilder sb, FfiInterface iface)
    {
        sb.AppendLine("# Auto-generated Python binding");
        sb.AppendLine("import ctypes");
        sb.AppendLine();

        if (iface.Kind == "struct")
        {
            sb.AppendLine($"class {iface.Name}(ctypes.Structure):");
            sb.AppendLine("    _fields_ = [");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"        (\"{param.Name}\", {MapToPythonCtypes(param.Type)}),");
            }
            sb.AppendLine("    ]");
        }
        else if (iface.Kind == "enum")
        {
            sb.AppendLine($"class {iface.Name}:");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {param.Name} = {param.Type}");
            }
        }
        else
        {
            sb.AppendLine("lib = ctypes.CDLL(\"./libnative.so\")");
            sb.AppendLine();
            sb.AppendLine($"lib.{iface.Name}.restype = {MapToPythonCtypes(iface.ReturnType)}");
            sb.AppendLine($"lib.{iface.Name}.argtypes = [");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {MapToPythonCtypes(param.Type)},  # {param.Name}");
            }
            sb.AppendLine("]");
        }
    }

    private static void GenerateRustBinding(StringBuilder sb, FfiInterface iface)
    {
        sb.AppendLine("// Auto-generated Rust binding");
        sb.AppendLine();

        if (iface.Kind == "struct")
        {
            sb.AppendLine("#[repr(C)]");
            sb.AppendLine($"pub struct {iface.Name} {{");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    pub {param.Name}: {MapToRustType(param.Type)},");
            }
            sb.AppendLine("}");
        }
        else if (iface.Kind == "enum")
        {
            sb.AppendLine("#[repr(C)]");
            sb.AppendLine($"pub enum {iface.Name} {{");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {param.Name} = {param.Type},");
            }
            sb.AppendLine("}");
        }
        else
        {
            sb.AppendLine("extern \"C\" {");
            var returnType = MapToRustType(iface.ReturnType);
            var paramsList = string.Join(", ", iface.Parameters.Select(p => $"{p.Name}: {MapToRustType(p.Type)}"));
            sb.AppendLine($"    pub fn {iface.Name}({paramsList}) -> {returnType};");
            sb.AppendLine("}");
        }
    }

    private static void GenerateGoBinding(StringBuilder sb, FfiInterface iface)
    {
        sb.AppendLine("// Auto-generated Go binding");
        sb.AppendLine("package main");
        sb.AppendLine();
        sb.AppendLine("/*");
        sb.AppendLine("#include <stdint.h>");
        sb.AppendLine("*/");
        sb.AppendLine("import \"C\"");
        sb.AppendLine();

        if (iface.Kind == "struct")
        {
            sb.AppendLine($"type {iface.Name} struct {{");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {ToPascalCase(param.Name)} {MapToGoType(param.Type)}");
            }
            sb.AppendLine("}");
        }
        else if (iface.Kind == "enum")
        {
            sb.AppendLine($"type {iface.Name} int");
            sb.AppendLine("const (");
            foreach (var param in iface.Parameters)
            {
                sb.AppendLine($"    {iface.Name}{param.Name} {iface.Name} = {param.Type}");
            }
            sb.AppendLine(")");
        }
        else
        {
            var returnType = MapToGoType(iface.ReturnType);
            var paramsList = string.Join(", ", iface.Parameters.Select(p => $"{p.Name} {MapToGoType(p.Type)}"));
            sb.AppendLine($"func {ToPascalCase(iface.Name)}({paramsList}) {returnType} {{");
            sb.AppendLine($"    return {returnType}(C.{iface.Name}(/* params */))");
            sb.AppendLine("}");
        }
    }

    private static string MapToCType(string type) => type switch
    {
        "u8" or "uint8_t" => "uint8_t",
        "u16" or "uint16_t" => "uint16_t",
        "u32" or "uint32_t" => "uint32_t",
        "u64" or "uint64_t" => "uint64_t",
        "i8" or "int8_t" => "int8_t",
        "i16" or "int16_t" => "int16_t",
        "i32" or "int32_t" => "int32_t",
        "i64" or "int64_t" => "int64_t",
        "usize" => "size_t",
        "isize" => "ptrdiff_t",
        "*const u8" or "*mut u8" => "uint8_t*",
        "*mut Context" => "Context*",
        _ => type
    };

    private static string MapToCSharpType(string type) => type switch
    {
        "u8" or "uint8_t" => "byte",
        "u16" or "uint16_t" => "ushort",
        "u32" or "uint32_t" => "uint",
        "u64" or "uint64_t" => "ulong",
        "i8" or "int8_t" => "sbyte",
        "i16" or "int16_t" => "short",
        "i32" or "int32_t" => "int",
        "i64" or "int64_t" => "long",
        "usize" => "nuint",
        "isize" => "nint",
        "*const u8" or "*mut u8" => "IntPtr",
        "*mut Context" => "IntPtr",
        "void" => "void",
        _ => type
    };

    private static string MapToPythonCtypes(string type) => type switch
    {
        "u8" or "uint8_t" => "ctypes.c_uint8",
        "u16" or "uint16_t" => "ctypes.c_uint16",
        "u32" or "uint32_t" => "ctypes.c_uint32",
        "u64" or "uint64_t" => "ctypes.c_uint64",
        "i8" or "int8_t" => "ctypes.c_int8",
        "i16" or "int16_t" => "ctypes.c_int16",
        "i32" or "int32_t" => "ctypes.c_int32",
        "i64" or "int64_t" => "ctypes.c_int64",
        "usize" => "ctypes.c_size_t",
        "*const u8" or "*mut u8" => "ctypes.POINTER(ctypes.c_uint8)",
        "*mut Context" => "ctypes.c_void_p",
        "void" => "None",
        _ => "ctypes.c_void_p"
    };

    private static string MapToRustType(string type) => type switch
    {
        "uint8_t" => "u8",
        "uint16_t" => "u16",
        "uint32_t" => "u32",
        "uint64_t" => "u64",
        "int8_t" => "i8",
        "int16_t" => "i16",
        "int32_t" => "i32",
        "int64_t" => "i64",
        "size_t" => "usize",
        "uint8_t*" => "*mut u8",
        "Context*" => "*mut Context",
        _ => type
    };

    private static string MapToGoType(string type) => type switch
    {
        "u8" or "uint8_t" => "uint8",
        "u16" or "uint16_t" => "uint16",
        "u32" or "uint32_t" => "uint32",
        "u64" or "uint64_t" => "uint64",
        "i8" or "int8_t" => "int8",
        "i16" or "int16_t" => "int16",
        "i32" or "int32_t" => "int32",
        "i64" or "int64_t" => "int64",
        "usize" => "C.size_t",
        "*const u8" or "*mut u8" => "*C.uint8_t",
        "*mut Context" => "*C.Context",
        "void" => "",
        _ => "C." + type
    };

    private static string ToPascalCase(string input)
    {
        if (string.IsNullOrEmpty(input)) return input;
        var parts = input.Split('_');
        return string.Join("", parts.Select(p => 
            char.ToUpperInvariant(p[0]) + p[1..].ToLowerInvariant()));
    }
}

public class FfiInterface
{
    public string Name { get; set; } = "";
    public string Kind { get; set; } = "";
    public string KindLabel { get; set; } = "";
    public IBrush KindColor { get; set; } = Brushes.Gray;
    public string ReturnType { get; set; } = "";
    public string CallingConvention { get; set; } = "";
    public string Signature { get; set; } = "";
    public ObservableCollection<FfiParameter> Parameters { get; set; } = new();
}

public class FfiParameter
{
    public string Name { get; set; }
    public string Type { get; set; }

    public FfiParameter(string name, string type)
    {
        Name = name;
        Type = type;
    }
}
