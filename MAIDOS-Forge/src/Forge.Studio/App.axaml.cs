// MAIDOS-Forge Studio - Application Class
// Code-QC v2.2B Compliant

using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Forge.Studio.Services;
using Forge.Studio.ViewModels;
using Forge.Studio.Views;

namespace Forge.Studio;

public partial class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            // Initialize services
            var forgeService = new ForgeService();
            var projectService = new ProjectService(forgeService);
            
            // Create main view model
            var mainViewModel = new MainWindowViewModel(forgeService, projectService);
            
            // Create and show main window
            desktop.MainWindow = new MainWindow
            {
                DataContext = mainViewModel
            };
        }

        base.OnFrameworkInitializationCompleted();
    }
}
