# MAIDOS-Forge Plugin Generator
# PowerShell Script for generating language plugin scaffolding

param(
    [Parameter(Mandatory=$true)]
    [string]$LanguageName,
    
    [Parameter(Mandatory=$true)]
    [string]$LanguageDisplayName,
    
    [string]$DefaultCompiler = "",
    [string]$PrimaryExtension = "",
    [string]$ObjectExtension = ".o",
    [string]$LibraryExtension = "a",
    [string]$DefaultStandard = "c99",
    [string[]]$Compilers = @(),
    [string[]]$SupportedExtensions = @(),
    [string[]]$SupportedTargets = @("linux", "windows", "macos"),
    
    [bool]$SupportsNativeCompilation = $True,
    [bool]$SupportsCrossCompilation = $True,
    [bool]$SupportsInterfaceExtraction = $True,
    [bool]$SupportsGlueGeneration = $True,
    [bool]$CreatesLibrary = $True,
    
    [string]$TemplateFile = "templates/GenericLanguagePlugin.cs",
    [string]$OutputDir = "src/Forge.Plugins/"
)

# Set default values based on language
switch ($LanguageName.ToLower()) {
    "cpp" {
        if (!$DefaultCompiler) { $DefaultCompiler = "g++" }
        if (!$PrimaryExtension) { $PrimaryExtension = ".cpp" }
        if (!$SupportedExtensions) { $SupportedExtensions = @(".cpp", ".cc", ".cxx", ".hpp", ".h") }
        if (!$Compilers) { $Compilers = @("g++", "clang++") }
        $DefaultStandard = "c++17"
    }
    "go" {
        if (!$DefaultCompiler) { $DefaultCompiler = "go" }
        if (!$PrimaryExtension) { $PrimaryExtension = ".go" }
        if (!$SupportedExtensions) { $SupportedExtensions = @(".go") }
        if (!$Compilers) { $Compilers = @("go") }
        $CreatesLibrary = $false
    }
    "python" {
        if (!$DefaultCompiler) { $DefaultCompiler = "python" }
        if (!$PrimaryExtension) { $PrimaryExtension = ".py" }
        if (!$SupportedExtensions) { $SupportedExtensions = @(".py") }
        if (!$Compilers) { $Compilers = @("python", "python3") }
        $SupportsNativeCompilation = $false
        $CreatesLibrary = $false
    }
    "javascript" {
        if (!$DefaultCompiler) { $DefaultCompiler = "node" }
        if (!$PrimaryExtension) { $PrimaryExtension = ".js" }
        if (!$SupportedExtensions) { $SupportedExtensions = @(".js") }
        if (!$Compilers) { $Compilers = @("node") }
        $SupportsNativeCompilation = $false
        $CreatesLibrary = $false
    }
    "typescript" {
        if (!$DefaultCompiler) { $DefaultCompiler = "tsc" }
        if (!$PrimaryExtension) { $PrimaryExtension = ".ts" }
        if (!$SupportedExtensions) { $SupportedExtensions = @(".ts") }
        if (!$Compilers) { $Compilers = @("tsc") }
        $SupportsNativeCompilation = $false
        $CreatesLibrary = $false
    }
    "java" {
        if (!$DefaultCompiler) { $DefaultCompiler = "javac" }
        if (!$PrimaryExtension) { $PrimaryExtension = ".java" }
        if (!$SupportedExtensions) { $SupportedExtensions = @(".java") }
        if (!$Compilers) { $Compilers = @("javac") }
    }
    default {
        if (!$DefaultCompiler) { $DefaultCompiler = $LanguageName }
        if (!$PrimaryExtension) { $PrimaryExtension = ".$LanguageName" }
        if (!$SupportedExtensions) { $SupportedExtensions = @(".$LanguageName") }
        if (!$Compilers) { $Compilers = @($LanguageName) }
    }
}

# Create plugin directory
$PluginDir = Join-Path $OutputDir "Forge.Plugin.$LanguageName"
Write-Host "Creating plugin directory: $PluginDir" -ForegroundColor Green

if (!(Test-Path $PluginDir)) {
    New-Item -ItemType Directory -Path $PluginDir -Force | Out-Null
}

# Read template
Write-Host "Reading template file: $TemplateFile" -ForegroundColor Yellow
$TemplateContent = Get-Content $TemplateFile -Raw

# Replace template variables
Write-Host "Generating plugin code..." -ForegroundColor Yellow

