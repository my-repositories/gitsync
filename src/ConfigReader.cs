using System;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;

namespace GitSync;

public class ConfigReader
{
    public async Task<ConfigSettings> ReadConfig()
    {
        var configPath = Environment.GetEnvironmentVariable("GITSYNC_CONFIG_PATH");

        if (string.IsNullOrWhiteSpace(configPath))
        {
            var home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            configPath = Path.Combine(home, ".gitsync", "config.json");
        }

        if (!File.Exists(configPath))
            throw new FileNotFoundException($"Config file not found: {configPath}");

        var json = await File.ReadAllTextAsync(configPath);
        var cfg = JsonSerializer.Deserialize<ConfigSettings>(json);

        return cfg ?? throw new InvalidOperationException("Failed to deserialize config.");
    }
}