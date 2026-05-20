using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

using GitSync.Configuration;
using GitSync.Services;

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
            .AddSingleton<IProcessRunner, ProcessRunner>()
            .AddSingleton<IGitSyncService, GitSyncService>()
            .AddSingleton<IKnownHostsService, KnownHostsService>();
    }

    public static async Task Main(string[] args)
    {
        var cfg = await new ConfigReader().ReadConfig();
        using var sp = ConfigureServices(cfg).BuildServiceProvider();
        var skipHost = args.Any(a =>
            string.Equals(a, "--skip-host", StringComparison.OrdinalIgnoreCase) ||
            string.Equals(a, "-s", StringComparison.OrdinalIgnoreCase));

        if (!skipHost)
        {
            var knownHostsService = sp.GetRequiredService<IKnownHostsService>();
            await knownHostsService.WarmUpAsync();
        }

        var gitSyncService = sp.GetRequiredService<IGitSyncService>();
        await gitSyncService.RunAsync();
    }
}
