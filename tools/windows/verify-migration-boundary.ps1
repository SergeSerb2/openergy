# Full external commands and real Windows ACL/token/pipe checks; no services/tasks.
# Run from PowerShell 7 as administrator, using an already-built virtual binary.
param(
 [string]$ScriptsDir=$PSScriptRoot,
 [string]$Lab=[IO.Path]::Combine([IO.Path]::GetTempPath(),'oe-migration-boundary-'+[guid]::NewGuid().ToString('N')),
 [Parameter(Mandatory=$true)][string]$BinaryPath,
 [ValidateRange(26520,26539)][int]$Port=26530
)
$ErrorActionPreference='Stop'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
if($PSVersionTable.PSEdition -ne 'Core' -or !([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Use an administrator PowerShell 7 for the disposable ACL fixtures'}
$source=[IO.Path]::GetFullPath($ScriptsDir)
$BinaryPath=[IO.Path]::GetFullPath($BinaryPath)
$Lab=Join-Path $Lab ('run-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Lab|Out-Null
"RUN_FIXTURE=$Lab"
$windows=[Environment]::GetFolderPath([Environment+SpecialFolder]::Windows)
$ps5=Join-Path $windows 'System32\WindowsPowerShell\v1.0\powershell.exe'
$ps7=[Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
$protected=Join-Path ([IO.Path]::GetPathRoot($windows)) ('oe-temp-migration-'+[guid]::NewGuid().ToString('N'))
[IO.File]::WriteAllText((Join-Path $Lab 'protected-fixture.txt'),$protected)
$binaryHash=(Get-FileHash $BinaryPath -Algorithm SHA256).Hash
$helperHash=(Get-FileHash (Join-Path $source 'openergy-migration-admin.ps1') -Algorithm SHA256).Hash
function Check($v,$name){if(!$v){throw $name};"PASS $name"}
function Quote([string]$s){"'"+$s.Replace("'","''")+"'"}
function Run-Full([string]$Engine,[string]$Code,[int]$Expected=0){
 $file=Join-Path $Lab ('full-'+[guid]::NewGuid().ToString('N')+'.ps1');[IO.File]::WriteAllText($file,$Code)
 $info=[Diagnostics.ProcessStartInfo]::new($Engine);$info.UseShellExecute=$false;$info.RedirectStandardError=$true;$info.RedirectStandardOutput=$true;$info.WorkingDirectory=$Lab;$info.Arguments='-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "'+$file+'"'
 $p=[Diagnostics.Process]::Start($info);$o=$p.StandardOutput.ReadToEndAsync();$e=$p.StandardError.ReadToEndAsync()
 try{if(!$p.WaitForExit(90000)){$p.Kill();$p.WaitForExit();throw 'owned command timeout'};if($p.ExitCode -ne $Expected){throw "command exit $($p.ExitCode): $($e.Result) $($o.Result)"};return $o.Result+$e.Result}finally{$p.Dispose()}
}
# This test launcher creates a real restricted medium token and launches the actual
# public scripts. It does not replace any service/task/platform interface.
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
public static class OeMediumProcess {
 [StructLayout(LayoutKind.Sequential)] struct SA {public IntPtr Sid;public uint Attributes;}
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct SI {public int cb;public string r;public string desktop;public string title;public int x,y,w,h,cx,cy,fill,flags;public short show,reserved;public IntPtr reserved2,input,output,error;}
 [StructLayout(LayoutKind.Sequential)] struct PI {public IntPtr process,thread;public uint pid,tid;}
 [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr p,uint access,out IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool CreateRestrictedToken(IntPtr token,uint flags,uint count,SA[] disable,uint priv,IntPtr deleted,uint restricted,IntPtr restricting,out IntPtr result);
 [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool ConvertStringSidToSid(string text,out IntPtr sid);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool SetTokenInformation(IntPtr token,int cls,ref SA value,int size);
 [DllImport("advapi32.dll")] static extern int GetLengthSid(IntPtr sid);
 [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr p);
 [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateProcessWithTokenW(IntPtr token,uint flags,string app,StringBuilder command,uint creation,IntPtr env,string cwd,ref SI si,out PI pi);
 [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateProcessAsUserW(IntPtr token,string app,StringBuilder command,IntPtr pa,IntPtr ta,bool inherit,uint creation,IntPtr env,string cwd,ref SI si,out PI pi);
 [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h,uint ms);
 [DllImport("kernel32.dll")] static extern bool GetExitCodeProcess(IntPtr h,out uint code);
 [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr h,uint code);
 public static int Run(string engine,string command,string cwd){
  IntPtr original=IntPtr.Zero,token=IntPtr.Zero,admin=IntPtr.Zero,medium=IntPtr.Zero;
  try{
   if(!OpenProcessToken(GetCurrentProcess(),0xF01FF,out original)||!ConvertStringSidToSid("S-1-5-32-544",out admin))throw new Win32Exception();
   if(!CreateRestrictedToken(original,1,1,new[]{new SA{Sid=admin}},0,IntPtr.Zero,0,IntPtr.Zero,out token))throw new Win32Exception(Marshal.GetLastWin32Error(),"CreateRestrictedToken");if(!ConvertStringSidToSid("S-1-16-8192",out medium))throw new Win32Exception(Marshal.GetLastWin32Error(),"Medium SID");
   var label=new SA{Sid=medium,Attributes=0x20};if(!SetTokenInformation(token,25,ref label,Marshal.SizeOf(typeof(SA))+GetLengthSid(medium)))throw new Win32Exception(Marshal.GetLastWin32Error(),"SetTokenInformation size="+(Marshal.SizeOf(typeof(SA))+GetLengthSid(medium)));
   var si=new SI{cb=Marshal.SizeOf(typeof(SI))};PI pi;
   if(!CreateProcessWithTokenW(token,0,engine,new StringBuilder(command),0,IntPtr.Zero,cwd,ref si,out pi))throw new Win32Exception(Marshal.GetLastWin32Error(),"CreateProcessWithTokenW error="+Marshal.GetLastWin32Error()+" SI="+si.cb);
   CloseHandle(pi.thread);
   try{if(WaitForSingleObject(pi.process,90000)!=0){TerminateProcess(pi.process,99);throw new Exception("owned medium child timeout");}uint code;if(!GetExitCodeProcess(pi.process,out code))throw new Win32Exception();return (int)code;}finally{CloseHandle(pi.process);}
  }finally{if(original!=IntPtr.Zero)CloseHandle(original);if(token!=IntPtr.Zero)CloseHandle(token);if(admin!=IntPtr.Zero)LocalFree(admin);if(medium!=IntPtr.Zero)LocalFree(medium);}
 }
}
'@
function Run-Medium([string]$Engine,[string]$Code,[string]$Tag,[int]$Expected=0){
 $log=Join-Path $Lab ($Tag+'.log')
 $wrapped='$ErrorActionPreference="Stop";try{& {'+$Code+'} *> '+(Quote $log)+';exit 0}catch{[IO.File]::AppendAllText('+(Quote $log)+',($_|Out-String));exit 1}'
 $child=Join-Path $Lab ($Tag+'-child.ps1');[IO.File]::WriteAllText($child,$wrapped)
 $command='"'+$Engine+'" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "'+$child+'"'
 $exit=[OeMediumProcess]::Run($Engine,$command,$Lab)
 $output=if(Test-Path $log){Get-Content $log -Raw}else{'NO LOG'}
 if($exit -ne $Expected){throw "medium $Tag exit=$exit expected=$Expected $output"}
 return $output
}
# Create only a disposable ACL fixture, atomically owned by Administrators. This is
# not a Program Files installation and never receives a service/task registration.
$prep=@'
param($Root,$Source,$Expected)
$ErrorActionPreference='Stop'
$owner=[Security.Principal.SecurityIdentifier]::new('S-1-5-32-544')
$acl=[Security.AccessControl.DirectorySecurity]::new();$acl.SetOwner($owner);$acl.SetAccessRuleProtection($true,$false)
foreach($sid in @('S-1-5-18','S-1-5-32-544')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))}
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-32-545'),'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow'))
$null=[IO.Directory]::CreateDirectory($Root,$acl)
$bytes=[IO.File]::ReadAllBytes($Source)
$sha=[Security.Cryptography.SHA256]::Create();try{$hash=[BitConverter]::ToString($sha.ComputeHash($bytes)).Replace('-','')}finally{$sha.Dispose()}
if($hash -ne $Expected){throw 'fixture source hash mismatch'}
$security=[Security.AccessControl.FileSecurity]::new();$security.SetOwner($owner);$security.SetAccessRuleProtection($true,$false)
foreach($sid in @('S-1-5-18','S-1-5-32-544')){$security.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','Allow'))}
$security.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-32-545'),'ReadAndExecute','Allow'))
$p=[IO.Path]::Combine($Root,'openergy-migration-admin.ps1')
$f=[IO.FileStream]::new($p,'CreateNew',[Security.AccessControl.FileSystemRights]::Write,'None',4096,'WriteThrough',$security)
try{$f.Write($bytes,0,$bytes.Length);$f.Flush($true)}finally{$f.Dispose()}
'@
$prepPath=Join-Path $Lab 'prepare-acl-fixture.ps1';[IO.File]::WriteAllText($prepPath,$prep)
try {
$null=Run-Full $ps5 ('& '+(Quote $prepPath)+' -Root '+(Quote $protected)+' -Source '+(Quote (Join-Path $source 'openergy-migration-admin.ps1'))+' -Expected '+(Quote $helperHash))
$helper=Join-Path $protected 'openergy-migration-admin.ps1'
$wrapper=Join-Path $source 'switch-from-synergy.ps1'
$marker=Join-Path $Lab 'replacement-manager-ran'
$attack=Join-Path $Lab 'attack-install';New-Item -ItemType Directory -Path $attack|Out-Null
[IO.File]::WriteAllText((Join-Path $attack 'openergy.ps1'),('[IO.File]::WriteAllText('+(Quote $marker)+',"executed")'))
foreach($engine in @($ps5,$ps7)){
 $name=if($engine -eq $ps5){'ps51'}else{'ps7'}
 foreach($file in Get-ChildItem -LiteralPath $source -Filter '*.ps1'){$null=Run-Full $engine ('$t=$null;$e=$null;$null=[Management.Automation.Language.Parser]::ParseFile('+(Quote $file.FullName)+',[ref]$t,[ref]$e);if($e.Count){throw ($e|Out-String)}')}
 Check $true "$name full script parsing"
 foreach($action in @('Install','Start','Commit','Rollback','CheckBroker')){
  $out=Run-Full $engine ('& '+(Quote $wrapper)+' -'+$action+' -InstallDir '+(Quote $attack)+' -DataDir '+(Quote (Join-Path $Lab 'attack-data'))) 1
  Check ($out.Contains('non-administrator') -and !(Test-Path $marker)) "$name elevated $action rejects before replacement manager execution"
 }
 $out=Run-Full $engine ('& '+(Quote (Join-Path $source 'openergy.ps1'))+' Status -InstallDir '+(Quote $attack)+' -DataDir '+(Quote (Join-Path $Lab 'attack-data'))) 1
 Check $out.Contains('non-administrator') "$name manager refuses elevated token"
 $out=Run-Full $engine ('& '+(Quote (Join-Path $source 'openergy-run.ps1'))+' -InstallDir '+(Quote $attack)+' -DataDir '+(Quote (Join-Path $Lab 'attack-data'))+' -Platform virtual') 1
 Check $out.Contains('non-administrator') "$name supervisor refuses elevated token"
 $out=Run-Full $engine ('& '+(Quote $helper)+' -Action Check')
 Check $out.Contains('PROTECTED_MIGRATION_HELPER_VALID') "$name protected admin-owned helper and ancestor ACL check"
 try{$out=Run-Medium $engine ('& '+(Quote $wrapper)+' -CheckBroker -BrokerPath '+(Quote $helper)) ($name+'-probe')}catch{
  if($_.Exception.Message.Contains('-1073741502')){"LIMIT $name restricted medium process exits 0xc0000142 before user code in SSH session 0; no desktop ACL changed";continue};throw
 }
 Check $out.Contains('PROTECTED_MIGRATION_CHANNEL_VALID') "$name actual medium-token protected helper probe and kernel peer binding"
 $node=Join-Path $Lab ($name+'-lifecycle');$install=Join-Path $node 'install';$data=Join-Path $node 'data';New-Item -ItemType Directory -Path $node|Out-Null
 $manager=Join-Path $source 'openergy.ps1'
 $code='$env:OPENERGY_DISABLE_DISCOVERY="1";$env:OPENERGY_ALLOW_PORT_24800="0";$m='+(Quote $manager)+';$i='+(Quote $install)+';$d='+(Quote $data)+';try{& $m Install -InstallDir $i -DataDir $d -BinaryPath '+(Quote $BinaryPath)+' -Platform virtual -NoStartup -Port '+$Port+'|Out-Null;& $m Validate -InstallDir $i -DataDir $d|Out-Null;$s=& $m Start -InstallDir $i -DataDir $d;if($s.daemon.platform -ne "virtual"){throw "wrong platform"};& $m Status -InstallDir $i -DataDir $d|Out-Null;& $m Stop -InstallDir $i -DataDir $d|Out-Null;& $m Uninstall -InstallDir $i -DataDir $d|Out-Null;"REAL_LIMITED_LIFECYCLE_PASS"}finally{if(Test-Path (Join-Path $i "install.json")){& $m Stop -InstallDir $i -DataDir $d|Out-Null;& $m Uninstall -InstallDir $i -DataDir $d|Out-Null}}'
 $out=Run-Medium $engine $code ($name+'-lifecycle')
 Check $out.Contains('REAL_LIMITED_LIFECYCLE_PASS') "$name actual limited-token install/validate/start/status/stop/uninstall virtual daemon"
}
# Read-only public broker commands exercise actual kernel pipe creation and peer
# rejection. No request can reach service/state operations in these cases.
foreach($wrong in @('pid','start','admin-token')){
 $pipeName='OpenergyMigration-'+[guid]::NewGuid().ToString('N')
 $caller=[Diagnostics.Process]::GetCurrentProcess()
 $expectedPid=if($wrong -eq 'pid'){$PID+100000}else{$PID}
 $expectedStarted=if($wrong -eq 'start'){'0'}else{$caller.StartTime.ToUniversalTime().Ticks.ToString()}
 $info=[Diagnostics.ProcessStartInfo]::new($ps5);$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $info.Arguments='-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "'+$helper+'" -Action Probe -PipeName '+$pipeName+' -UserSid '+[Security.Principal.WindowsIdentity]::GetCurrent().User.Value+' -ControllerPid '+$expectedPid+' -ControllerStarted '+$expectedStarted+' -ControllerImage "'+$ps7+'"'
 $p=[Diagnostics.Process]::Start($info);$out=$p.StandardOutput.ReadToEndAsync();$err=$p.StandardError.ReadToEndAsync()
 $client=[IO.Pipes.NamedPipeClientStream]::new('.',$pipeName,'InOut','Asynchronous',[Security.Principal.TokenImpersonationLevel]::Identification)
 try{
  try{$client.Connect(10000)}catch{if([Security.Principal.WindowsIdentity]::GetCurrent().Groups.Value -contains 'S-1-5-2'){Check ($_.Exception.Message.Contains('denied')) 'actual kernel migration pipe denies the SSH Network SID';continue};throw}
  $acl=[IO.Pipes.PipeStreamAclExtensions]::GetAccessControl($client)
  Check ($acl.AreAccessRulesProtected -and @($acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])|Where-Object {$_.IdentityReference.Value -eq 'S-1-5-2' -and $_.AccessControlType -eq 'Deny'}).Count -eq 1) 'actual migration pipe has protected DACL and Network SID deny'
  if(!$p.WaitForExit(10000)){throw 'readonly broker peer rejection deadline'}
  Check ($p.ExitCode -eq 1 -and $err.Result.Contains($(if($wrong -eq 'pid'){'not the controller'}else{'does not match'}))) "actual broker rejects $wrong before any privileged operation"
 }finally{$client.Dispose();if(!$p.HasExited){$p.Kill();$p.WaitForExit()};$p.Dispose()}
}
# A separate real server owns the nonce endpoint. FIRST_PIPE_INSTANCE must refuse
# it before any reply or service operation; no counterfeit protocol is substituted.
$occupiedName='OpenergyMigration-'+[guid]::NewGuid().ToString('N')
$squatter=[IO.Pipes.NamedPipeServerStream]::new($occupiedName,'InOut',1)
try{
 $out=Run-Full $ps5 ('& '+(Quote $helper)+' -Action Probe -PipeName '+$occupiedName+' -UserSid '+[Security.Principal.WindowsIdentity]::GetCurrent().User.Value+' -ControllerPid '+$PID+' -ControllerStarted '+[Diagnostics.Process]::GetCurrentProcess().StartTime.ToUniversalTime().Ticks.ToString()+' -ControllerImage '+(Quote $ps7)) 1
 Check $out.Contains('Cannot create exclusive local migration pipe') 'actual broker refuses an occupied nonce pipe before sending any reply'
}finally{$squatter.Dispose()}
$checkTarget=Join-Path $protected 'candidate.ps1'
[IO.File]::WriteAllText($checkTarget,'throw "must never execute this candidate"')
$acl=Get-Acl -LiteralPath $checkTarget
$acl.SetOwner([Security.Principal.WindowsIdentity]::GetCurrent().User)
Set-Acl -LiteralPath $checkTarget -AclObject $acl
foreach($engine in @($ps5,$ps7)){
 $out=Run-Full $engine ('& '+(Quote $helper)+' -Action Check -CheckPath '+(Quote $checkTarget)) 1
 Check $out.Contains('owner must be Administrators or SYSTEM') 'actual kernel ACL rejects same-user ownership despite protected DACL'
}
$acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.WindowsIdentity]::GetCurrent().User,'Write','Allow'))
Set-Acl -LiteralPath $checkTarget -AclObject $acl
foreach($engine in @($ps5,$ps7)){
 $out=Run-Full $engine ('& '+(Quote $helper)+' -Action Check -CheckPath '+(Quote $checkTarget)) 1
 Check $out.Contains('grants mutation or replacement') 'actual kernel ACL rejects writable protected-owner candidate'
}

# Candidate files are inspected only as data by the protected public Check action.
$acl.RemoveAccessRuleAll([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.WindowsIdentity]::GetCurrent().User,'Write','Allow'))
Set-Acl -LiteralPath $checkTarget -AclObject $acl
$ancestor=Join-Path $protected 'ancestor'
$null=[IO.Directory]::CreateDirectory($ancestor)
$leaf=Join-Path $ancestor 'candidate.ps1'
[IO.File]::WriteAllText($leaf,'throw "candidate must not execute"')
$leafAcl=Get-Acl -LiteralPath $leaf;$leafAcl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'));Set-Acl -LiteralPath $leaf -AclObject $leafAcl
$parentAcl=Get-Acl -LiteralPath $ancestor
$parentAcl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
$parentAcl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.WindowsIdentity]::GetCurrent().User,'DeleteSubdirectoriesAndFiles','Allow'))
Set-Acl -LiteralPath $ancestor -AclObject $parentAcl
foreach($engine in @($ps5,$ps7)){
 $out=Run-Full $engine ('& '+(Quote $helper)+' -Action Check -CheckPath '+(Quote $leaf)) 1
 Check $out.Contains('grants mutation or replacement') 'actual kernel ACL rejects parent DELETE_CHILD replacement rights'
}
$junction=Join-Path $protected 'junction'
$null=New-Item -ItemType Junction -Path $junction -Target $protected
foreach($engine in @($ps5,$ps7)){
 $out=Run-Full $engine ('& '+(Quote $helper)+' -Action Check -CheckPath '+(Quote (Join-Path $junction 'candidate.ps1'))) 1
 Check $out.Contains('reparse point') 'actual kernel path rejects junction ancestor'
}
[IO.Directory]::Delete($junction)
$moduleRoot=Join-Path $Lab 'poison-modules'
$moduleMarker=Join-Path $Lab 'user-module-loaded'
foreach($module in @('Microsoft.PowerShell.Management','ScheduledTasks')){
 $dir=Join-Path $moduleRoot $module;$null=[IO.Directory]::CreateDirectory($dir)
 [IO.File]::WriteAllText((Join-Path $dir ($module+'.psm1')),('[IO.File]::WriteAllText('+(Quote $moduleMarker)+',"unsafe load");throw "unsafe module"'))
}
foreach($engine in @($ps5,$ps7)){
 $out=Run-Full $engine ('$env:PSModulePath='+(Quote $moduleRoot)+';& '+(Quote $helper)+' -Action Check')
 Check ($out.Contains('PROTECTED_MIGRATION_HELPER_VALID') -and !(Test-Path $moduleMarker)) 'protected read-only command does not autoload user modules'
}

