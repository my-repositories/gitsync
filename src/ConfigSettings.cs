namespace GitSync;

public class ConfigSettings
{
    public required string LogLevel { get; set; }

    public required string RemoteBranchTemplate { get; set; }
    
    public required Dictionary<string, string> RemoteUrls { get; set;}

    public required string SourceRemoteName { get; set; }
}
