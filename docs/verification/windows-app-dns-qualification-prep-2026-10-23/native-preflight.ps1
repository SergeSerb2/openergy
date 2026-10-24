$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
if ((Get-Process -Id $PID).PriorityClass -ne 'BelowNormal') { throw 'priority gate' }
$msvc = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207'
$sdk = 'C:\Program Files (x86)\Windows Kits\10'
$version = '10.0.26100.0'
function Hash($p) { (Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLower() }
$required = @(
    (Join-Path $msvc 'bin\Hostx64\x64\cl.exe'),
    (Join-Path $msvc 'bin\Hostx64\x64\link.exe'),
    (Join-Path $msvc 'bin\Hostx64\x64\dumpbin.exe'),
    (Join-Path $sdk "Include\$version\um\WinDNS.h"),
    (Join-Path $sdk "Include\$version\shared\windnsdef.h"),
    (Join-Path $sdk "Lib\$version\um\x64\dnsapi.lib")
)
$inputs = @(foreach ($p in $required) {
    [ordered]@{path=$p; exists=(Test-Path -LiteralPath $p -PathType Leaf); sha256=$(if (Test-Path -LiteralPath $p -PathType Leaf) { Hash $p } else { $null })}
})
$extracts = @(foreach ($p in $required | Where-Object { $_ -match '\.h$' }) {
    if (Test-Path -LiteralPath $p -PathType Leaf) {
        $lines = [IO.File]::ReadAllLines($p)
        $selected = [Collections.Generic.SortedSet[int]]::new()
        for ($i = 0; $i -lt $lines.Length; ++$i) {
            if ($lines[$i] -match 'DNS_APPLICATION_SETTINGS|DNS_APP_SETTINGS_|DnsSetApplicationSettings|DnsGetApplicationSettings|DnsFreeCustomServers|_DNS_CUSTOM_SERVER|DNS_CUSTOM_SERVER_TYPE_UDP|DNS_ADDR_MAX_SOCKADDR_LENGTH') {
                for ($j = [Math]::Max(0, $i-8); $j -le [Math]::Min($lines.Length-1, $i+22); ++$j) { [void]$selected.Add($j) }
            }
        }
        [ordered]@{path=$p; lines=@(foreach ($i in $selected) { [ordered]@{line=$i+1; text=$lines[$i]} })}
    }
})
[ordered]@{
    timestamp_utc=[DateTime]::UtcNow.ToString('o'); priority=(Get-Process -Id $PID).PriorityClass.ToString()
    read_only=$true; settings_api_calls=0; query_api_calls=0; socket_binds=0
    inputs=$inputs; sdk_header_extracts=$extracts
    pwsh7_available=(Test-Path -LiteralPath 'C:\Program Files\PowerShell\7\pwsh.exe' -PathType Leaf)
    all_required_available=(@($inputs | Where-Object { -not $_.exists }).Count -eq 0)
} | ConvertTo-Json -Depth 10 -Compress