$ReplacedContent = $TemplateContent `
    -replace '\{\{LanguageName\}\}', $LanguageName `
    -replace '\{\{LanguageDisplayName\}\}', $LanguageDisplayName `
    -replace '\{\{DefaultCompiler\}\}', $DefaultCompiler `
    -replace '\{\{PrimaryExtension\}\}', $PrimaryExtension `
    -replace '\{\{ObjectExtension\}\}', $ObjectExtension `
    -replace '\{\{LibraryExtension\}\}', $LibraryExtension `
    -replace '\{\{DefaultStandard\}\}', $DefaultStandard `
    -replace '\{\{SupportsNativeCompilation\}\}', $SupportsNativeCompilation.ToString().ToLower() `
    -replace '\{\{SupportsCrossCompilation\}\}', $SupportsCrossCompilation.ToString().ToLower() `
    -replace '\{\{SupportsInterfaceExtraction\}\}', $SupportsInterfaceExtraction.ToString().ToLower() `
    -replace '\{\{SupportsGlueGeneration\}\}', $SupportsGlueGeneration.ToString().ToLower() `
    -replace '\{\{CreatesLibrary\}\}', $CreatesLibrary.ToString().ToLower()

# Replace arrays and lists
$SupportedExtensionsStr = $SupportedExtensions -join '", "'
$SupportedTargetsStr = $SupportedTargets -join '", "'
$CompilersStr = $Compilers -join '", "'