Check (!(Test-Path (Join-Path $protected 'State'))) 'read-only checks and rejected peers create no recovery state'

Check ((Get-FileHash $helper -Algorithm SHA256).Hash -eq $helperHash) 'protected fixture code remains hash-identical'
Check ((Get-FileHash $BinaryPath -Algorithm SHA256).Hash -eq $binaryHash) 'validated binary remains unchanged'
"PROTECTED_FIXTURE=$protected"
'WINDOWS_MIGRATION_BOUNDARY_NEGATIVE_PASS'
'MEDIUM_INTERACTIVE_ACCEPTANCE_REMAINS_REQUIRED'

} finally {
 # Delete only this run's exact fixture, after checking its owner, code hash and
 # complete file inventory. Never traverse a reparse point or a recovery record.
 if([IO.Directory]::Exists($protected)){
  $expected=@('openergy-migration-admin.ps1','candidate.ps1','ancestor\candidate.ps1')
  $rootItem=[IO.DirectoryInfo]::new($protected)
  if($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Cleanup root is a reparse point'}
  if((Get-Acl $protected).GetOwner([Security.Principal.SecurityIdentifier]).Value -ne 'S-1-5-32-544'){throw 'Cleanup fixture owner changed'}
  $helper=Join-Path $protected 'openergy-migration-admin.ps1'
  if(!(Test-Path $helper) -or (Get-FileHash $helper -Algorithm SHA256).Hash -ne $helperHash){throw 'Cleanup helper hash changed; fixture retained'}
  $junction=Join-Path $protected 'junction'
  if([IO.Directory]::Exists($junction)){
   $link=[IO.DirectoryInfo]::new($junction)
   if(!($link.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Cleanup junction is not the owned link'}
   [IO.Directory]::Delete($junction)
  }
  foreach($item in Get-ChildItem -LiteralPath $protected -Recurse -Force){
   if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Unexpected cleanup reparse point; fixture retained'}
   if($item.PSIsContainer){if($item.Name -ne 'ancestor'){throw 'Unexpected cleanup directory; fixture retained'}}
   elseif($item.FullName.Substring($protected.Length+1) -notin $expected){throw 'Unexpected cleanup file; fixture retained'}
  }
  Remove-Item -LiteralPath $protected -Recurse -Force
  'OWNED_PROTECTED_FIXTURE_HASH_CHECKED_CLEANUP_PASS'
 }
}
