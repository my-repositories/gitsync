using System.Diagnostics;
using Microsoft.Extensions.Logging;

namespace GitSync;

public sealed class GitSyncService
{
    private readonly ILogger<GitSyncService> _logger;
    private readonly ConfigSettings _cfg;

    public GitSyncService(ILogger<GitSyncService> logger, ConfigSettings cfg)
    {
        _logger = logger;
        _cfg = cfg;
    }

    public async Task RunAsync()
    {
        _logger.LogInformation("Start syncing");

        var sourceRemoteUrl = await StartProcessAsync
        (
            "git",
            $"remote get-url {_cfg.SourceRemoteName}"
        );
        var branchName = await StartProcessAsync("git", "branch --show-current");
        var remoteBranch = BuildRemoteBranch
        (
            _cfg.RemoteBranchTemplate,
            sourceRemoteUrl,
            branchName
        );
        var refspec = $"{branchName}:{remoteBranch}";

        foreach (var remote in _cfg.RemoteUrls)
        {
            await EnsureRemoteAsync(remote.Key, remote.Value);
        }

        var results = await Task.WhenAll(
            _cfg.RemoteUrls.Select(remote => PushRemoteAsync(remote, refspec))
        );

        var success = results.Count(r => r.Success);
        var error = results.Length - success;

        _logger.LogInformation("Success total: {Success}/{Total}", success, results.Length);
        _logger.LogInformation("Error total: {Error}/{Total}", error, results.Length);
    }

    private async Task<(string Name, bool Success)> PushRemoteAsync(KeyValuePair<string, string> remote, string refspec)
    {
        try
        {
            await StartProcessAsync("git", $"push {remote.Key} {refspec}");
            _logger.LogInformation("{Name} push successfully completed", remote.Key);
            return (remote.Key, true);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "{Name} failed to sync", remote.Key);
            return (remote.Key, false);
        }
    }

    private static string BuildRemoteBranch(string template, string originUrl, string branchName)
    {
        var repoPart = originUrl.Contains(':') ? originUrl.Split(':', 2)[1] : originUrl;
        repoPart = repoPart.EndsWith(".git", StringComparison.OrdinalIgnoreCase) ? repoPart[..^4] : repoPart;

        var parts = repoPart.Split('/', 2);
        if (parts.Length != 2)
            throw new InvalidOperationException($"Invalid origin url: {originUrl}");

        return template
            .Replace("%owner%", parts[0])
            .Replace("%reponame%", parts[1])
            .Replace("%branchname%", branchName);
    }

    private async Task EnsureRemoteAsync(string remoteName, string remoteUrl)
    {
        var existingUrl = await TryGetRemoteUrlAsync(remoteName);

        if (existingUrl is null)
        {
            _logger.LogInformation("{Name} adding remote url...", remoteName);
            await StartProcessAsync("git", $"remote add {remoteName} {remoteUrl}");
            _logger.LogInformation("{Name} remote url successfully added", remoteName);
            return;
        }

        if (string.Equals(existingUrl, remoteUrl, StringComparison.OrdinalIgnoreCase))
        {
            _logger.LogDebug("{Name} remote url already exists, skipping", remoteName);
            return;
        }

        _logger.LogInformation("{Name} updating remote url...", remoteName);
        await StartProcessAsync("git", $"remote set-url {remoteName} {remoteUrl}");
        _logger.LogInformation("{Name} remote url successfully updated", remoteName);
    }

    private async Task<string?> TryGetRemoteUrlAsync(string remoteName)
    {
        try
        {
            return await StartProcessAsync("git", $"remote get-url {remoteName}");
        }
        catch
        {
            return null;
        }
    }

    private static async Task<string> StartProcessAsync(string fileName, string arguments)
    {
        var psi = new ProcessStartInfo
        {
            FileName = fileName,
            Arguments = arguments,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        };

        using var process = Process.Start(psi)
            ?? throw new InvalidOperationException("Failed to start process.");

        var stdoutTask = process.StandardOutput.ReadToEndAsync();
        var stderrTask = process.StandardError.ReadToEndAsync();

        await process.WaitForExitAsync();

        var stdout = await stdoutTask;
        var stderr = await stderrTask;

        if (process.ExitCode != 0)
            throw new InvalidOperationException(string.IsNullOrWhiteSpace(stderr) ? stdout : stderr);

        return stdout.Trim();
    }
}