$ReplacedContent = $ReplacedContent -replace '\{\{SupportedExtensions\}\}', "`"$SupportedExtensionsStr`""
$ReplacedContent = $ReplacedContent -replace '\{\{SupportedTargets\}\}', "`"$SupportedTargetsStr`""
$ReplacedContent = $ReplacedContent -replace '\{\{CompilerCommands\}\}', "`"$CompilersStr`""

# Default values for optional features
$ReplacedContent = $ReplacedContent -replace '\{\{#if HasOutputOption\}\}(.*?)\{\{/if\}\}', '$1'
$ReplacedContent = $ReplacedContent -replace '\{\{#if HasStandardFlag\}\}(.*?)\{\{/if\}\}', '$1'
$ReplacedContent = $ReplacedContent -replace '\{\{#if HasWarningFlags\}\}(.*?)\{\{/if\}\}', '$1'
$ReplacedContent = $ReplacedContent -replace '\{\{#if SupportsPIC\}\}(.*?)\{\{/if\}\}', '$1'

# Default compiler flags (simplified)
$ReplacedContent = $ReplacedContent -replace '\{\{CompileBaseArgs\}\}', '-c'
$ReplacedContent = $ReplacedContent -replace '\{\{DebugOptimization\}\}', '-O0'
$ReplacedContent = $ReplacedContent -replace '\{\{ReleaseOptimization\}\}', '-O2'
$ReplacedContent = $ReplacedContent -replace '\{\{DebugFlags\}\}', '-g'
$ReplacedContent = $ReplacedContent -replace '\{\{WarningFlags\}\}', '-Wall -Wextra'
$ReplacedContent = $ReplacedContent -replace '\{\{StandardFlag\}\}', '-std'
$ReplacedContent = $ReplacedContent -replace '\{\{LibraryBaseArgs\}\}', 'rcs'

# Handle conditional template logic
if ($CreatesLibrary) {
    $ReplacedContent = $ReplacedContent -replace '\{\{#if CreatesLibrary\}\}(.*?)\{\{else\}\}(.*?)\{\{/if\}\}', '$1'
} else {
    $ReplacedContent = $ReplacedContent -replace '\{\{#if CreatesLibrary\}\}(.*?)\{\{else\}\}(.*?)\{\{/if\}\}', '$2'
}

# Remove remaining template tags
$ReplacedContent = $ReplacedContent -replace '\{\{#if.*?\}\}', ''
$ReplacedContent = $ReplacedContent -replace '\{\{/if\}\}', ''
$ReplacedContent = $ReplacedContent -replace '\{\{#each.*?\}\}', ''
$ReplacedContent = $ReplacedContent -replace '\{\{/each\}\}', ''
$ReplacedContent = $ReplacedContent -replace '\{\{else\}\}', ''

# Handle compiler commands string
$ReplacedContent = $ReplacedContent -replace '\{\{CompilerCommands\}\}', "$($Compilers -join ', ')"

# Handle unsupported template directives by replacing with defaults
$ReplacedContent = $ReplacedContent -replace '\{\{VersionCommand\}\}', '--version'
$ReplacedContent = $ReplacedContent -replace '\{\{#unless.*?\}\}', ''

# Write plugin file
$PluginFile = Join-Path $PluginDir "${LanguageName}Plugin.cs"
Write-Host "Writing plugin file: $PluginFile" -ForegroundColor Green
$ReplacedContent | Out-File -FilePath $PluginFile -Encoding UTF8

# Create plugin.json
$PluginJsonContent = @"
{
  "`"name`": `"forge.plugin.$LanguageName`",
  "`"version`": `"0.1.0`",
  "`"language`": `"$LanguageName`",
  "`"displayName`": `"$LanguageDisplayName Plugin`",
  "`"description`": `"$LanguageDisplayName language support for MAIDOS-Forge`",
  "`"author`": `"MAIDOS`",
  "`"extensions`": [`"$PrimaryExtension`"$(
    if ($SupportedExtensions.Length -gt 1) {
        $others = $SupportedExtensions | Where-Object { $_ -ne $PrimaryExtension }
        $others | ForEach-Object { ",`"$_`"" }
    }
  )],
  "`"toolchains`": [`"$DefaultCompiler`"$(
    if ($Compilers.Length -gt 1) {
        $others = $Compilers | Where-Object { $_ -ne $DefaultCompiler }
        $others | ForEach-Object { ",`"$_`"" }
    }
  )],
  "`"forgeVersion`": `">=0.7.0`",
  "`"entry`": `"Forge.Plugin.$LanguageName.dll`",
  "`"pluginClass`": `"Forge.Plugin.$LanguageName.${LanguageName}Plugin`",
  "`"dependencies`": []
}
"@

$PluginJsonFile = Join-Path $PluginDir "plugin.json"
Write-Host "Writing plugin.json: $PluginJsonFile" -ForegroundColor Green
$PluginJsonContent | Out-File -FilePath $PluginJsonFile -Encoding UTF8

# Create csproj file
$CsprojContent = @"
<Project Sdk=`"Microsoft.NET.Sdk`">

  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <AssemblyName>Forge.Plugin.$LanguageName</AssemblyName>
    <RootNamespace>Forge.Plugin.$LanguageName</RootNamespace>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include=`"..\..\src\Forge.Core\src\Forge.Core.New\Forge.Core.New.csproj`" />
  </ItemGroup>

</Project>
"@

$CsprojFile = Join-Path $PluginDir "Forge.Plugin.$LanguageName.csproj"
Write-Host "Writing csproj file: $CsprojFile" -ForegroundColor Green
$CsprojContent | Out-File -FilePath $CsprojFile -Encoding UTF8

# Create test file
$TestContent = @"
using Forge.Tests;

public class ${LanguageName}PluginTests
{
    public void Test_GetCapabilities()
    {
        var plugin = new Forge.Plugin.$LanguageName.${LanguageName}Plugin();
        var caps = plugin.GetCapabilities();

        Assert.Equal(`"$LanguageName`", caps.LanguageName);
        Assert.True(caps.SupportedExtensions.Contains(`"$PrimaryExtension`"));
    }

    public void Test_GenerateGlue_CSharp()
    {
        var plugin = new Forge.Plugin.$LanguageName.${LanguageName}Plugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = `"test`", Version = `"1.0.0`" },
            Language = new Forge.Core.Plugin.InterfaceLanguage { Name = `"$LanguageName`", Abi = `"c`" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, `"csharp`");
        Assert.True(result.IsSuccess);
    }

    public void Test_GenerateGlue_Rust()
    {
        var plugin = new Forge.Plugin.$LanguageName.${LanguageName}Plugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = `"test`", Version = `"1.0.0`" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, `"rust`");
        Assert.True(result.IsSuccess);
    }
}
"@

$TestFile = Join-Path $PluginDir "${LanguageName}PluginTests.cs"
Write-Host "Writing test file: $TestFile" -ForegroundColor Green
$TestContent | Out-File -FilePath $TestFile -Encoding UTF8

Write-Host "`nPlugin generation completed successfully!" -ForegroundColor Green
Write-Host "Files created:" -ForegroundColor Cyan
Write-Host "- $PluginFile" -ForegroundColor Cyan
Write-Host "- $PluginJsonFile" -ForegroundColor Cyan
Write-Host "- $CsprojFile" -ForegroundColor Cyan
Write-Host "- $TestFile" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. Add test methods to src/Forge.Tests/Program.cs" -ForegroundColor Yellow
Write-Host "2. Test the plugin using: dotnet build $PluginDir" -ForegroundColor Yellow
Write-Host "3. Run tests: dotnet test src/Forge.Tests/Forge.Tests.csproj" -ForegroundColor Yellow