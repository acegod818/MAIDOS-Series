// MAIDOS-Forge Studio - Module ViewModel
// Code-QC v2.2B Compliant

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using Forge.Studio.Models;

namespace Forge.Studio.ViewModels;

public partial class ModuleViewModel : ViewModelBase
{
    [ObservableProperty]
    private string _name = "";

    [ObservableProperty]
    private string _language = "";

    [ObservableProperty]
    private string _path = "";

    [ObservableProperty]
    private ObservableCollection<string> _dependencies = new();

    [ObservableProperty]
    private double _x;

    [ObservableProperty]
    private double _y;

    [ObservableProperty]
    private bool _isSelected;

    public ModuleViewModel()
    {
    }

    public ModuleViewModel(ModuleInfo module)
    {
        Name = module.Name;
        Language = module.Language;
        Path = module.Path;
        
        foreach (var dep in module.Dependencies)
        {
            Dependencies.Add(dep);
        }
    }
}
