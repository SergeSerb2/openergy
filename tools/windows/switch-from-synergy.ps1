# Unelevated migration controller. Only the separately provisioned protected helper
# may be elevated; the per-user manager and supervisor always keep this token.
[CmdletBinding()]
param(
    [switch]$Install, [switch]$Start, [switch]$Commit, [switch]$Rollback, [switch]$CheckBroker,
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Openergy",
    [string]$DataDir = "$env:APPDATA\Openergy",
    [string]$BinaryPath = "$PSScriptRoot\openergy.exe",
    [string]$BrokerPath = ([IO.Path]::Combine([Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles),'OpenergyMigration','openergy-migration-admin.ps1'))
)
$ErrorActionPreference='Stop'
# Used only by unelevated per-user entry points. Never load user code as admin.
$oeIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
if ($oeIdentity.IsSystem -or ([Security.Principal.WindowsPrincipal]$oeIdentity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this per-user Openergy command from a non-administrator PowerShell. Only the protected migration helper may be elevated.'
}
if (-not ('OpenergyUserIntegrity' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Security.Principal;
public static class OpenergyUserIntegrity {
    [DllImport("advapi32.dll", SetLastError=true)] static extern bool GetTokenInformation(IntPtr token,int kind,IntPtr data,int length,out int needed);
    public static int Level() {
        using(var identity=WindowsIdentity.GetCurrent()) {
            int needed; GetTokenInformation(identity.Token,25,IntPtr.Zero,0,out needed);
            IntPtr data=Marshal.AllocHGlobal(needed);
            try {
                if(!GetTokenInformation(identity.Token,25,data,needed,out needed)) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
                var sid=new SecurityIdentifier(Marshal.ReadIntPtr(data));
                string[] parts=sid.Value.Split('-'); return int.Parse(parts[parts.Length-1]);
            } finally { Marshal.FreeHGlobal(data); }
        }
    }
}
'@
}
if ([OpenergyUserIntegrity]::Level() -gt 8192) { throw 'Per-user Openergy commands require a medium or lower integrity token.' }

# No module loading, script invocation, or compilation is needed to check trust.
function Get-MigrationAcl($Item) {
    if ($PSVersionTable.PSEdition -eq 'Core') { return [IO.FileSystemAclExtensions]::GetAccessControl($Item) }
    $Item.GetAccessControl()
}
function Assert-MigrationPath([string]$Path, [bool]$AdminLeaf=$true) {
    $full = [IO.Path]::GetFullPath($Path)
    if ($full -notmatch '^[A-Za-z]:\\' -or $full.IndexOf(':',2) -ge 0 -or $full.IndexOfAny([char[]]"`"`r`n`0") -ge 0) { throw 'Protected migration path must be a local drive path without alternate streams' }
    $item = if ([IO.Directory]::Exists($full)) { [IO.DirectoryInfo]::new($full) } else { [IO.FileInfo]::new($full) }
    if (!$item.Exists) { throw 'Protected migration helper or path is missing; provision the reviewed helper first' }
    $first = $true
    while ($item) {
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Protected migration path contains a reparse point' }
        $acl = Get-MigrationAcl $item
        $raw = [Security.AccessControl.RawSecurityDescriptor]::new($acl.GetSecurityDescriptorBinaryForm(),0)
        if ($null -eq $raw.DiscretionaryAcl) { throw 'Protected migration path has a null DACL' }
        $owner = $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value
        $trusted = @('S-1-5-18','S-1-5-32-544')
        $platform = 'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464'
        if ((!$first -or !$AdminLeaf) -and $owner -eq $platform) { } elseif ($owner -notin $trusted) { throw 'Protected migration path owner must be Administrators or SYSTEM (platform ancestors may be TrustedInstaller)' }
        # A protected child still can be replaced through DELETE_CHILD on a parent.
        # Adding a sibling alone cannot replace an existing protected child.
        $mask = if ($first) { [long]0x000D0156 } else { [long]0x000D0150 }
        foreach ($rule in $acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])) {
            if ($rule.AccessControlType -ne [Security.AccessControl.AccessControlType]::Allow -or ($rule.PropagationFlags -band [Security.AccessControl.PropagationFlags]::InheritOnly) -ne 0) { continue }
            $sid = $rule.IdentityReference.Value
            if ($sid -in ($trusted + @($platform,'S-1-3-4'))) { continue }
            $rights = [long]$rule.FileSystemRights -band 0xffffffffL
            if (($rights -band ($mask -bor 0x50000000L)) -ne 0) { throw 'Protected migration path grants mutation or replacement rights to a non-administrator principal' }
        }
        if ($item -is [IO.FileInfo]) { $item=$item.Directory } else { $item=$item.Parent }
        $first=$false
    }
    $full
}

if (@($Install,$Start,$Commit,$Rollback,$CheckBroker).Where({$_}).Count -ne 1) { throw 'Choose exactly one migration action or -CheckBroker' }
function Check-Broker {
    if([IO.Path]::GetFileName($BrokerPath) -ne 'openergy-migration-admin.ps1'){throw 'Protected helper must be named openergy-migration-admin.ps1'}
    $null=Assert-MigrationPath $BrokerPath
    $null=Assert-MigrationPath ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($BrokerPath)))
}

$Manager=if($Install){[IO.Path]::Combine($PSScriptRoot,'openergy.ps1')}else{[IO.Path]::Combine($InstallDir,'openergy.ps1')}
function Manage([string]$Action){& $Manager -Action $Action -InstallDir $InstallDir -DataDir $DataDir}
if($Install){& $Manager -Action Install -InstallDir $InstallDir -DataDir $DataDir -BinaryPath $BinaryPath;return}
$legacy=[IO.Path]::Combine($DataDir,'synergy-state.json')
if(!$CheckBroker -and [IO.File]::Exists($legacy)){throw 'Legacy writable migration state needs administrator review and recovery before using the protected migration helper; it will not be trusted or overwritten'}
$windows=[Environment]::GetFolderPath([Environment+SpecialFolder]::Windows)
$system32=[IO.Path]::Combine($windows,'System32')
$hostPath=[IO.Path]::Combine($system32,'WindowsPowerShell\v1.0\powershell.exe')
# Fixed Win32 declarations emitted in memory. No Add-Type/compiler/temp DLL/module.
function Initialize-MigrationNative {
    if ('OpenergyMigrationNative' -as [type]) { return }
    $name=[Reflection.AssemblyName]::new('OpenergyMigrationNative')
    $assembly=if($PSVersionTable.PSEdition -eq 'Core'){[Reflection.Emit.AssemblyBuilder]::DefineDynamicAssembly($name,[Reflection.Emit.AssemblyBuilderAccess]::Run)}else{[AppDomain]::CurrentDomain.DefineDynamicAssembly($name,[Reflection.Emit.AssemblyBuilderAccess]::Run)}
    $type=$assembly.DefineDynamicModule('Native').DefineType('OpenergyMigrationNative','Public,Sealed,Abstract')
    $definitions=@(
        @('kernel32.dll','GetNamedPipeServerProcessId',[bool],@([IntPtr],[uint32].MakeByRefType())),
        @('kernel32.dll','GetNamedPipeClientProcessId',[bool],@([IntPtr],[uint32].MakeByRefType())),
        @('kernel32.dll','OpenProcess',[IntPtr],@([uint32],[bool],[uint32])),
        @('kernel32.dll','CloseHandle',[bool],@([IntPtr])),
        @('kernel32.dll','GetProcessTimes',[bool],@([IntPtr],[long].MakeByRefType(),[long].MakeByRefType(),[long].MakeByRefType(),[long].MakeByRefType())),
        @('kernel32.dll','QueryFullProcessImageNameW',[bool],@([IntPtr],[uint32],[Text.StringBuilder],[uint32].MakeByRefType())),
        @('advapi32.dll','OpenProcessToken',[bool],@([IntPtr],[uint32],[IntPtr].MakeByRefType())),
        @('advapi32.dll','GetTokenInformation',[bool],@([IntPtr],[int],[IntPtr],[int],[int].MakeByRefType())),
        @('kernel32.dll','CreateNamedPipeW',[IntPtr],@([string],[uint32],[uint32],[uint32],[uint32],[uint32],[uint32],[IntPtr])),
        @('kernel32.dll','CreateFileW',[IntPtr],@([string],[uint32],[uint32],[IntPtr],[uint32],[uint32],[IntPtr])),
        @('kernel32.dll','WaitNamedPipeW',[bool],@([string],[uint32]))
    )
    foreach($definition in $definitions){
        $method=$type.DefinePInvokeMethod($definition[1],$definition[0],$definition[1],[Reflection.MethodAttributes]'Public,Static,PinvokeImpl',[Reflection.CallingConventions]::Standard,$definition[2],[Type[]]$definition[3],[Runtime.InteropServices.CallingConvention]::Winapi,[Runtime.InteropServices.CharSet]::Unicode)
        if($definition[1] -in @('CreateFileW','WaitNamedPipeW')){
            $attribute=[Reflection.Emit.CustomAttributeBuilder]::new([Runtime.InteropServices.DllImportAttribute].GetConstructor(@([string])),[object[]]@($definition[0]),[Reflection.FieldInfo[]]@([Runtime.InteropServices.DllImportAttribute].GetField('SetLastError'),[Runtime.InteropServices.DllImportAttribute].GetField('CharSet'),[Runtime.InteropServices.DllImportAttribute].GetField('ExactSpelling')),[object[]]@($true,[Runtime.InteropServices.CharSet]::Unicode,$true))
            $method.SetCustomAttribute($attribute)
        }
        $method.SetImplementationFlags([Reflection.MethodImplAttributes]::PreserveSig)
    }
    $null=$type.CreateType()
}
function Read-MigrationToken([IntPtr]$Token,[int]$Kind){
    $needed=0;$null=[OpenergyMigrationNative]::GetTokenInformation($Token,$Kind,[IntPtr]::Zero,0,[ref]$needed)
    if($needed -le 0 -or $needed -gt 1048576){throw 'Cannot query migration peer token'}
    $data=[Runtime.InteropServices.Marshal]::AllocHGlobal($needed)
    try{if(![OpenergyMigrationNative]::GetTokenInformation($Token,$Kind,$data,$needed,[ref]$needed)){throw 'Cannot query migration peer token'};return $data}catch{[Runtime.InteropServices.Marshal]::FreeHGlobal($data);throw}
}
function Get-MigrationProcess([int]$ProcessId){
    $process=[OpenergyMigrationNative]::OpenProcess(0x1000,$false,$ProcessId)
    if($process -eq [IntPtr]::Zero){throw 'Cannot open migration peer process'}
    $token=[IntPtr]::Zero
    try{
        $created=0L;$exited=0L;$kernel=0L;$user=0L
        if(![OpenergyMigrationNative]::GetProcessTimes($process,[ref]$created,[ref]$exited,[ref]$kernel,[ref]$user) -or $exited -ne 0){throw 'Migration peer is no longer the launched process'}
        $image=[Text.StringBuilder]::new(32768);$length=[uint32]32768
        if(![OpenergyMigrationNative]::QueryFullProcessImageNameW($process,0,$image,[ref]$length) -or ![OpenergyMigrationNative]::OpenProcessToken($process,8,[ref]$token)){throw 'Cannot query migration peer identity'}
        $data=Read-MigrationToken $token 1
        try{$sid=[Security.Principal.SecurityIdentifier]::new([Runtime.InteropServices.Marshal]::ReadIntPtr($data)).Value}finally{[Runtime.InteropServices.Marshal]::FreeHGlobal($data)}
        $data=Read-MigrationToken $token 25
        try{$label=[Security.Principal.SecurityIdentifier]::new([Runtime.InteropServices.Marshal]::ReadIntPtr($data)).Value;$integrity=[int]$label.Substring($label.LastIndexOf('-')+1)}finally{[Runtime.InteropServices.Marshal]::FreeHGlobal($data)}
        $admin=$false;$data=Read-MigrationToken $token 2
        try{
            $count=[Runtime.InteropServices.Marshal]::ReadInt32($data)
            $offset=if([IntPtr]::Size -eq 8){8}else{4};$stride=if([IntPtr]::Size -eq 8){16}else{8}
            for($index=0;$index -lt $count;$index++){
                $entry=[IntPtr]::Add($data,$offset+$stride*$index)
                $group=[Security.Principal.SecurityIdentifier]::new([Runtime.InteropServices.Marshal]::ReadIntPtr($entry)).Value
                $attributes=[Runtime.InteropServices.Marshal]::ReadInt32($entry,[IntPtr]::Size)
                if($group -eq 'S-1-5-32-544' -and ($attributes -band 4) -ne 0 -and ($attributes -band 16) -eq 0){$admin=$true}
            }
        }finally{[Runtime.InteropServices.Marshal]::FreeHGlobal($data)}
        @{pid=$ProcessId;started=[DateTime]::FromFileTimeUtc($created).Ticks.ToString();image=$image.ToString();sid=$sid;integrity=$integrity;admin=$admin}
    }finally{if($token -ne [IntPtr]::Zero){$null=[OpenergyMigrationNative]::CloseHandle($token)};$null=[OpenergyMigrationNative]::CloseHandle($process)}
}

Initialize-MigrationNative
$broker=$null;$pipe=$null
function Read-Broker {
    $bytes=[byte[]]::new(1);$text=[Text.StringBuilder]::new();$deadline=[DateTime]::UtcNow.AddSeconds(120)
    while($text.Length -lt 100){
        $remaining=[int]($deadline-[DateTime]::UtcNow).TotalMilliseconds
        if($remaining -le 0){throw 'Protected migration response deadline'}
        $task=$pipe.ReadAsync($bytes,0,1)
        if(!$task.Wait($remaining) -or $task.Result -eq 0){throw 'Protected migration helper closed or timed out; inspect protected recovery state'}
        if($bytes[0] -eq 10){$reply=$text.ToString();if($reply -eq 'ERROR|NO_STATE'){throw 'No saved migration state; refusing to change Synergy or Openergy'};if($reply -eq 'ERROR|DISABLED_RUNNING'){throw 'Synergy is running with startup type Disabled. Stop it or set its startup type to Demand/Manual or Automatic before retrying migration.'};if($reply.StartsWith('ERROR|')){throw 'Protected migration helper failed; recovery state remains available'};return $reply}
        if($bytes[0] -lt 32 -or $bytes[0] -gt 126){throw 'Invalid protected migration response'}
        $null=$text.Append([char]$bytes[0])
    }
    throw 'Protected migration response too long'
}
function Send-Broker([string]$Command){$bytes=[Text.Encoding]::ASCII.GetBytes($Command+"`n");if(!$pipe.WriteAsync($bytes,0,$bytes.Length).Wait(5000)){throw 'Protected migration request deadline'}}
function Connect-Broker([string]$Action){
    Check-Broker
    $null=Assert-MigrationPath $hostPath $false
    $name='OpenergyMigration-'+[guid]::NewGuid().ToString('N')
    $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $info=[Diagnostics.ProcessStartInfo]::new($hostPath)
    $info.UseShellExecute=($Action -ne 'Probe');if($Action -ne 'Probe'){$info.Verb='runas'};$info.WorkingDirectory=$system32
    $controller=Get-MigrationProcess $PID
    if($controller.image.IndexOfAny([char[]]"`"`r`n`0") -ge 0){throw 'Invalid controller image path'}
    $info.Arguments='-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "'+[IO.Path]::GetFullPath($BrokerPath)+'" -Action '+$Action+' -PipeName '+$name+' -UserSid '+$sid+' -ControllerPid '+$PID+' -ControllerStarted '+$controller.started+' -ControllerImage "'+$controller.image+'"'
    # RunAs uses ShellExecute. Remove inherited CLR/module injection settings before
    # it creates the fixed host; restore this ordinary process's environment after.
    $environment=@{}
    foreach($entry in [Environment]::GetEnvironmentVariables().GetEnumerator()){
        if($entry.Key -match '^(COR_|CORECLR_|COMPLUS_|DOTNET_)' -or $entry.Key -in @('PSModulePath','PSExecutionPolicyPreference','__COMPAT_LAYER','DEVPATH','PATH','SystemRoot')){$environment[$entry.Key]=$entry.Value;[Environment]::SetEnvironmentVariable($entry.Key,$null,'Process')}
    }
    try{
        [Environment]::SetEnvironmentVariable('PATH',$system32+';'+$windows,'Process')
        [Environment]::SetEnvironmentVariable('SystemRoot',$windows,'Process')
        $script:broker=[Diagnostics.Process]::Start($info)
    }finally{[Environment]::SetEnvironmentVariable('PATH',$null,'Process');[Environment]::SetEnvironmentVariable('SystemRoot',$null,'Process');foreach($entry in $environment.GetEnumerator()){[Environment]::SetEnvironmentVariable($entry.Key,$entry.Value,'Process')}}
    $launched=Get-MigrationProcess $broker.Id
    if($launched.image -ne $hostPath -or $launched.sid -ne $sid -or ($Action -eq 'Probe' -and ($launched.admin -or $launched.integrity -gt 8192)) -or ($Action -ne 'Probe' -and (!$launched.admin -or $launched.integrity -lt 12288))){throw 'Launched migration host token or image does not match'}
    # Generic write includes FILE_CREATE_PIPE_INSTANCE. Request only the matching
    # ReadWrite|Synchronize rights; never grant instance creation to the caller.
    # The handle constructor is available in both Windows PowerShell 5.1 and 7.
    $endpoint='\\.\pipe\'+$name
    $deadline=[DateTime]::UtcNow.AddSeconds(90)
    while($true){
        # OVERLAPPED | SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION; no inherit.
        $handle=[OpenergyMigrationNative]::CreateFileW($endpoint,0x0012019b,0,[IntPtr]::Zero,3,0x40110000,[IntPtr]::Zero)
        if($handle -ne [IntPtr]::new(-1)){
            $safe=[Microsoft.Win32.SafeHandles.SafePipeHandle]::new($handle,$true)
            try{$script:pipe=[IO.Pipes.NamedPipeClientStream]::new('InOut',$true,$true,$safe)}catch{$safe.Dispose();throw}
            break
        }
        $errorCode=[Runtime.InteropServices.Marshal]::GetLastWin32Error()
        if($errorCode -notin @(2,231)){throw [ComponentModel.Win32Exception]::new($errorCode)}
        $remaining=[int]($deadline-[DateTime]::UtcNow).TotalMilliseconds
        if($remaining -le 0){throw 'Protected migration connect deadline'}
        $null=[OpenergyMigrationNative]::WaitNamedPipeW($endpoint,[uint32][Math]::Min($remaining,200))
        # A missing endpoint makes WaitNamedPipe return immediately.
        [Threading.Thread]::Sleep([Math]::Min($remaining,20))
    }
    $serverId=[uint32]0
    if(![OpenergyMigrationNative]::GetNamedPipeServerProcessId($pipe.SafePipeHandle.DangerousGetHandle(),[ref]$serverId) -or $serverId -ne $broker.Id){throw 'Migration pipe server is not the launched protected host'}
    $connected=Get-MigrationProcess $serverId
    if($connected.started -ne $launched.started -or $connected.image -ne $launched.image -or $connected.sid -ne $launched.sid -or $connected.admin -ne $launched.admin -or $connected.integrity -ne $launched.integrity){throw 'Migration pipe server process identity changed'}
    Read-Broker
}
function Restore-Ui([string]$Reply){
    if($Reply -notin @('RESTORED|0','RESTORED|1')){throw 'Invalid protected restoration response'}
    if($Reply -eq 'RESTORED|1'){
        $exe=[IO.Path]::Combine([Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles),'Synergy','synergy-service.exe')
        $null=Assert-MigrationPath $exe
        $running=$false
        foreach($candidate in [Diagnostics.Process]::GetProcessesByName('synergy-service')){
            try{$peer=Get-MigrationProcess $candidate.Id;if($peer.image -eq $exe -and $peer.sid -eq [Security.Principal.WindowsIdentity]::GetCurrent().User.Value -and !$peer.admin -and $peer.integrity -le 8192){$running=$true}}catch{}finally{$candidate.Dispose()}
        }
        if(!$running){
            $info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.Arguments='-d';$info.WorkingDirectory=[IO.Path]::GetDirectoryName($exe)
            $ui=[Diagnostics.Process]::Start($info)
            try{$null=$ui.WaitForExit(3000)}finally{$ui.Dispose()}
            $found=$false
            foreach($candidate in [Diagnostics.Process]::GetProcessesByName('synergy-service')){
                try{$peer=Get-MigrationProcess $candidate.Id;if($peer.image -eq $exe -and $peer.sid -eq [Security.Principal.WindowsIdentity]::GetCurrent().User.Value -and !$peer.admin -and $peer.integrity -le 8192){$found=$true}}catch{}finally{$candidate.Dispose()}
            }
            if(!$found){throw 'Synergy UI did not start with the limited user token; protected recovery state remains for -Rollback'}
        }
    }
    Send-Broker 'UI_RESTORED'
    $completion=Read-Broker
    if($completion -notin @('RECOVERY_COMPLETE','RECOVERY_PENDING')){throw 'Protected restoration completion was not confirmed'}
}
try{
    if($CheckBroker){if((Connect-Broker 'Probe') -ne 'PROBE_OK'){throw 'Protected migration pipe probe failed'};'PROTECTED_MIGRATION_CHANNEL_VALID';return}
    if($Start){
        # Preserve the full per-user installation/endpoint preflight before cutover.
        $null=Manage 'Validate'
        if((Connect-Broker 'Start') -ne 'AVAILABLE'){throw 'Invalid protected migration start response'}
        Send-Broker 'BEGIN'
        if((Read-Broker) -ne 'STOPPED'){throw 'Protected helper did not confirm Synergy stop'}
        try{
            $status=Manage 'Start'
            if($status.state -notin @('listening','connected')){throw 'Openergy is not ready for migration'}
            Send-Broker 'READY'
            if((Read-Broker) -ne 'STARTED'){throw 'Protected helper did not confirm migration'}
            # Until this acknowledgement arrives, reply failure/caller loss must
            # restore the service and retain state. Start never deletes state.
            Send-Broker 'STARTED_ACK'
            $status
        }catch{
            $failure=$_;$stopFailed=$false
            try{$null=Manage 'Stop'}catch{$stopFailed=$true;Write-Warning "Openergy stop failed; protected restoration still runs. $($_.Exception.Message)"}
            try{Send-Broker $(if($stopFailed){'FAILED_STOP'}else{'FAILED'});Restore-Ui (Read-Broker)}catch{Write-Warning "Protected recovery state remains; run -Rollback. $($_.Exception.Message)"}
            throw $failure
        }
        return
    }
    if($Commit){
        $status=Manage 'Status'
        if($status.state -notin @('listening','connected')){throw 'Openergy must be ready before committing migration'}
        if((Connect-Broker 'Commit') -ne 'PENDING'){throw 'Invalid protected migration state response'}
        Send-Broker 'COMMIT'
        if((Read-Broker) -ne 'COMMITTED'){throw 'Protected helper did not confirm commit'}
        'Synergy automatic startup is disabled. Protected saved state remains available for -Rollback.'
        return
    }
    # Confirm protected saved state before attempting any per-user stop.
    if((Connect-Broker 'Rollback') -ne 'PENDING'){throw 'Invalid protected migration state response'}
    $stopFailure=$null
    try{$null=Manage 'Stop'}catch{$stopFailure=$_}
    Send-Broker $(if($stopFailure){'ROLLBACK_STOP_FAILED'}else{'ROLLBACK'})
    Restore-Ui (Read-Broker)
    if($stopFailure){Write-Warning 'Synergy was restored; Openergy stop failed and protected recovery state remains.';throw $stopFailure}
    'Saved Synergy service and startup state restored.'
}finally{
    # Closing a failed Start exchange makes the protected helper restore Synergy.
    # Never terminate an elevated helper while it may be restoring service state.
    if($pipe){$pipe.Dispose()}
    if($broker){$null=$broker.WaitForExit(5000);$broker.Dispose()}
}
