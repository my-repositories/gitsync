using System.Text;
using Microsoft.Extensions.Logging;

using GitSync.Configuration;

namespace GitSync.Services;

public interface IKnownHostsService
{
    Task WarmUpAsync();
}

public sealed class KnownHostsService : IKnownHostsService
{
    private readonly IProcessRunner _processRunner;
    private readonly ConfigSettings _cfg;
    private readonly ILogger<KnownHostsService> _logger;

    public KnownHostsService(
        IProcessRunner processRunner,
        ConfigSettings cfg,
        ILogger<KnownHostsService> logger)
    {
        _processRunner = processRunner;
        _cfg = cfg;
        _logger = logger;
    }

    public async Task WarmUpAsync()
    {
        var hosts = GetHosts();
        var knownHostsPath = GetKnownHostsPath();

        _logger.LogDebug("Known hosts warmup started for: {Hosts}", string.Join(", ", hosts));
        var existing = await LoadKnownHostsAsync(knownHostsPath);
        var updated = await BuildUpdatedKnownHostsAsync(existing, hosts);

        await SaveKnownHostsAsync(knownHostsPath, updated);

        _logger.LogDebug("Known hosts warmup completed. Total lines: {Count}", updated.Count);
    }

    private string[] GetHosts()
        => _cfg.RemoteUrls.Values
            .Select(ParseHost)
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();

    private static string GetKnownHostsPath()
    {
        var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        var sshDir = Path.Combine(home, ".ssh");
        Directory.CreateDirectory(sshDir);
        return Path.Combine(sshDir, "known_hosts");
    }

    private static async Task<List<string>> LoadKnownHostsAsync(string knownHostsPath)
    {
        if (!File.Exists(knownHostsPath))
            return [];

        return (await File.ReadAllLinesAsync(knownHostsPath))
            .Where(line => !string.IsNullOrWhiteSpace(line))
            .ToList();
    }

    private async Task<List<string>> BuildUpdatedKnownHostsAsync(List<string> existing, string[] hosts)
    {
        var preserved = existing
            .Where(line => !hosts.Any(host => LineMatchesHost(line, host)))
            .ToList();

        var scanned = await ScanHostsAsync(hosts);
        preserved.AddRange(scanned);

        return preserved.Distinct(StringComparer.Ordinal).ToList();
    }

    private async Task<List<string>> ScanHostsAsync(string[] hosts)
    {
        var result = new List<string>();

        foreach (var host in hosts)
        {
            _logger.LogDebug("Scanning host {Host}", host);

            var output = await _processRunner.RunAsync("ssh-keyscan", host);
            var lines = output
                .Split('\n', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
                .Where(LineIsValidKnownHostLine)
                .Where(line => LineMatchesHost(line, host));

            result.AddRange(lines);
        }

        return result;
    }

    private static Task SaveKnownHostsAsync(string knownHostsPath, List<string> lines)
        => File.WriteAllLinesAsync(knownHostsPath, lines, Encoding.UTF8);

    private static bool LineIsValidKnownHostLine(string line)
        => !string.IsNullOrWhiteSpace(line) && !line.StartsWith("#", StringComparison.Ordinal);

    private static bool LineMatchesHost(string line, string host)
        => line.StartsWith(host + " ", StringComparison.OrdinalIgnoreCase)
           || line.StartsWith(host + ",", StringComparison.OrdinalIgnoreCase);

    private static string ParseHost(string remoteUrl)
    {
        if (remoteUrl.StartsWith("git@", StringComparison.OrdinalIgnoreCase) && remoteUrl.Contains(':'))
            return remoteUrl.Split('@', 2)[1].Split(':', 2)[0];

        if (Uri.TryCreate(remoteUrl, UriKind.Absolute, out var uri))
            return uri.Host;

        throw new InvalidOperationException($"Invalid remote url: {remoteUrl}");
    }
}