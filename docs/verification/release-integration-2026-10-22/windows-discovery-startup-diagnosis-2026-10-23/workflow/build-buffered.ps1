. (Join-Path $PSScriptRoot 'common.ps1')
if(Test-Path (Join-Path $root 'compile-v2-result.json')){throw 'buffered build already attempted'}
$active=@(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.Name -in @('cargo.exe','rustc.exe','cargo-clippy.exe','clippy-driver.exe')})
if($active.Count -ne 0){throw 'another build active'}
$env:RUSTUP_TOOLCHAIN='1.99.1-x86_64-pc-windows-msvc'
$first=Get-Content -Raw (Join-Path $root 'compile-result.json')|ConvertFrom-Json
if($first.exit -ne 0){throw 'first compile failed'}
$source=Join-Path $root 'startup_diagnostic_v2.rs';(Get-Item $source).IsReadOnly=$true
$args=@($first.arguments);$args[$args.Count-3]=$source;$args[$args.Count-1]=Join-Path $root 'bin/startup_diagnostic_v2.exe'
$watch=[Diagnostics.Stopwatch]::StartNew();$started=[DateTime]::UtcNow.ToString('o')
& (Join-Path $toolRoot 'rustc.exe') @args *> (Join-Path $root 'logs/compile-v2.log')
$code=$LASTEXITCODE;$watch.Stop()
$result=[ordered]@{started_utc=$started;ended_utc=[DateTime]::UtcNow.ToString('o');exit=$code;elapsed_ms=$watch.ElapsedMilliseconds;arguments=$args;source_sha256=(Hash $source);source_bytes=(Get-Item $source).Length;compiler_sha256=(Hash (Join-Path $toolRoot 'rustc.exe'));session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();jobs=2;concurrent_compiler_invocations=1;log_sha256=(Hash (Join-Path $root 'logs/compile-v2.log'))}
if($code -eq 0){$out=$args[-1];$f=Get-Item $out;$result.binary=[ordered]@{path=$out;bytes=$f.Length;sha256=(Hash $out);readonly=$true};foreach($f in @(Get-ChildItem (Join-Path $root 'bin') -File)){$f.IsReadOnly=$true}}
Save-Json $result (Join-Path $root 'compile-v2-result.json')
Write-Output ('BUFFERED_COMPILE_EXIT='+$code)
exit $code
