using Forge.Tests;

public class zigPluginTests
{
    public void Test_GetCapabilities()
    {
        var plugin = new Forge.Plugin.zig.zigPlugin();
        var caps = plugin.GetCapabilities();

        Assert.Equal("zig", caps.LanguageName);
        Assert.True(caps.SupportedExtensions.Contains(".zig"));
    }

    public void Test_GenerateGlue_CSharp()
    {
        var plugin = new Forge.Plugin.zig.zigPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "test", Version = "1.0.0" },
            Language = new Forge.Core.Plugin.InterfaceLanguage { Name = "zig", Abi = "c" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "csharp");
        Assert.True(result.IsSuccess);
    }

    public void Test_GenerateGlue_Rust()
    {
        var plugin = new Forge.Plugin.zig.zigPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "test", Version = "1.0.0" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "rust");
        Assert.True(result.IsSuccess);
    }
}
