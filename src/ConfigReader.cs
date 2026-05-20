using System.Text.Json;

namespace GitSync;

public class ConfigReader
{
    public async Task<ConfigSettings> ReadConfig()
    {
        var configPath = GetConfigPath();

        if (!File.Exists(configPath))
            throw new FileNotFoundException($"Config file not found: {configPath}");

        var json = await File.ReadAllTextAsync(configPath);
        var cfg = JsonSerializer.Deserialize<ConfigSettings>(json);

        return cfg ?? throw new InvalidOperationException("Failed to deserialize config.");
    }

    private static string GetConfigPath()
    {
        var path = Environment.GetEnvironmentVariable("GITSYNC_CONFIG_PATH");
        if (!string.IsNullOrWhiteSpace(path))
            return path;

        var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        return Path.Combine(home, ".gitsync", "config.json");
    }
}