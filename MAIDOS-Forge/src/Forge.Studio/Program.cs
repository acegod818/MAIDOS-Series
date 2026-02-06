// MAIDOS-Forge Studio - GUI Application Entry Point
// Code-QC v2.2B Compliant
// M12 - User Experience

using System;
using Avalonia;
using Avalonia.ReactiveUI;

namespace Forge.Studio;

class Program
{
    [STAThread]
    public static void Main(string[] args) => BuildAvaloniaApp()
        .StartWithClassicDesktopLifetime(args);

    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .LogToTrace()
            .UseReactiveUI();
}
