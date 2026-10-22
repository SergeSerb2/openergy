# Privileged fixed Synergy operations. Provision reviewed bytes into an admin-owned
# protected directory. Never run this file elevated from a checkout or LocalAppData.
[CmdletBinding()]
param(
    [ValidateSet('Check','Probe','Start','Commit','Rollback')][string]$Action='Check',
    [ValidatePattern('^OpenergyMigration-[0-9a-f]{32}$')][string]$PipeName,
    [string]$CheckPath,
    [string]$UserSid,
    [int]$ControllerPid, [string]$ControllerStarted, [string]$ControllerImage
)
$ErrorActionPreference='Stop'
$PSModuleAutoLoadingPreference='None'
$env:PSModulePath=''
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

$null=Assert-MigrationPath $PSCommandPath
$null=Assert-MigrationPath $PSScriptRoot
if ($Action -eq 'Check') {
    if($CheckPath){$null=Assert-MigrationPath $CheckPath;$null=Assert-MigrationPath ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($CheckPath)))}
    'PROTECTED_MIGRATION_HELPER_VALID'; return
}
if($CheckPath){throw 'CheckPath is only available for read-only trust checks'}
$windows=[Environment]::GetFolderPath([Environment+SpecialFolder]::Windows)
$system32=[IO.Path]::Combine($windows,'System32')
$hostPath=[IO.Path]::Combine($system32,'WindowsPowerShell\v1.0\powershell.exe')
if ($PSVersionTable.PSEdition -eq 'Core' -or ![String]::Equals([Diagnostics.Process]::GetCurrentProcess().MainModule.FileName,$hostPath,[StringComparison]::OrdinalIgnoreCase) -or [Environment]::GetCommandLineArgs() -notcontains '-NoProfile') { throw 'Migration operations require the fixed Windows PowerShell host with -NoProfile' }
$null=Assert-MigrationPath $hostPath $false
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
        @('kernel32.dll','CreateNamedPipeW',[IntPtr],@([string],[uint32],[uint32],[uint32],[uint32],[uint32],[uint32],[IntPtr]))
    )
    foreach($definition in $definitions){
        $method=$type.DefinePInvokeMethod($definition[1],$definition[0],$definition[1],[Reflection.MethodAttributes]'Public,Static,PinvokeImpl',[Reflection.CallingConventions]::Standard,$definition[2],[Type[]]$definition[3],[Runtime.InteropServices.CallingConvention]::Winapi,[Runtime.InteropServices.CharSet]::Unicode)
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
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if (($Action -ne 'Probe' -and !([Security.Principal.WindowsPrincipal]$identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) -or !$UserSid -or $UserSid -ne $identity.User.Value -or !$PipeName -or $ControllerPid -le 0 -or !$ControllerStarted -or !$ControllerImage) { throw 'Migration requires elevation as the same Windows user and an explicit control pipe' }
$null=[Security.Principal.SecurityIdentifier]::new($UserSid)
$serviceName='Synergy Core Daemon'
$synergy=[IO.Path]::Combine([Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles),'Synergy')
$serviceKey='SYSTEM\CurrentControlSet\Services\Synergy Core Daemon'
$approvalKey='Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run'
$runName='Synergy 3 Service'
$stateRoot=[IO.Path]::Combine($PSScriptRoot,'State',$UserSid)
$stateFile=[IO.Path]::Combine($stateRoot,'synergy-state.v2')
$lock=$null; $pipe=$null; $saved=$null; $cutover=$false
function Private-Directory([string]$Path) {
    if (![IO.Directory]::Exists($Path)) {
        $acl=[Security.AccessControl.DirectorySecurity]::new()
        $acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
        $acl.SetAccessRuleProtection($true,$false)
        foreach($sid in @('S-1-5-18','S-1-5-32-544')) { $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','ContainerInherit,ObjectInherit','None','Allow')) }
        $null=[IO.Directory]::CreateDirectory($Path,$acl)
    }
    $null=Assert-MigrationPath $Path
}
function Open-Private([string]$Path,[IO.FileMode]$Mode) {
    $acl=[Security.AccessControl.FileSecurity]::new()
    $acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
    $acl.SetAccessRuleProtection($true,$false)
    foreach($sid in @('S-1-5-18','S-1-5-32-544')) { $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','Allow')) }
    [IO.FileStream]::new($Path,$Mode,[Security.AccessControl.FileSystemRights]::Write,[IO.FileShare]::None,4096,[IO.FileOptions]::WriteThrough,$acl)
}
function Lock-State {
    Private-Directory ([IO.Path]::Combine($PSScriptRoot,'State'))
    Private-Directory $stateRoot
    $path=[IO.Path]::Combine($stateRoot,'migration.lock')
    if ([IO.File]::Exists($path)) { $null=Assert-MigrationPath $path; return [IO.File]::Open($path,'Open','Write','None') }
    Open-Private $path ([IO.FileMode]::CreateNew)
}
function Read-State {
    if (![IO.File]::Exists($stateFile)) { throw 'NO_STATE' }
    $null=Assert-MigrationPath $stateFile
    if ([IO.FileInfo]::new($stateFile).Length -gt 8192) { throw 'Invalid protected migration state' }
    $lines=[IO.File]::ReadAllLines($stateFile)
    if ($lines.Length -ne 8 -or $lines[0] -ne '2' -or $lines[1] -ne $UserSid -or $lines[2] -notin @('2','3','4') -or $lines[3] -notin @('0','1') -or $lines[4] -notin @('0','1')) { throw 'Invalid protected migration state' }
    $delayed=if($lines[5] -eq '-'){$null}else{[uint32]::Parse($lines[5])}
    $approved=$null
    if($lines[6] -ne '-'){$approved=[Convert]::FromBase64String($lines[6])}
    if ($lines[7] -ne 'END' -or ($null -ne $approved -and $approved.Length -gt 4096)) { throw 'Invalid protected migration state' }
    @{start=[int]$lines[2];running=($lines[3] -eq '1');ui=($lines[4] -eq '1');delayed=$delayed;approved=$approved}
}
function Save-State($Value) {
    $lines=@('2',$UserSid,[string]$Value.start,([int][bool]$Value.running).ToString(),([int][bool]$Value.ui).ToString(),$(if($null -eq $Value.delayed){'-'}else{[string]$Value.delayed}),$(if($null -eq $Value.approved){'-'}else{[Convert]::ToBase64String($Value.approved)}),'END')
    $file=Open-Private $stateFile ([IO.FileMode]::CreateNew)
    try { $bytes=[Text.Encoding]::UTF8.GetBytes(($lines -join "`n")+"`n"); $file.Write($bytes,0,$bytes.Length); $file.Flush($true) } finally { $file.Dispose() }
}
function Load-Service {
    $assembly=[IO.Path]::Combine([Runtime.InteropServices.RuntimeEnvironment]::GetRuntimeDirectory(),'System.ServiceProcess.dll')
    $null=Assert-MigrationPath $assembly $false
    $null=[Reflection.Assembly]::LoadFrom($assembly)
    $null=Assert-MigrationPath $synergy
    foreach($name in @('synergy-core','synergy-service','synergy-tray')) { $path=[IO.Path]::Combine($synergy,$name+'.exe'); if([IO.File]::Exists($path)){$null=Assert-MigrationPath $path} }
    $key=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($serviceKey)
    try {
        if(!$key){throw 'Synergy service is not installed'}
        $image=[string]$key.GetValue('ImagePath','',[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $exe=if($image -match '^"([^"\r\n]+)"(?:\s|$)'){$matches[1]}elseif($image -match '^([^\s"\r\n]+)(?:\s|$)'){$matches[1]}else{throw 'Invalid Synergy service ImagePath'}
        if($exe -notin @([IO.Path]::Combine($synergy,'synergy-service.exe'),[IO.Path]::Combine($synergy,'synergy-core.exe'))){throw 'Synergy service executable is outside the protected installation'}
        $null=Assert-MigrationPath $exe
    } finally { if($key){$key.Dispose()} }
    [System.ServiceProcess.ServiceController]::new($serviceName)
}
function Set-Start([int]$Value) {
    $mode=switch($Value){2{'auto'}3{'demand'}4{'disabled'}default{throw 'Unsupported saved service start mode'}}
    $exe=[IO.Path]::Combine($system32,'sc.exe');$null=Assert-MigrationPath $exe $false
    $info=[Diagnostics.ProcessStartInfo]::new($exe)
    $info.UseShellExecute=$false;$info.WorkingDirectory=$system32
    $info.Arguments='config "Synergy Core Daemon" start= '+$mode
    $process=[Diagnostics.Process]::Start($info)
    try { if(!$process.WaitForExit(15000) -or $process.ExitCode -ne 0){throw 'Synergy service start-mode change failed'} } finally { $process.Dispose() }
}
function Restore-Service($Value) {
    $service=Load-Service
    try {
        Set-Start $Value.start
        $key=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($serviceKey,$true)
        try { if($null -eq $Value.delayed){$key.DeleteValue('DelayedAutostart',$false)}else{$key.SetValue('DelayedAutostart',[BitConverter]::ToInt32([BitConverter]::GetBytes([uint32]$Value.delayed),0),[Microsoft.Win32.RegistryValueKind]::DWord)} } finally {$key.Dispose()}
        $key=[Microsoft.Win32.Registry]::LocalMachine.CreateSubKey($approvalKey)
        try { if($null -eq $Value.approved){$key.DeleteValue($runName,$false)}else{$key.SetValue($runName,[byte[]]$Value.approved,[Microsoft.Win32.RegistryValueKind]::Binary)} } finally {$key.Dispose()}
        $service.Refresh()
        if($Value.running){
            if($service.Status -ne [System.ServiceProcess.ServiceControllerStatus]::Running){$service.Start();$service.WaitForStatus('Running',[TimeSpan]::FromSeconds(20))}
        }elseif($service.Status -ne [System.ServiceProcess.ServiceControllerStatus]::Stopped){
            if($service.Status -ne [System.ServiceProcess.ServiceControllerStatus]::StopPending){$service.Stop()}
            $service.WaitForStatus('Stopped',[TimeSpan]::FromSeconds(20))
        }
    } finally {$service.Dispose()}
}
function Complete-Restoration($Value,[bool]$KeepState) {
    Reply ('RESTORED|'+[int][bool]$Value.ui)
    # Keep the record on caller loss, UI failure, or an unsuccessful Openergy stop.
    if((Read-Command) -ne 'UI_RESTORED'){throw 'Limited UI restoration was not acknowledged; protected recovery state retained'}
    if(!$KeepState){[IO.File]::Delete($stateFile)}
    Reply $(if($KeepState){'RECOVERY_PENDING'}else{'RECOVERY_COMPLETE'})
}
function Read-Command {
    $bytes=[byte[]]::new(1);$text=[Text.StringBuilder]::new();$deadline=[DateTime]::UtcNow.AddSeconds(90)
    while($text.Length -lt 80){
        $remaining=[int]($deadline-[DateTime]::UtcNow).TotalMilliseconds
        if($remaining -le 0){throw 'Migration control deadline'}
        $task=$pipe.ReadAsync($bytes,0,1)
        if(!$task.Wait($remaining)){throw 'Migration control deadline'}
        if($task.Result -eq 0){throw 'Migration caller disconnected'}
        if($bytes[0] -eq 10){return $text.ToString()}
        if($bytes[0] -lt 32 -or $bytes[0] -gt 126){throw 'Invalid migration control command'}
        $null=$text.Append([char]$bytes[0])
    }
    throw 'Migration control command too long'
}
function Reply([string]$Text) { $bytes=[Text.Encoding]::ASCII.GetBytes($Text+"`n");if(!$pipe.WriteAsync($bytes,0,$bytes.Length).Wait(5000)){throw 'Migration control write deadline'} }
try {
    $security=[IO.Pipes.PipeSecurity]::new()
    $security.SetOwner($(if($Action -eq 'Probe'){$identity.User}else{[Security.Principal.SecurityIdentifier]::new('S-1-5-32-544')}))
    $security.SetAccessRuleProtection($true,$false)
    $security.AddAccessRule([IO.Pipes.PipeAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-2'),'FullControl','Deny'))
    $security.AddAccessRule([IO.Pipes.PipeAccessRule]::new($identity.User,'ReadWrite','Allow'))
    foreach($sid in @('S-1-5-18','S-1-5-32-544')){$security.AddAccessRule([IO.Pipes.PipeAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','Allow'))}
    $descriptor=$security.GetSecurityDescriptorBinaryForm()
    $sd=[Runtime.InteropServices.Marshal]::AllocHGlobal($descriptor.Length)
    $size=if([IntPtr]::Size -eq 8){24}else{12}
    $attributes=[Runtime.InteropServices.Marshal]::AllocHGlobal($size)
    try{
        [Runtime.InteropServices.Marshal]::Copy($descriptor,0,$sd,$descriptor.Length)
        [Runtime.InteropServices.Marshal]::WriteInt32($attributes,0,$size)
        [Runtime.InteropServices.Marshal]::WriteIntPtr($attributes,$(if([IntPtr]::Size -eq 8){8}else{4}),$sd)
        [Runtime.InteropServices.Marshal]::WriteInt32($attributes,$(if([IntPtr]::Size -eq 8){16}else{8}),0)
        # FIRST_PIPE_INSTANCE and REJECT_REMOTE_CLIENTS; SID ACL is defense in depth.
        $handle=[OpenergyMigrationNative]::CreateNamedPipeW(('\\.\pipe\'+$PipeName),0x40080003,8,1,1024,1024,90000,$attributes)
        if($handle -eq [IntPtr]::new(-1)){throw 'Cannot create exclusive local migration pipe'}
        $safe=[Microsoft.Win32.SafeHandles.SafePipeHandle]::new($handle,$true)
        try{$pipe=[IO.Pipes.NamedPipeServerStream]::new('InOut',$true,$false,$safe)}catch{$safe.Dispose();throw}
    }finally{[Runtime.InteropServices.Marshal]::FreeHGlobal($attributes);[Runtime.InteropServices.Marshal]::FreeHGlobal($sd)}
    if(!$pipe.WaitForConnectionAsync().Wait(90000)){throw 'Migration caller did not connect'}
    $clientId=[uint32]0
    if(![OpenergyMigrationNative]::GetNamedPipeClientProcessId($pipe.SafePipeHandle.DangerousGetHandle(),[ref]$clientId) -or $clientId -ne $ControllerPid){throw 'Migration pipe client is not the controller'}
    $client=Get-MigrationProcess $clientId
    if($client.started -ne $ControllerStarted -or $client.sid -ne $UserSid -or $client.image -ne $ControllerImage -or $client.admin -or $client.integrity -gt 8192){throw 'Migration controller token or process identity does not match'}
    if($Action -eq 'Probe'){Reply 'PROBE_OK';return}
    # The bounded protocol contains no paths or executable code.
    if($Action -ne 'Start' -and ![IO.File]::Exists($stateFile)){Reply 'ERROR|NO_STATE';throw 'NO_STATE'}
    $lock=Lock-State
    if($Action -eq 'Start') {
        if([IO.File]::Exists($stateFile)){throw 'A protected migration is already pending; roll it back first'}
        Reply 'AVAILABLE'
        if((Read-Command) -ne 'BEGIN'){throw 'Invalid migration start request'}
        $service=Load-Service
        try {
            $key=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($serviceKey)
            try{$start=[int]$key.GetValue('Start');$delayed=$key.GetValue('DelayedAutostart',$null)}finally{$key.Dispose()}
            if($start -notin @(2,3,4)){throw 'Unsupported service start mode'}
            $key=[Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($approvalKey)
            $approved=$null
            try{if($key){$approved=$key.GetValue($runName,$null)}}finally{if($key){$key.Dispose()}}
            if($null -ne $approved -and ($approved -isnot [byte[]] -or $approved.Length -gt 4096)){throw 'Invalid startup approval value'}
            $ui=$false
            foreach($process in [Diagnostics.Process]::GetProcessesByName('synergy-service')){
                try{$peer=Get-MigrationProcess $process.Id;if($peer.image -eq [IO.Path]::Combine($synergy,'synergy-service.exe') -and $peer.sid -eq $UserSid -and !$peer.admin -and $peer.integrity -le 8192){$ui=$true}}catch{}finally{$process.Dispose()}
            }
            $saved=@{start=$start;running=($service.Status -eq 'Running');ui=$ui;delayed=$delayed;approved=$approved}
            Save-State $saved
            $cutover=$true
            if($saved.running){$service.Stop();$service.WaitForStatus('Stopped',[TimeSpan]::FromSeconds(20))}
            foreach($name in @('synergy-core','synergy-tray','synergy-service')){
                foreach($process in [Diagnostics.Process]::GetProcessesByName($name)){
                    try{if($process.MainModule.FileName -eq [IO.Path]::Combine($synergy,$name+'.exe')){$process.Kill();$null=$process.WaitForExit(5000)}}finally{$process.Dispose()}
                }
            }
        } finally {$service.Dispose()}
        Reply 'STOPPED'
        $command=Read-Command
        if($command -eq 'READY'){
            Reply 'STARTED'
            # A completed pipe write alone does not prove the controller received
            # the reply. Keep recovery armed until it explicitly acknowledges.
            if((Read-Command) -ne 'STARTED_ACK'){throw 'Migration start was not acknowledged; protected recovery state retained'}
            $cutover=$false
            return
        }
        if($command -notin @('FAILED','FAILED_STOP')){throw 'Invalid migration completion request'}
        Restore-Service $saved;$cutover=$false
        Complete-Restoration $saved ($command -eq 'FAILED_STOP')
        return
    }
    $saved=Read-State
    Reply 'PENDING'
    $command=Read-Command
    if($Action -eq 'Commit') {
        if($command -ne 'COMMIT'){throw 'Invalid migration commit request'}
        $service=Load-Service;try{Set-Start 3}finally{$service.Dispose()}
        $key=[Microsoft.Win32.Registry]::LocalMachine.CreateSubKey($approvalKey)
        try{$key.SetValue($runName,[byte[]](3,0,0,0,0,0,0,0,0,0,0,0),[Microsoft.Win32.RegistryValueKind]::Binary)}finally{$key.Dispose()}
        Reply 'COMMITTED';return
    }
    if($command -notin @('ROLLBACK','ROLLBACK_STOP_FAILED')){throw 'Invalid migration rollback request'}
    Restore-Service $saved
    Complete-Restoration $saved ($command -eq 'ROLLBACK_STOP_FAILED')
} catch {
    # Caller loss/start failure must restore independently of per-user stop success.
    # Keep protected recovery state when completion could not be confirmed.
    if($cutover -and $saved){try{Restore-Service $saved}catch{[Console]::Error.WriteLine('Automatic Synergy restoration failed; protected recovery state retained.')}}
    try{if($pipe -and $pipe.IsConnected){Reply 'ERROR|BROKER_FAILED'}}catch{}
    throw
} finally {if($pipe){$pipe.Dispose()};if($lock){$lock.Dispose()}}
