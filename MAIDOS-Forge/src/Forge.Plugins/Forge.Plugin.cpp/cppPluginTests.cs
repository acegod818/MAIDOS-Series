using Forge.Tests;

public class cppPluginTests
{
    public void Test_GetCapabilities()
    {
        var plugin = new Forge.Plugin.cpp.cppPlugin();
        var caps = plugin.GetCapabilities();

        Assert.Equal("cpp", caps.LanguageName);
        Assert.True(caps.SupportedExtensions.Contains(".cpp"));
    }

    public void Test_GenerateGlue_CSharp()
    {
        var plugin = new Forge.Plugin.cpp.cppPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "test", Version = "1.0.0" },
            Language = new Forge.Core.Plugin.InterfaceLanguage { Name = "cpp", Abi = "c" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "csharp");
        Assert.True(result.IsSuccess);
    }

    public void Test_GenerateGlue_Rust()
    {
        var plugin = new Forge.Plugin.cpp.cppPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "test", Version = "1.0.0" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "rust");
        Assert.True(result.IsSuccess);
    }
}
