$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$root = $PSScriptRoot
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
if ((Get-Process -Id $PID).PriorityClass -ne 'BelowNormal') { throw 'priority gate' }
if ((Get-Process -Id $PID).SessionId -ne 0) { throw 'noninteractive session 0 required' }
$guard = Join-Path $root 'native-build-once.guard'
$stream = [IO.File]::Open($guard, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
$stream.Dispose()
$msvc = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207'
$tools = Join-Path $msvc 'bin\Hostx64\x64'
$sdk = 'C:\Program Files (x86)\Windows Kits\10'
$version = '10.0.26100.0'
$temp = Join-Path $root 'temp'
[void](New-Item -ItemType Directory -Path $temp)
function Hash($p) { (Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLower() }
function Save($name, $data) { [IO.File]::WriteAllText((Join-Path $root $name), ($data | ConvertTo-Json -Depth 14), [Text.UTF8Encoding]::new($false)) }
$manifest = Get-Content -LiteralPath (Join-Path $root 'source-manifest.json') -Raw | ConvertFrom-Json
foreach ($name in @('qualify.c', 'qualify.manifest', 'native-build.ps1')) {
    $item = @($manifest.files | Where-Object { $_.path -eq $name })
    if ($item.Count -ne 1 -or (Hash (Join-Path $root $name)) -ne $item[0].sha256) { throw 'manifest source binding failed' }
}
$inputPaths = @(
    (Join-Path $root 'qualify.c'), (Join-Path $root 'qualify.manifest'), (Join-Path $root 'native-build.ps1'),
    (Join-Path $tools 'cl.exe'), (Join-Path $tools 'c1.dll'), (Join-Path $tools 'c2.dll'),
    (Join-Path $tools 'link.exe'), (Join-Path $tools 'dumpbin.exe'),
    (Join-Path $sdk "Include\$version\um\WinDNS.h"),
    (Join-Path $sdk "Include\$version\shared\windnsdef.h"),
    (Join-Path $sdk "Lib\$version\um\x64\dnsapi.lib"),
    (Join-Path $env:WINDIR 'System32\dnsapi.dll')
)
function Inputs { @(foreach ($p in $inputPaths) { [ordered]@{path=$p; sha256=(Hash $p)} }) }
$before = Inputs
foreach ($expected in $manifest.native_inputs) {
    if ((Hash $expected.path) -ne $expected.sha256) { throw 'installed prerequisite differs from read-only preflight' }
}
$watch = [Diagnostics.Stopwatch]::StartNew()
$processes = [Collections.Generic.List[object]]::new()
$record = [ordered]@{
    started_utc=[DateTime]::UtcNow.ToString('o'); root=$root
    parent_pid=$PID; parent_priority=(Get-Process -Id $PID).PriorityClass.ToString()
    session=(Get-Process -Id $PID).SessionId; sdk=$version; msvc='14.44.35207'
    jobs=1; jobs_limit=2; offline=$true; package_managers_used=$false
    generated_executable_runs=0; settings_api_calls=0; query_api_calls=0; socket_binds=0
    unit_tests=0; mocks=0; max_tool_seconds=8; total_build_budget_seconds=40
    inputs_before=$before; passed=$false
}
function Tool($label, $exe, $arguments) {
    if ($watch.Elapsed.TotalSeconds -gt 30) { throw 'build wall budget exhausted before next tool' }
    $info = [Diagnostics.ProcessStartInfo]::new($exe)
    $info.UseShellExecute=$false; $info.CreateNoWindow=$true; $info.WorkingDirectory=$root
    $info.RedirectStandardOutput=$true; $info.RedirectStandardError=$true
    foreach ($arg in $arguments) { $info.ArgumentList.Add($arg) }
    # Process-local build environment only. No vcvars, registry, rustup, cargo,
    # cache copy, global PATH mutation or reads of secrets/configuration.
    $info.Environment['INCLUDE'] = (Join-Path $msvc 'include')+';'+(Join-Path $sdk "Include\$version\ucrt")+';'+(Join-Path $sdk "Include\$version\shared")+';'+(Join-Path $sdk "Include\$version\um")
    $info.Environment['LIB'] = (Join-Path $msvc 'lib\x64')+';'+(Join-Path $sdk "Lib\$version\ucrt\x64")+';'+(Join-Path $sdk "Lib\$version\um\x64")
    $info.Environment['PATH'] = $tools+';'+(Join-Path $sdk "bin\$version\x64")+';'+(Join-Path $env:WINDIR 'System32')
    $info.Environment['TEMP']=$temp; $info.Environment['TMP']=$temp
    foreach ($name in @('CL','_CL_','LINK','_LINK_')) { [void]$info.Environment.Remove($name) }
    $child = [Diagnostics.Process]::Start($info)
    $entry = [ordered]@{label=$label; pid=$child.Id; executable=$exe; arguments=$arguments; timed_out=$false}
    $processes.Add($entry)
    $stdout=$child.StandardOutput.ReadToEndAsync(); $stderr=$child.StandardError.ReadToEndAsync()
    try {
        $entry.start_utc=$child.StartTime.ToUniversalTime().ToString('o')
        $entry.observed_priority=$child.PriorityClass.ToString()
        if ($entry.observed_priority -ne 'BelowNormal') { throw 'child effective priority mismatch' }
        $entry.timed_out = -not $child.WaitForExit(8000)
        if ($entry.timed_out) { throw 'native tool timeout; no retry' }
        $entry.exit_code=$child.ExitCode
    } finally {
        if (-not $child.HasExited) { $child.Kill($true); $child.WaitForExit(1000) | Out-Null; $entry.owned_tree_killed=$true }
        $entry.exited=$child.HasExited
        $outText=$stdout.GetAwaiter().GetResult(); $errText=$stderr.GetAwaiter().GetResult()
        if ($label -eq 'exports') {
            # Keep only the three public API symbols, not the entire DLL table.
            $outText=($outText -split "`n" | Where-Object { $_ -match '\bDns(SetApplicationSettings|GetApplicationSettings|FreeCustomServers)\b' }) -join "`n"
            $record.installed_dll_export_lines=$outText
        } else {
            [IO.File]::WriteAllText((Join-Path $root ($label+'.stdout')), $outText, [Text.UTF8Encoding]::new($false))
            [IO.File]::WriteAllText((Join-Path $root ($label+'.stderr')), $errText, [Text.UTF8Encoding]::new($false))
        }
        $child.Dispose()
    }
    if ($entry.exit_code -ne 0) { throw ($label+' failed; no retry') }
}
try {
    Tool 'compile' (Join-Path $tools 'cl.exe') @('/nologo','/std:c17','/W4','/WX','/Od','/Z7','/MT','/guard:cf','/c',('/Fo'+(Join-Path $root 'qualify.obj')),('/Fd'+(Join-Path $root 'qualify.pdb')),(Join-Path $root 'qualify.c'))
    Tool 'link' (Join-Path $tools 'link.exe') @('/nologo','/machine:x64','/subsystem:console','/incremental:no','/opt:noref','/dynamicbase','/nxcompat','/guard:cf','/manifest:embed',('/manifestinput:'+(Join-Path $root 'qualify.manifest')),('/out:'+(Join-Path $root 'qualify.exe')),(Join-Path $root 'qualify.obj'),'kernel32.lib','ws2_32.lib','iphlpapi.lib')
    Tool 'imports' (Join-Path $tools 'dumpbin.exe') @('/nologo','/imports',(Join-Path $root 'qualify.exe'))
    Tool 'exports' (Join-Path $tools 'dumpbin.exe') @('/nologo','/exports',(Join-Path $env:WINDIR 'System32\dnsapi.dll'))
    foreach ($symbol in @('DnsSetApplicationSettings','DnsGetApplicationSettings','DnsFreeCustomServers')) {
        if ($record.installed_dll_export_lines -notmatch ('\b'+$symbol+'\b')) { throw 'installed DLL lacks required export' }
    }
    $imports=Get-Content -LiteralPath (Join-Path $root 'imports.stdout') -Raw
    if ($imports -match '(?i)\b(getaddrinfo|GetAddrInfoW|GetAddrInfoEx\w*|gethostbyname|DnsQuery\w*|sendto|WSASendTo|recvfrom|WSARecvFrom)\b') { throw 'unexpected lookup or DNS I/O import' }
    $binary=Join-Path $root 'qualify.exe'
    $record.artifact=[ordered]@{path=$binary; bytes=(Get-Item -LiteralPath $binary).Length; sha256=(Hash $binary); executed=$false}
    $record.object=[ordered]@{bytes=(Get-Item -LiteralPath (Join-Path $root 'qualify.obj')).Length; sha256=(Hash (Join-Path $root 'qualify.obj'))}
    $record.inputs_after=Inputs
    $record.inputs_unchanged=($before | ConvertTo-Json -Compress) -eq ($record.inputs_after | ConvertTo-Json -Compress)
    if (-not $record.inputs_unchanged) { throw 'input bytes changed during compile' }
    $record.passed=$true
} catch {
    $record.failure=$_.Exception.Message
} finally {
    $record.processes=$processes.ToArray(); $record.elapsed_ms=$watch.ElapsedMilliseconds
    $record.all_owned_tool_handles_observed_exited=(@($processes | Where-Object { -not $_.exited }).Count -eq 0)
    $record.finished_utc=[DateTime]::UtcNow.ToString('o')
    Save 'build-result.json' $record
}
$acl=Get-Acl -LiteralPath $root
$files=@(foreach ($name in @('build-result.json','compile.stdout','compile.stderr','link.stdout','link.stderr','imports.stdout','imports.stderr')) {
    $path=Join-Path $root $name
    if (Test-Path -LiteralPath $path -PathType Leaf) { [ordered]@{name=$name; base64=[Convert]::ToBase64String([IO.File]::ReadAllBytes($path))} }
})
[ordered]@{
    root=$root; root_acl_protected=$acl.AreAccessRulesProtected
    root_acl_current_user_only=(@($acl.Access).Count -eq 1 -and $acl.Access[0].IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -eq [Security.Principal.WindowsIdentity]::GetCurrent().User.Value)
    build_passed=$record.passed; generated_executable_runs=0; evidence_files=$files
} | ConvertTo-Json -Depth 5 -Compress
# Transport success means evidence was returned, not that the build passed.
exit 0
