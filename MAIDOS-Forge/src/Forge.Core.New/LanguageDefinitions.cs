using System;
using System.Collections.Generic;

namespace Forge.Core.Plugin;

/// <summary>
/// Language category
/// </summary>
public enum LanguageCategory
{
    System,
    Managed,
    Scripting,
    Web,
    Functional,
    Mobile,
    Concurrent,
    Scientific,
    Hardware,
    Blockchain,
    Verification,
    Modern,
    Configuration,
    Logic,
    Markup,
    Other
}

/// <summary>
/// Language definition
/// </summary>
public sealed class LanguageDefinition
{
    public string Id { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public string DisplayName { get; set; } = string.Empty;
    public LanguageCategory Category { get; set; } = LanguageCategory.Other;
    public string[] Extensions { get; set; } = Array.Empty<string>();
    public string[] Toolchains { get; set; } = Array.Empty<string>();
    public string[] OutputTypes { get; set; } = Array.Empty<string>();
    public string Description { get; set; } = string.Empty;
    public bool IsBuiltin { get; set; }
}

/// <summary>
/// Global language definition registry - UEP v1.7C Compliant (95 Languages)
/// </summary>
public static class LanguageDefinitions
{
    private static readonly List<LanguageDefinition> _languages = new();

    static LanguageDefinitions()
    {
        // Tier 1: Mainstream programming languages (21)
        Add("c", "C", "C", LanguageCategory.System, new[] { ".c" }, new[] { "clang" }, true);
        Add("cpp", "C++", "C++", LanguageCategory.System, new[] { ".cpp" }, new[] { "clang++" }, true);
        Add("rust", "Rust", "Rust", LanguageCategory.System, new[] { ".rs" }, new[] { "cargo" }, true);
        Add("go", "Go", "Go", LanguageCategory.System, new[] { ".go" }, new[] { "go" }, true);
        Add("python", "Python", "Python", LanguageCategory.Scripting, new[] { ".py" }, new[] { "python3" }, true);
        Add("javascript", "JavaScript", "JavaScript", LanguageCategory.Web, new[] { ".js" }, new[] { "node" }, true);
        Add("typescript", "TypeScript", "TypeScript", LanguageCategory.Web, new[] { ".ts" }, new[] { "tsc" }, true);
        Add("java", "Java", "Java", LanguageCategory.Managed, new[] { ".java" }, new[] { "javac" }, true);
        Add("kotlin", "Kotlin", "Kotlin", LanguageCategory.Managed, new[] { ".kt" }, new[] { "kotlinc" }, true);
        Add("csharp", "C#", "C#", LanguageCategory.Managed, new[] { ".cs" }, new[] { "dotnet" }, true);
        Add("swift", "Swift", "Swift", LanguageCategory.Mobile, new[] { ".swift" }, new[] { "swiftc" }, false);
        Add("ruby", "Ruby", "Ruby", LanguageCategory.Scripting, new[] { ".rb" }, new[] { "ruby" }, false);
        Add("php", "PHP", "PHP", LanguageCategory.Web, new[] { ".php" }, new[] { "php" }, false);
        Add("scala", "Scala", "Scala", LanguageCategory.Managed, new[] { ".scala" }, new[] { "scalac" }, false);
        Add("haskell", "Haskell", "Haskell", LanguageCategory.Functional, new[] { ".hs" }, new[] { "ghc" }, false);
        Add("ocaml", "OCaml", "OCaml", LanguageCategory.Functional, new[] { ".ml" }, new[] { "ocamlopt" }, false);
        Add("fsharp", "F#", "F#", LanguageCategory.Functional, new[] { ".fs" }, new[] { "dotnet" }, false);
        Add("erlang", "Erlang", "Erlang", LanguageCategory.Concurrent, new[] { ".erl" }, new[] { "erlc" }, false);
        Add("elixir", "Elixir", "Elixir", LanguageCategory.Concurrent, new[] { ".ex" }, new[] { "elixir" }, false);
        Add("clojure", "Clojure", "Clojure", LanguageCategory.Functional, new[] { ".clj" }, new[] { "clojure" }, false);
        Add("objective-c", "Objective-C", "Objective-C", LanguageCategory.Mobile, new[] { ".m" }, new[] { "clang" }, false);

        // Tier 2: Specialized programming languages (33)
        Add("perl", "Perl", "Perl", LanguageCategory.Scripting, new[] { ".pl" }, new[] { "perl" }, false);
        Add("lua", "Lua", "Lua", LanguageCategory.Scripting, new[] { ".lua" }, new[] { "lua" }, false);
        Add("r", "R", "R", LanguageCategory.Scientific, new[] { ".r" }, new[] { "R" }, false);
        Add("julia", "Julia", "Julia", LanguageCategory.Scientific, new[] { ".jl" }, new[] { "julia" }, false);
        Add("matlab", "MATLAB", "MATLAB", LanguageCategory.Scientific, new[] { ".m" }, new[] { "matlab" }, false);
        Add("fortran", "Fortran", "Fortran", LanguageCategory.Scientific, new[] { ".f" }, new[] { "gfortran" }, false);
        Add("cobol", "COBOL", "COBOL", LanguageCategory.System, new[] { ".cob" }, new[] { "cobc" }, false);
        Add("pascal", "Pascal", "Pascal", LanguageCategory.System, new[] { ".pas" }, new[] { "fpc" }, false);
        Add("delphi", "Delphi", "Delphi", LanguageCategory.System, new[] { ".pas" }, new[] { "dcc32" }, false);
        Add("d", "D", "D", LanguageCategory.System, new[] { ".d" }, new[] { "dmd" }, false);
        Add("nim", "Nim", "Nim", LanguageCategory.System, new[] { ".nim" }, new[] { "nim" }, false);
        Add("zig", "Zig", "Zig", LanguageCategory.System, new[] { ".zig" }, new[] { "zig" }, false);
        Add("v", "V", "V", LanguageCategory.System, new[] { ".v" }, new[] { "v" }, false);
        Add("crystal", "Crystal", "Crystal", LanguageCategory.System, new[] { ".cr" }, new[] { "crystal" }, false);
        Add("dart", "Dart", "Dart", LanguageCategory.Mobile, new[] { ".dart" }, new[] { "dart" }, false);
        Add("groovy", "Groovy", "Groovy", LanguageCategory.Managed, new[] { ".groovy" }, new[] { "groovyc" }, false);
        Add("assembly-x86", "Assembly x86", "Assembly (x86)", LanguageCategory.Hardware, new[] { ".asm" }, new[] { "nasm" }, true);
        Add("assembly-arm", "Assembly ARM", "Assembly (ARM)", LanguageCategory.Hardware, new[] { ".s" }, new[] { "as" }, false);
        Add("vhdl", "VHDL", "VHDL", LanguageCategory.Hardware, new[] { ".vhdl" }, new[] { "ghdl" }, false);
        Add("verilog", "Verilog", "Verilog", LanguageCategory.Hardware, new[] { ".v" }, new[] { "iverilog" }, false);
        Add("sql-ansi", "SQL", "SQL (ANSI)", LanguageCategory.Logic, new[] { ".sql" }, new[] { "sqlcmd" }, false);
        Add("plsql", "PL/SQL", "PL/SQL", LanguageCategory.Logic, new[] { ".sql" }, new[] { "sqlplus" }, false);
        Add("tsql", "T-SQL", "T-SQL", LanguageCategory.Logic, new[] { ".sql" }, new[] { "sqlcmd" }, false);
        Add("bash", "Bash", "Bash", LanguageCategory.Scripting, new[] { ".sh" }, new[] { "bash" }, false);
        Add("powershell", "PowerShell", "PowerShell", LanguageCategory.Scripting, new[] { ".ps1" }, new[] { "pwsh" }, false);
        Add("zsh", "Zsh", "Zsh", LanguageCategory.Scripting, new[] { ".zsh" }, new[] { "zsh" }, false);
        Add("fish", "Fish", "Fish", LanguageCategory.Scripting, new[] { ".fish" }, new[] { "fish" }, false);
        Add("awk", "AWK", "AWK", LanguageCategory.Scripting, new[] { ".awk" }, new[] { "awk" }, false);
        Add("sed", "Sed", "Sed", LanguageCategory.Scripting, new[] { ".sed" }, new[] { "sed" }, false);
        Add("make", "Make", "Make", LanguageCategory.Configuration, new[] { "Makefile" }, new[] { "make" }, false);
        Add("sql-sqlite", "SQLite", "SQLite", LanguageCategory.Logic, new[] { ".db" }, new[] { "sqlite3" }, false);
        Add("sql-pg", "PostgreSQL", "PostgreSQL", LanguageCategory.Logic, new[] { ".sql" }, new[] { "psql" }, false);
        Add("sql-mysql", "MySQL", "MySQL", LanguageCategory.Logic, new[] { ".sql" }, new[] { "mysql" }, false);

        // Tier 3: Markup and configuration languages (41)
        Add("ada", "Ada", "Ada", LanguageCategory.System, new[] { ".adb" }, new[] { "gnat" }, false);
        Add("forth", "Forth", "Forth", LanguageCategory.System, new[] { ".fth" }, new[] { "gforth" }, false);
        Add("prolog", "Prolog", "Prolog", LanguageCategory.Logic, new[] { ".pl" }, new[] { "swipl" }, false);
        Add("common-lisp", "Common Lisp", "Common Lisp", LanguageCategory.Functional, new[] { ".lisp" }, new[] { "sbcl" }, false);
        Add("scheme", "Scheme", "Scheme", LanguageCategory.Functional, new[] { ".scm" }, new[] { "guile" }, false);
        Add("smalltalk", "Smalltalk", "Smalltalk", LanguageCategory.Managed, new[] { ".st" }, new[] { "gst" }, false);
        Add("apl", "APL", "APL", LanguageCategory.Scientific, new[] { ".apl" }, new[] { "apl" }, false);
        Add("j-lang", "J", "J", LanguageCategory.Scientific, new[] { ".j" }, new[] { "j" }, false);
        Add("k-lang", "K", "K", LanguageCategory.Scientific, new[] { ".k" }, new[] { "k" }, false);
        Add("q-lang", "Q", "Q", LanguageCategory.Scientific, new[] { ".q" }, new[] { "q" }, false);
        Add("factor", "Factor", "Factor", LanguageCategory.Functional, new[] { ".factor" }, new[] { "factor" }, false);
        Add("io-lang", "Io", "Io", LanguageCategory.Scripting, new[] { ".io" }, new[] { "io" }, false);
        Add("tcl", "Tcl", "Tcl", LanguageCategory.Scripting, new[] { ".tcl" }, new[] { "tclsh" }, false);
        Add("rexx", "Rexx", "Rexx", LanguageCategory.Scripting, new[] { ".rexx" }, new[] { "rexx" }, false);
        Add("abap", "ABAP", "ABAP", LanguageCategory.Managed, new[] { ".abap" }, new[] { "sap" }, false);
        Add("rpg", "RPG", "RPG", LanguageCategory.Managed, new[] { ".rpg" }, new[] { "ibm" }, false);
        Add("mumps", "MUMPS", "MUMPS", LanguageCategory.Managed, new[] { ".m" }, new[] { "mumps" }, false);
        Add("pli", "PL/I", "PL/I", LanguageCategory.System, new[] { ".pli" }, new[] { "pli" }, false);
        Add("natural", "NATURAL", "NATURAL", LanguageCategory.Managed, new[] { ".nat" }, new[] { "natural" }, false);
        Add("jcl", "JCL", "JCL", LanguageCategory.Configuration, new[] { ".jcl" }, new[] { "ibm" }, false);
        Add("cics", "CICS", "CICS", LanguageCategory.Managed, new[] { ".cics" }, new[] { "ibm" }, false);
        Add("sas", "SAS", "SAS", LanguageCategory.Scientific, new[] { ".sas" }, new[] { "sas" }, false);
        Add("stata", "Stata", "Stata", LanguageCategory.Scientific, new[] { ".do" }, new[] { "stata" }, false);
        Add("spss", "SPSS", "SPSS", LanguageCategory.Scientific, new[] { ".sps" }, new[] { "spss" }, false);
        Add("labview", "LabVIEW", "LabVIEW", LanguageCategory.Scientific, new[] { ".vi" }, new[] { "labview" }, false);
        Add("simulink", "Simulink", "Simulink", LanguageCategory.Scientific, new[] { ".slx" }, new[] { "matlab" }, false);
        Add("gams", "GAMS", "GAMS", LanguageCategory.Scientific, new[] { ".gms" }, new[] { "gams" }, false);
        Add("ampl", "AMPL", "AMPL", LanguageCategory.Scientific, new[] { ".ampl" }, new[] { "ampl" }, false);
        Add("wolfram", "Wolfram", "Wolfram", LanguageCategory.Scientific, new[] { ".nb" }, new[] { "wolfram" }, false);
        Add("postscript", "PostScript", "PostScript", LanguageCategory.Markup, new[] { ".ps" }, new[] { "gs" }, false);
        Add("latex", "LaTeX", "LaTeX", LanguageCategory.Markup, new[] { ".tex" }, new[] { "pdflatex" }, false);
        Add("markdown", "Markdown", "Markdown", LanguageCategory.Markup, new[] { ".md" }, new[] { "pandoc" }, false);
        Add("asciidoc", "AsciiDoc", "AsciiDoc", LanguageCategory.Markup, new[] { ".adoc" }, new[] { "asciidoctor" }, false);
        Add("graphql", "GraphQL", "GraphQL", LanguageCategory.Web, new[] { ".graphql" }, new[] { "graphql" }, false);
        Add("protobuf", "Protobuf", "Protobuf", LanguageCategory.Configuration, new[] { ".proto" }, new[] { "protoc" }, false);
        Add("thrift", "Thrift", "Thrift", LanguageCategory.Configuration, new[] { ".thrift" }, new[] { "thrift" }, false);
        Add("avro", "Avro", "Avro", LanguageCategory.Configuration, new[] { ".avsc" }, new[] { "avro" }, false);
        Add("wasm", "WASM", "WASM", LanguageCategory.Web, new[] { ".wasm" }, new[] { "wasm" }, false);
        Add("toml", "TOML", "TOML", LanguageCategory.Configuration, new[] { ".toml" }, new[] { "toml" }, false);
        Add("yaml", "YAML", "YAML", LanguageCategory.Configuration, new[] { ".yaml" }, new[] { "yaml" }, false);
        Add("json", "JSON", "JSON", LanguageCategory.Configuration, new[] { ".json" }, new[] { "json" }, false);

        // Total: 21 + 33 + 41 = 95 Languages
    }

    private static void Add(string id, string name, string disp, LanguageCategory cat, string[] exts, string[] tools, bool builtin)
    {
        _languages.Add(new LanguageDefinition
        {
            Id = id,
            Name = name,
            DisplayName = disp,
            Category = cat,
            Extensions = exts,
            Toolchains = tools,
            IsBuiltin = builtin,
            OutputTypes = new[] { ".o" }
        });
    }

    public static IReadOnlyList<LanguageDefinition> GetAllLanguages() => _languages;
}
