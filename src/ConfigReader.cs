namespace GitSync;

public interface IConfigReader
{
    Task<ConfigSettings> ReadConfig();
}

public class ConfigReader : IConfigReader
{
    public async Task<ConfigSettings> ReadConfig()
    {
        var cfg = new ConfigSettings
        {
            LogLevel = "Debug",
            RemoteBranchTemplate = "%owner%/%reponame%/%branchname%",
            RemoteUrls = new()
            {
                { "gitlab", "git@gitlab.com:loktionov129/gitsync.git" },
                { "gitverse", "git@gitverse.ru:loktionov129/gitsync.git" },
            },
        };

        return cfg;
    }    
}
