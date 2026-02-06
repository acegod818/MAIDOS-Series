using Forge.Tests;

public class erlangPluginTests
{
    public void Test_GetCapabilities()
    {
        var plugin = new Forge.Plugin.erlang.erlangPlugin();
        var caps = plugin.GetCapabilities();

        Assert.Equal("erlang", caps.LanguageName);
        Assert.True(caps.SupportedExtensions.Contains(".erlang"));
    }

    public void Test_GenerateGlue_CSharp()
    {
        var plugin = new Forge.Plugin.erlang.erlangPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "test", Version = "1.0.0" },
            Language = new Forge.Core.Plugin.InterfaceLanguage { Name = "erlang", Abi = "c" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "csharp");
        Assert.True(result.IsSuccess);
    }

    public void Test_GenerateGlue_Rust()
    {
        var plugin = new Forge.Plugin.erlang.erlangPlugin();
        var iface = new Forge.Core.Plugin.InterfaceDescription
        {
            Module = new Forge.Core.Plugin.InterfaceModule { Name = "test", Version = "1.0.0" },
            Exports = Array.Empty<Forge.Core.Plugin.ExportedFunction>()
        };

        var result = plugin.GenerateGlue(iface, "rust");
        Assert.True(result.IsSuccess);
    }
}
