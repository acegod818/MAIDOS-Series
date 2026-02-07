$ErrorActionPreference = "Continue"
$root = Split-Path $PSScriptRoot -Parent
$log = ""
$fail = 0
$plugins = @("C","cpp","CSharp","Rust","Go","Python","JavaScript","TypeScript","Java","Kotlin","Swift","Ruby","Dart","Haskell","Elixir")

foreach ($p in $plugins) {
    $csproj = Join-Path $root "src\Forge.Plugins\Forge.Plugin.$p\Forge.Plugin.$p.csproj"
    $r = dotnet build $csproj --nologo -v q 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) {
        $fail++
        $log += "[FAIL] $p`n$r`n"
    } else {
        $log += "[PASS] $p`n"
    }
}

$logPath = Join-Path $root "evidence\build_plugins.log"
$log | Set-Content -Path $logPath -Encoding UTF8
Write-Host "$($plugins.Count - $fail)/$($plugins.Count) passed, $fail failed"
exit $fail
