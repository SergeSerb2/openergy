. (Join-Path $PSScriptRoot 'common.ps1')
if(-not (Get-Content -Raw -LiteralPath (Join-Path $root 'qualification.json')|ConvertFrom-Json).passed){throw 'source qualification required'}
if(Test-Path -LiteralPath (Join-Path $root 'build-once.guard')){throw 'build already attempted; no retry'}
[IO.File]::WriteAllText((Join-Path $root 'build-once.guard'),[DateTime]::UtcNow.ToString('o'))
$env:CARGO_HOME=Join-Path $root 'cargo-home'
$env:CARGO_TARGET_DIR=Join-Path $root 'target'
$env:CARGO_BUILD_JOBS='2';$env:CARGO_NET_OFFLINE='true'
$env:RUSTUP_AUTO_INSTALL='0';$env:RUSTUP_TOOLCHAIN='1.99.1-x86_64-pc-windows-msvc'
$env:RUSTC=Join-Path $toolRoot 'rustc.exe';$env:RUSTDOC=Join-Path $toolRoot 'rustdoc.exe'
$env:RUSTC_WRAPPER='';$env:RUSTC_WORKSPACE_WRAPPER=''
$env:RUSTFLAGS='';$env:CARGO_ENCODED_RUSTFLAGS=''
$env:TEMP=Join-Path $root 'temp';$env:TMP=$env:TEMP
$msvcBin=Join-Path $msvc 'bin/Hostx64/x64'
$sdkBin=Join-Path $sdk ('bin/'+$sdkVersion+'/x64')
$env:PATH=$toolRoot+';'+$msvcBin+';'+$sdkBin+';'+$env:PATH
$env:INCLUDE=(Join-Path $msvc 'include')+';'+(Join-Path $sdk ('Include/'+$sdkVersion+'/ucrt'))+';'+(Join-Path $sdk ('Include/'+$sdkVersion+'/shared'))+';'+(Join-Path $sdk ('Include/'+$sdkVersion+'/um'))+';'+(Join-Path $sdk ('Include/'+$sdkVersion+'/winrt'))
$env:LIB=(Join-Path $msvc 'lib/x64')+';'+(Join-Path $sdk ('Lib/'+$sdkVersion+'/ucrt/x64'))+';'+(Join-Path $sdk ('Lib/'+$sdkVersion+'/um/x64'))
$env:LIBPATH=Join-Path $msvc 'lib/x64'
$env:VCToolsInstallDir=$msvc+'\';$env:WindowsSdkDir=$sdk+'\';$env:WindowsSDKVersion=$sdkVersion+'\'
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=Join-Path $msvcBin 'link.exe'
$env:CC_x86_64_pc_windows_msvc=Join-Path $msvcBin 'cl.exe'
$env:AR_x86_64_pc_windows_msvc=Join-Path $msvcBin 'lib.exe'
$env:OPENERGY_DISABLE_DISCOVERY='1';$env:OPENERGY_ALLOW_PORT_24800='0'
$base=@('--locked','--offline','--release','--target','x86_64-pc-windows-msvc','--jobs','2','-p','openergy-daemon','--bin','openergy')
$steps=@(@{name='clippy';arguments=(@('clippy')+$base+@('--','-D','warnings'))},@{name='release';arguments=(@('build')+$base)})
function Same-Time($a,$b){[Math]::Abs(($a.ToUniversalTime()-$b.ToUniversalTime()).TotalMilliseconds) -lt 2}
$result=[ordered]@{source_commit='e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a';started_utc=[DateTime]::UtcNow.ToString('o');session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();jobs=2;native_target='x86_64-pc-windows-msvc';msvc=$msvc;sdk_version=$sdkVersion;child_only_environment=[ordered]@{CARGO_HOME=$env:CARGO_HOME;CARGO_TARGET_DIR=$env:CARGO_TARGET_DIR;RUSTC=$env:RUSTC;INCLUDE=$env:INCLUDE;LIB=$env:LIB;LIBPATH=$env:LIBPATH;linker=$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER;cc=$env:CC_x86_64_pc_windows_msvc;ar=$env:AR_x86_64_pc_windows_msvc;TEMP=$env:TEMP};steps=@();runtime_invocations=0;binary_executed=$false;passed=$false}
Save 'build-start.json' $result
Save 'build-controller.json' (Ancestors (Get-Process -Id $PID))
foreach($step in $steps){
 $info=[Diagnostics.ProcessStartInfo]::new((Join-Path $toolRoot 'cargo.exe'))
 $info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.RedirectStandardInput=$true
 $info.StandardOutputEncoding=[Text.UTF8Encoding]::new($false);$info.StandardErrorEncoding=[Text.UTF8Encoding]::new($false)
 $info.WorkingDirectory=Join-Path $root 'source'
 foreach($a in $step.arguments){$info.ArgumentList.Add($a)}
 $started=[DateTime]::UtcNow;$watch=[Diagnostics.Stopwatch]::StartNew()
 $child=[Diagnostics.Process]::Start($info);$child.StandardInput.Close();$handle=$child.SafeHandle;$rootIdentity=Identity $child;if($rootIdentity.priority -ne 'BelowNormal' -or $rootIdentity.session -ne 0){throw 'Cargo root priority/session mismatch'}
 $out=$child.StandardOutput.ReadToEndAsync();$err=$child.StandardError.ReadToEndAsync()
 $known=@{};$known[[int]$child.Id]=$child.StartTime.ToUniversalTime()
 $samples=[Collections.Generic.List[object]]::new()
 $races=[Collections.Generic.List[object]]::new()
 $seen=@{}
 $stepRecord=[ordered]@{name=$step.name;command=@($info.FileName)+$step.arguments;started_utc=$started.ToString('o');cargo_pid=$child.Id;root_identity=$rootIdentity;timeout_seconds=600;stdin_closed=$true;timed_out=$false;stopped=@()}
 Save ($step.name+'-start.json') $stepRecord
 while(-not $child.HasExited){
  $all=@(Get-CimInstance Win32_Process -Filter "Name='cargo.exe' OR Name='rustc.exe' OR Name='cargo-clippy.exe' OR Name='clippy-driver.exe' OR Name='link.exe' OR Name='cl.exe' OR Name LIKE 'build-script%'")
  for($pass=0;$pass -lt 4;$pass++){foreach($p in $all){if($known.ContainsKey([int]$p.ParentProcessId) -and $p.CreationDate.ToUniversalTime() -ge $known[[int]$p.ParentProcessId]){$known[[int]$p.ProcessId]=$p.CreationDate.ToUniversalTime()}}}
  foreach($p in $all){
   if(-not $known.ContainsKey([int]$p.ProcessId) -or -not (Same-Time $p.CreationDate $known[[int]$p.ProcessId])){continue}
   $key=([string]$p.ProcessId)+'/'+$p.CreationDate.ToUniversalTime().ToString('o')
   if($seen.ContainsKey($key)){continue}
   try{$live=Get-Process -Id $p.ProcessId -ErrorAction Stop;if(-not (Same-Time $live.StartTime $known[[int]$p.ProcessId])){continue};$sample=[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');name=$p.Name;pid=$p.ProcessId;parent_pid=$p.ParentProcessId;creation_utc=$p.CreationDate.ToUniversalTime().ToString('o');session=$live.SessionId;priority=$live.PriorityClass.ToString();path=$p.ExecutablePath;command_line=$p.CommandLine};$samples.Add($sample);$seen[$key]=$true}catch {$races.Add([ordered]@{pid=$p.ProcessId;message=$_.Exception.Message;recorded_utc=[DateTime]::UtcNow.ToString('o')})}
  }
  if($watch.Elapsed.TotalSeconds -ge 600){
   $stepRecord.timed_out=$true
   foreach($p in @($all|Sort-Object CreationDate -Descending)){
    if($known.ContainsKey([int]$p.ProcessId) -and (Same-Time $p.CreationDate $known[[int]$p.ProcessId])){
     $live=Get-Process -Id $p.ProcessId -ErrorAction SilentlyContinue
     if($live -and (Same-Time $live.StartTime $known[[int]$p.ProcessId])){$stepRecord.stopped+=@([ordered]@{pid=$p.ProcessId;creation=$p.CreationDate;path=$p.ExecutablePath;command_line=$p.CommandLine;owned_parent_chain=$true});$live.Kill()}
    }
   };break
  }
  Start-Sleep -Milliseconds 50
 }
 if(-not $child.WaitForExit(10000)){throw 'owned compiler recovery exceeded 10 seconds'};if(-not $out.Wait(5000) -or -not $err.Wait(5000)){throw 'compiler output receipt exceeded 5 seconds'};$watch.Stop()
 [IO.File]::WriteAllText((Join-Path $root ($step.name+'.stdout')),$out.Result,[Text.UTF8Encoding]::new($false))
 [IO.File]::WriteAllText((Join-Path $root ($step.name+'.stderr')),$err.Result,[Text.UTF8Encoding]::new($false))
 Save ($step.name+'-processes.json') @($samples.ToArray())
 Save ($step.name+'-observation-races.json') @($races.ToArray())
 $stepRecord.exit_code=$child.ExitCode;$stepRecord.elapsed_ms=$watch.ElapsedMilliseconds;$stepRecord.ended_utc=[DateTime]::UtcNow.ToString('o')
 $stepRecord.stdout_sha256=Hash (Join-Path $root ($step.name+'.stdout'));$stepRecord.stderr_sha256=Hash (Join-Path $root ($step.name+'.stderr'))
 $stepRecord.process_samples_sha256=Hash (Join-Path $root ($step.name+'-processes.json'))
 Save ($step.name+'-result.json') $stepRecord
 $result.steps+=@($stepRecord);Save 'build-result.json' $result
 $compilerCode=$child.ExitCode;$child.Dispose();$stepRecord.handle_disposed=$handle.IsClosed;Save ($step.name+'-result.json') $stepRecord
 if($compilerCode -ne 0 -or $stepRecord.timed_out){exit $(if($compilerCode -ne 0){$compilerCode}else{1})}
}
$binary=Join-Path $root 'target/x86_64-pc-windows-msvc/release/openergy.exe'
$f=Get-Item -LiteralPath $binary
$result.artifact=[ordered]@{path=$binary;bytes=$f.Length;sha256=(Hash $binary);executed=$false}
$pe=[IO.File]::ReadAllBytes($binary);$peOffset=[BitConverter]::ToInt32($pe,60);if($pe[0] -ne 77 -or $pe[1] -ne 90 -or [BitConverter]::ToUInt32($pe,$peOffset) -ne 17744 -or [BitConverter]::ToUInt16($pe,$peOffset+4) -ne 34404 -or [BitConverter]::ToUInt16($pe,$peOffset+24) -ne 523){throw 'not AMD64 PE32+'};$result.artifact.amd64_pe32plus=$true
Save 'source-after-build.json' (Assert-Source)
$result.passed=$true;$result.ended_utc=[DateTime]::UtcNow.ToString('o')
Save 'build-result.json' $result
Write-Output 'NATIVE_WINDOWS_CLIPPY_AND_BUILD_PASS_NO_RUNTIME'
