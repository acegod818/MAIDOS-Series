// MAIDOS-Forge Studio - Main Window
// Code-QC v2.2B Compliant
// M12 - User Experience

using Avalonia.Controls;
using Avalonia.Input;

namespace Forge.Studio.Views;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        
        // Setup keyboard shortcuts
        KeyDown += OnKeyDown;
    }

    private void OnKeyDown(object? sender, KeyEventArgs e)
    {
        if (DataContext is not ViewModels.MainWindowViewModel vm)
            return;

        // F5 = Build
        if (e.Key == Key.F5 && vm.BuildCommand.CanExecute(null))
        {
            vm.BuildCommand.Execute(null);
            e.Handled = true;
        }
        
        // Ctrl+Shift+B = Rebuild
        if (e.Key == Key.B && e.KeyModifiers == (KeyModifiers.Control | KeyModifiers.Shift))
        {
            if (vm.RebuildCommand.CanExecute(null))
                vm.RebuildCommand.Execute(null);
            e.Handled = true;
        }
    }
}
