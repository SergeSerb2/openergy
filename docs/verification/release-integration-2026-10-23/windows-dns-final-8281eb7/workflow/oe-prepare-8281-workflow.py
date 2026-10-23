import pathlib,json
ptr=json.loads(pathlib.Path('/private/tmp/oe-win-dns-8281eb7-private-pointer.json').read_text());p=pathlib.Path(ptr['local'])
b=p/'build.ps1';s=b.read_text().replace('8333a48df046bdc7e5f0cfc687ed09f2b75b39f3','8281eb79f54725ddc8fe8bae83dbd0f44fa5d885')
s=s.replace("$preTools=Tools", """$nativeReference=Get-Content -Raw (Join-Path $root 'native-tool-reference.json')|ConvertFrom-Json
$nativeBefore=@(foreach($e in $nativeReference.native_tools){$f=Get-Item -LiteralPath $e.path;if($f.Length -ne $e.bytes -or (Hash $f.FullName) -ne $e.sha256){throw 'native compiler/tool reference mismatch before build'};[ordered]@{name=$e.name;path=$e.path;bytes=$f.Length;sha256=(Hash $f.FullName);file_version=$f.VersionInfo.FileVersion}})
Save-Json ([ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');native_tools=$nativeBefore;fresh_prebuild_sample=$true}) (Join-Path $root 'native-tools-before.json')
$preTools=Tools""")
old=""" & (Join-Path $toolRoot 'cargo.exe') @arguments *> (Join-Path $root ('logs/'+$step.name+'.log'))
 $exit=$LASTEXITCODE;$watch.Stop();"""
new=""" $info=[Diagnostics.ProcessStartInfo]::new((Join-Path $toolRoot 'cargo.exe'));$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.WorkingDirectory=Join-Path $root 'source'
 foreach($arg in $arguments){$info.ArgumentList.Add($arg)}
 $child=[Diagnostics.Process]::Start($info);$childStart=$child.StartTime;$out=$child.StandardOutput.ReadToEndAsync();$err=$child.StandardError.ReadToEndAsync();$samples=@();$timedOut=$false
 $samples+=@([ordered]@{name='cargo.exe';pid=$child.Id;parent_pid=$PID;path=$child.Path;created=$child.StartTime.ToUniversalTime().ToString('o');session=$child.SessionId;priority=$child.PriorityClass.ToString();at_ms=$watch.ElapsedMilliseconds})
 $seen=@{}
 while(-not $child.WaitForExit(250)){
  foreach($c in @(Get-CimInstance Win32_Process -ErrorAction Stop|Where-Object {$_.ExecutablePath -and $_.ExecutablePath.StartsWith($toolRoot+'\\',[StringComparison]::OrdinalIgnoreCase) -and $_.Name -in @('cargo.exe','cargo-clippy.exe','rustc.exe','clippy-driver.exe') -and $_.CreationDate.ToUniversalTime() -ge $childStart.ToUniversalTime()})){
   $key=[string]$c.ProcessId+':'+$c.CreationDate.ToString('o')
   if(-not $seen.ContainsKey($key)){$actual=Get-Process -Id $c.ProcessId -ErrorAction SilentlyContinue;if($actual){$samples+=@([ordered]@{name=$c.Name;pid=$c.ProcessId;parent_pid=$c.ParentProcessId;path=$c.ExecutablePath;created=$c.CreationDate.ToUniversalTime().ToString('o');session=$c.SessionId;priority=$actual.PriorityClass.ToString();at_ms=$watch.ElapsedMilliseconds});$seen[$key]=$true}}
  }
  if($watch.Elapsed.TotalSeconds -gt 600){$timedOut=$true;$owned=Get-Process -Id $child.Id -ErrorAction Stop;if($owned.Path -ne (Join-Path $toolRoot 'cargo.exe') -or $owned.StartTime -ne $childStart){throw 'build watchdog ownership mismatch'};$owned.Kill($true);break}
 }
 if(-not $child.WaitForExit(5000)){throw 'build child did not exit'}
 $exit=$child.ExitCode;$watch.Stop()
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$step.name+'.stdout')),$out.Result,[Text.UTF8Encoding]::new($false));[IO.File]::WriteAllText((Join-Path $root ('logs/'+$step.name+'.stderr')),$err.Result,[Text.UTF8Encoding]::new($false))
 [IO.File]::WriteAllText((Join-Path $root ('logs/'+$step.name+'.log')),$out.Result+$err.Result,[Text.UTF8Encoding]::new($false))
 Save-Json ([ordered]@{step=$step.name;timed_out=$timedOut;samples=$samples;stdout_sha256=(Hash (Join-Path $root ('logs/'+$step.name+'.stdout')));stderr_sha256=(Hash (Join-Path $root ('logs/'+$step.name+'.stderr')))}) (Join-Path $root ($step.name+'-child-observation.json'))
 if($timedOut){throw 'build watchdog exceeded; no retry'}
 """
assert old in s;s=s.replace(old,new);b.write_text(s)
r=p/'runtime-common.ps1';s=r.read_text().replace('IPv6Loopback,25119','IPv6Loopback,25116');r.write_text(s)
# Write all runtime/audit workflow inputs before build, and retain their immutable hashes.
inputs=['source.tar','source-manifest.json','common.ps1','audit.ps1','build.ps1','runtime-common.ps1','run.ps1','postflight.ps1','state-input.json','native-tool-reference.json']
import hashlib
(p/'final-input-hashes.json').write_text(json.dumps([{'path':f,'sha256':hashlib.sha256((p/f).read_bytes()).hexdigest()} for f in inputs],indent=2)+'\n')
print('Prepared exact build and fixed two-command runtime workflow; no source edits.')
