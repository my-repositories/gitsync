using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace GitSync;

public class Program
{
    private static IServiceCollection ConfigureServices(ConfigSettings cfg)
    {
        if (!Enum.TryParse<LogLevel>(cfg.LogLevel, ignoreCase: true, out var minLevel))
        {
            throw new InvalidOperationException($"Invalid log level: {cfg.LogLevel}");
        }

        return new ServiceCollection()
            .AddLogging(builder =>
            {
                builder.AddConsole();
                builder.SetMinimumLevel(minLevel);
            })
            .AddSingleton(cfg)
            .AddSingleton<GitSyncService>();
    }

    public static async Task Main()
    {
        var cfg = await new ConfigReader().ReadConfig();

        using var sp = ConfigureServices(cfg).BuildServiceProvider();
        var app = sp.GetRequiredService<GitSyncService>();
        await app.RunAsync();
    }
}
