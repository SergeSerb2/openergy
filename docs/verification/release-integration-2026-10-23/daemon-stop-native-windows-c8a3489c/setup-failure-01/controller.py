"""Fresh bounded native Windows daemon qualification. Never executes a saved controller.
Private originals contain transport details; committed copies mask known prefixes only.
One compiler stage and one runtime stage; neither can be repeated.
"""
import base64
import datetime
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile
import time
import traceback

HERE = Path(__file__).resolve().parent
REPO = Path(subprocess.check_output(['git','-C',str(HERE),'rev-parse','--show-toplevel']).decode().strip())
SOURCE = 'e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a'
OPTIONS = ['-o','BatchMode=yes','-o','ConnectTimeout=8','-o','StrictHostKeyChecking=yes','-o','UpdateHostKeys=no','-o','ControlMaster=no','-o','ServerAliveInterval=10','-o','ServerAliveCountMax=2','-o','ForwardAgent=no']
def sha(data): return hashlib.sha256(data).hexdigest()
def save(path,obj): path.write_text(json.dumps(obj,indent=2)+'\n')
def now(): return datetime.datetime.now().astimezone().isoformat()
def git(*args): return subprocess.check_output(['git','-C',str(REPO),*args],cwd=REPO)

def local_prior():
    roots=[HERE.parent/'idle-tls-windows-build-r6',HERE.parent/'idle-tls-windows-build-r5',HERE.parent/'idle-tls-deadline-r2-fix-06b9a13-773e34c7']
    other=Path('/Users/serge/.t3/worktrees/openergy/t3-a3f6033f/docs/verification/release-integration-2026-10-23')
    roots += [other/'idle-tls-windows-build-r6',other/'idle-tls-windows-build-r5']
    return [{'owner':str(root),'files':[{'path':str(p.relative_to(root)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(root.rglob('*')) if p.is_file() and '__pycache__' not in p.parts]} for root in roots if root.is_dir()]

class Controller:
    def __init__(self):
        with (HERE/'controller-once.guard').open('x') as f:f.write(now()+'\n')
        self.started=time.monotonic(); self.deadline=self.started+2400
        assert os.getpriority(os.PRIO_PROCESS,0)==15
        assert os.environ.get('MACOSX_DEPLOYMENT_TARGET')=='15.0'
        transport=json.loads(Path('/tmp/oe-stop-c8a3489c-transport-selection.json').read_text())
        self.alias=transport['ssh_alias'];self.old_private=Path(transport['r6_private_root'])
        self.private=Path(tempfile.mkdtemp(prefix='oe-stop-native-win-c8a3489c-'));os.chmod(self.private,0o700)
        self.raw=self.private/'raw';self.raw.mkdir(mode=0o700)
        self.root=None;self.state=None
        self.nonce=os.urandom(4).hex();self.remote_name='oe-stop-win-c8a3489c-e1ef124-'+self.nonce
        self.result={'source_commit':SOURCE,'started_local':now(),'controller_nice':os.getpriority(os.PRIO_PROCESS,0),'macos_deployment_target':'15.0','total_workflow_bound_seconds':2400,'owned_recovery_bound_seconds':30,'passed':False,'source_overlays':[],'compiler_stage_attempts':0,'runtime_stage_attempts':0,'remote_fixture':self.remote_name,'transport':'Existing R6 transport; OpenSSH immediate control stdin EOF; fixed-file scp -O; strict known-host checking; no credential read/copy'}
        save(HERE/'private-evidence-location.json',{'private_root':str(self.private)})
        self.prior=local_prior();save(self.private/'local-prior-before.json',self.prior)

    def command(self,label,argv,timeout,script=None):
        assert not (self.raw/(label+'.json')).exists(),'stage already recorded; no retry: '+label
        timeout=min(timeout,max(1,int(self.deadline-time.monotonic())))
        assert time.monotonic()<self.deadline,'total workflow bound reached'
        identity=sha(Path(__file__).read_bytes());frozen=self.private/('controller-'+identity+'.py')
        if not frozen.exists():frozen.write_bytes(Path(__file__).read_bytes())
        if script is not None:(self.raw/(label+'.ps1')).write_text(script)
        started=now();clock=time.monotonic();timedout=False
        try:
            p=subprocess.run(argv,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=timeout)
            out,err,code=p.stdout,p.stderr,p.returncode
        except subprocess.TimeoutExpired as e:out,err,code,timedout=e.stdout or b'',e.stderr or b'',None,True
        (self.raw/(label+'.stdout')).write_bytes(out);(self.raw/(label+'.stderr')).write_bytes(err)
        shown=[x.replace(self.alias,'<SSH_TRANSPORT>').replace(str(self.private),'<LOCAL_PRIVATE>') for x in argv]
        if script is not None:shown[-1]='<UTF16LE_BASE64_OF_'+label+'.ps1>'
        receipt={'command':shown,'signature':sha(json.dumps(argv).encode()),'controller_sha256':identity,'started_local':started,'ended_local':now(),'elapsed_seconds':time.monotonic()-clock,'timeout_seconds':timeout,'exit_code':code,'timed_out':timedout,'stdin':'DEVNULL immediate EOF','script_sha256':sha(script.encode()) if script else None,'stdout_sha256':sha(out),'stderr_sha256':sha(err)}
        save(self.raw/(label+'.json'),receipt);print(label,'exit=',code,'timed_out=',timedout,flush=True)
        if timedout and script is not None:self.recover_transport_owner(script)
        return receipt,out

    def remote(self,label,script,timeout=45):
        encoded=base64.b64encode(script.encode('utf-16le')).decode()
        return self.command(label,['/usr/bin/ssh',*OPTIONS,self.alias,'powershell.exe','-NoLogo','-NoProfile','-NonInteractive','-EncodedCommand',encoded],timeout,script)

    def recover_transport_owner(self,original):
        encoded=base64.b64encode(original.encode('utf-16le')).decode()
        script=r'''$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'
$all=@(Get-CimInstance Win32_Process)
$owners=@($all|Where-Object {$_.Name -eq 'powershell.exe' -and $_.CommandLine -and $_.CommandLine.EndsWith('__ENCODED__')})
if($owners.Count -eq 0){Write-Output 'NO_EXACT_TRANSPORT_OWNER_REMAINS';exit 0}
if($owners.Count -ne 1){throw 'exact transport owner not unique; no kill'}
$known=@{};$known[[int]$owners[0].ProcessId]=$owners[0].CreationDate
for($pass=0;$pass -lt 12;$pass++){foreach($p in $all){if($known.ContainsKey([int]$p.ParentProcessId) -and $p.CreationDate -ge $known[[int]$p.ParentProcessId]){$known[[int]$p.ProcessId]=$p.CreationDate}}}
$stopped=@(foreach($p in $all|Sort-Object CreationDate -Descending){if($known.ContainsKey([int]$p.ProcessId)){$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$p.ProcessId);if($live -and $live.CreationDate -eq $p.CreationDate -and $live.CommandLine -eq $p.CommandLine){Stop-Process -Id $p.ProcessId -ErrorAction Stop;[ordered]@{pid=$p.ProcessId;created=$p.CreationDate;path=$p.ExecutablePath;command_line=$p.CommandLine;exact_owner_or_descendant=$true}}}})
$stopped|ConvertTo-Json -Depth 8 -Compress
'''.replace('__ENCODED__',encoded)
        rec,_=self.remote('transport-owned-recovery',script,30)
        assert rec['exit_code']==0,'bounded owned transport recovery unverified'

    def stage(self,name,timeout):
        script="$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'; & 'C:\\Program Files\\PowerShell\\7\\pwsh.exe' -NoLogo -NoProfile -NonInteractive -Command \"try { & '"+self.root+'\\'+name+".ps1' } catch { [IO.File]::WriteAllText('"+self.root+'\\'+name+"-first-stage-failure.stderr', (\u0024_|Out-String)); exit 1 }\"; exit $LASTEXITCODE"
        # -EncodedCommand below protects the outer command. A single-quoted
        # inner command prevents Windows PowerShell expanding pwsh's $_.
        inner="try { & '"+self.root+'\\'+name+".ps1' } catch { [IO.File]::WriteAllText('"+self.root+'\\'+name+"-first-stage-failure.stderr', ($_|Out-String)); exit 1 }"
        inner64=base64.b64encode(inner.encode('utf-16le')).decode()
        script="$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'; & 'C:\\Program Files\\PowerShell\\7\\pwsh.exe' -NoLogo -NoProfile -NonInteractive -EncodedCommand "+inner64+"; exit $LASTEXITCODE"
        return self.remote(name,script,timeout)

    def freeze(self):
        assert git('rev-parse','HEAD').decode().strip()==SOURCE
        assert not git('diff','--name-only',SOURCE,'--','Cargo.toml','Cargo.lock','rust-toolchain.toml','rustfmt.toml','crates','tools/e2e','vendor')
        paths=['Cargo.toml','Cargo.lock','rust-toolchain.toml','rustfmt.toml','crates','tools/e2e','vendor']
        archive=git('archive','--format=tar',SOURCE,'--',*paths);entries={}
        for line in git('ls-tree','-rz',SOURCE,'--',*paths).split(b'\0'):
            if line:
                meta,path=line.split(b'\t',1);mode,kind,oid=meta.decode().split()
                assert kind=='blob' and mode in ('100644','100755')
                entries[path.decode()]=oid
        files=[];seen=set()
        with tarfile.open(fileobj=io.BytesIO(archive),mode='r:') as tar:
            for item in tar.getmembers():
                p=PurePosixPath(item.name);assert not p.is_absolute() and '..' not in p.parts and (item.isdir() or item.isfile())
                if item.isfile():
                    assert item.name not in seen;seen.add(item.name)
                    data=tar.extractfile(item).read();oid=entries.pop(item.name)
                    assert git('cat-file','blob',oid)==data and (REPO/item.name).read_bytes()==data
                    files.append({'path':item.name,'bytes':len(data),'sha256':sha(data),'git_blob':oid,'crlf_pairs':data.count(b'\r\n'),'lf_bytes':data.count(b'\n'),'archive_equals_git_blob':True})
        assert not entries
        module=next(x['sha256'] for x in files if x['path']=='crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs')
        manifest={'schema':'openergy.windows-build-source.v1','source_commit':SOURCE,'source_tree':git('rev-parse',SOURCE+'^{tree}').decode().strip(),'module_sha256':module,'archive_bytes':len(archive),'archive_sha256':sha(archive),'export_paths':paths,'files':files,'overlays':[],'line_ending_conversion':False}
        (self.private/'source.tar').write_bytes(archive);save(self.private/'source-manifest.json',manifest);save(HERE/'source-manifest.json',manifest)
        self.result['archive_sha256']=sha(archive);self.result['source_files']=len(files)
        print('SOURCE_FROZEN',len(files),flush=True)

    def setup(self):
        script=r'''$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue'
(Get-Process -Id $PID).PriorityClass='BelowNormal'
if((Get-Process -Id $PID).SessionId -ne 0){throw 'session 0 required'}
if(@(Get-Process -Name cargo,rustc,cargo-clippy,clippy-driver -ErrorAction SilentlyContinue).Count){throw 'another native build active'}
foreach($p in @('C:\Program Files\PowerShell\7\pwsh.exe',(Join-Path $env:USERPROFILE '.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/bin/cargo.exe'),'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe','C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64\kernel32.lib')){if(-not (Test-Path -LiteralPath $p -PathType Leaf)){throw 'installed prerequisite absent'}}
$root=Join-Path $env:TEMP '__NAME__'
if(Test-Path -LiteralPath $root){throw 'fresh owned root exists; no retry'}
New-Item -ItemType Directory -Path $root|Out-Null
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
$acl=[Security.AccessControl.DirectorySecurity]::new();$acl.SetAccessRuleProtection($true,$false);$acl.SetOwner($sid)
$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow'))
Set-Acl -LiteralPath $root -AclObject $acl
$a=Get-Acl -LiteralPath $root
if(-not $a.AreAccessRulesProtected -or $a.Owner -ne $sid.Translate([Security.Principal.NTAccount]).Value -or @($a.Access).Count -ne 1){throw 'protected current-user-owned ACL required'}
[ordered]@{root=$root;original_temp=$env:TEMP;profile=$env:USERPROFILE;session=(Get-Process -Id $PID).SessionId;priority=(Get-Process -Id $PID).PriorityClass.ToString();created_utc=[DateTime]::UtcNow.ToString('o');root_acl_protected=$a.AreAccessRulesProtected}|ConvertTo-Json -Compress
'''.replace('__NAME__',self.remote_name)
        rec,out=self.remote('setup',script);assert rec['exit_code']==0,'fresh setup failed'
        self.state=json.loads(out.decode('utf-8-sig'));self.root=self.state['root']
        old=json.loads((self.old_private/'state.json').read_text());self.state['prior_fixtures']=old['prior_fixtures']
        save(self.private/'state.json',self.state)
        for name in ('common.ps1','qualify.ps1','build.ps1','observer.cs','runtime.ps1','postflight.ps1'):
            data=(HERE/name).read_text().replace('__NONCE__',self.nonce).encode()
            (self.private/name).write_bytes(data)
        files=['source.tar','source-manifest.json','state.json','common.ps1','qualify.ps1','build.ps1','observer.cs','runtime.ps1','postflight.ps1']
        manifest={'files':[{'path':n,'bytes':(self.private/n).stat().st_size,'sha256':sha((self.private/n).read_bytes())} for n in files]}
        save(self.private/'transfer-manifest.json',manifest);save(HERE/'workflow-bindings.json',{'nonce':self.nonce,'files':[e for e in manifest['files'] if e['path'] not in ('source.tar','state.json')]})
        rec,_=self.command('upload',['/usr/bin/scp','-O','-q',*OPTIONS,*[str(self.private/n) for n in files+['transfer-manifest.json']],self.alias+':'+self.root.replace('\\','/')+'/'],90)
        assert rec['exit_code']==0,'fixed-file upload failed; no blind retry'

    def collect(self):
        rec,out=self.stage('postflight',600);self.result['postflight_exit']=rec['exit_code']
        bundle=json.loads(out.decode('utf-8-sig'));save(self.private/'bundle-remote.json',bundle)
        rec,_=self.command('download',['/usr/bin/scp','-O','-q',*OPTIONS,self.alias+':'+self.root.replace('\\','/')+'/evidence.tar',str(self.private/'evidence.tar')],90)
        assert rec['exit_code']==0,'evidence transfer failed; fixture retained'
        data=(self.private/'evidence.tar').read_bytes();assert len(data)==bundle['bytes'] and sha(data)==bundle['sha256']
        self.result['bundle']=bundle;seen=set()
        with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as tar:
            for item in tar.getmembers():
                p=PurePosixPath(item.name);assert not p.is_absolute() and '..' not in p.parts and item.isfile() and item.name not in seen;seen.add(item.name)
                dest=self.private/'downloaded'/item.name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(tar.extractfile(item).read())
        for e in json.loads((self.private/'downloaded/evidence-manifest.json').read_text()):
            data=(self.private/'downloaded'/e['path']).read_bytes();assert len(data)==e['bytes'] and sha(data)==e['sha256']
        build_path=self.private/'downloaded/build-result.json'
        if build_path.exists():
            build=json.loads(build_path.read_text());artifact=build.get('artifact')
            if artifact:
                data=(self.private/'downloaded/target/x86_64-pc-windows-msvc/release/openergy.exe').read_bytes();assert len(data)==artifact['bytes'] and sha(data)==artifact['sha256']
        # Removal is permitted only after independently verifying the private archive.
        script=r'''$ErrorActionPreference='Stop';(Get-Process -Id $PID).PriorityClass='BelowNormal'
$root='__ROOT__'
if((Split-Path $root -Leaf) -ne '__NAME__' -or (Split-Path $root -Parent) -ne '__TEMP__'){throw 'owned root identity mismatch'}
$owned=@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -ne $PID -and (($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)) -or ($_.CommandLine -and $_.CommandLine.Contains($root)))}|Select-Object Name,ProcessId,CreationDate,ExecutablePath,CommandLine)
if($owned.Count){throw 'owned process remains; fixture retained'}
Remove-Item -LiteralPath $root -Recurse -Force
[ordered]@{recorded_utc=[DateTime]::UtcNow.ToString('o');owned_processes_before_removal=$owned;owned_root_removed=(-not (Test-Path -LiteralPath $root));removed_only_fresh_owned_root=$true;artifact_and_raw_evidence_verified_and_retained_privately_before_removal=$true}|ConvertTo-Json -Depth 8 -Compress
'''.replace('__ROOT__',self.root).replace('__NAME__',self.remote_name).replace('__TEMP__',self.state['original_temp'])
        rec,out=self.remote('cleanup',script,90);self.result['cleanup_exit']=rec['exit_code']
        assert rec['exit_code']==0,'owned cleanup failed'
        self.result['cleanup']=json.loads(out.decode('utf-8-sig'))

    def export(self):
        current=local_prior();save(self.private/'local-prior-after.json',current)
        self.result['local_prior_evidence_equal']=current==self.prior
        replacements=[(self.alias,'<SSH_TRANSPORT>'),(str(self.private),'<LOCAL_PRIVATE>'),(str(self.old_private),'<R6_PRIVATE>')]
        if self.state:
            p=self.state['profile'];replacements += [(p.replace('\\','\\\\'),'<PROFILE_JSON>'),(p,'<PROFILE>'),(p.replace('\\','/'),'<PROFILE_FORWARD_SLASH>')]
        def mask(s):
            for old,new in replacements:s=s.replace(old,new)
            return s
        ledger=[]
        for folder in (self.raw,self.private/'downloaded'):
            if not folder.exists():continue
            for p in sorted(folder.rglob('*')):
                if not p.is_file() or p.suffix not in ('.json','.ps1','.cs','.stdout','.stderr','.guard','.toml','.log'):continue
                original=p.read_bytes();rel=Path(folder.name)/p.relative_to(folder)
                omit=p.name in ('shared-before.json','shared-after.json','source-manifest.json')
                copied=mask(original.decode('utf-8-sig')).encode()
                if not omit:
                    dest=HERE/'evidence'/rel;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(copied)
                ledger.append({'path':str(rel),'original_bytes':len(original),'original_sha256':sha(original),'committed_copy':not omit,'copy_bytes':len(copied),'copy_sha256':sha(copied),'byte_equal':original==copied})
        save(HERE/'copy-ledger.json',{'method':'Exact raw originals retained privately. Derived UTF-8 copies remove BOM and replace known transport/profile/private-root prefixes only; preserve line endings. Full shared snapshots and duplicate source manifest omitted.','files':ledger})
        for name in ('shared-before.json','shared-after.json'):
            path=self.private/'downloaded'/name
            if path.exists():
                data=path.read_bytes();value=json.loads(data.decode('utf-8-sig'))
                save(HERE/(name.replace('.json','-summary.json')),{'bytes':len(data),'sha256':sha(data),'trees':[{'label':x['label'],'files':x['files'],'items':x['items'],'entries_sha256':sha(json.dumps(x['entries'],sort_keys=True).encode())} for x in value['trees']],'default_fields':len(value['defaults']),'tool_fields':len(value['tools']),'sharing_identity_count':len(value['sharing']),'old_failed_root_absent':value['old_failed_root_absent']})
        save(HERE/'local-prior-preservation.json',{'equal':current==self.prior,'before_sha256':sha(json.dumps(self.prior,sort_keys=True).encode()),'after_sha256':sha(json.dumps(current,sort_keys=True).encode()),'roots':[{'owner':x['owner'],'files':len(x['files'])} for x in self.prior],'full_manifests_retained_privately':True})
        save(self.private/'result.json',self.result);save(HERE/'result.json',json.loads(mask(json.dumps(self.result))))

    def run(self):
        self.freeze();self.setup()
        rec,_=self.stage('qualify',600);self.result['qualification_exit']=rec['exit_code']
        if rec['exit_code']==0:
            self.result['compiler_stage_attempts']+=1
            rec,_=self.stage('build',1250);self.result['build_exit']=rec['exit_code']
            if rec['exit_code']==0:
                self.result['runtime_stage_attempts']+=1
                rec,_=self.stage('runtime',150);self.result['runtime_exit']=rec['exit_code']
        self.collect()
        self.result['passed']=all(self.result.get(x)==0 for x in ('qualification_exit','build_exit','runtime_exit','postflight_exit','cleanup_exit')) and self.result['cleanup']['owned_root_removed']

if __name__=='__main__':
    workflow=Controller();code=0
    try:workflow.run()
    except BaseException:
        code=1;raw=traceback.format_exc();(workflow.raw/'controller-first-failure.stderr').write_text(raw);workflow.result['failure']=raw.splitlines()[-1];print(workflow.result['failure'],flush=True)
    finally:
        workflow.result['ended_local']=now();workflow.result['elapsed_seconds']=time.monotonic()-workflow.started;workflow.result['controller_exit']=code
        workflow.export();print('NATIVE_DAEMON_OFF_RESULT','PASS' if workflow.result['passed'] else 'FAIL_OR_UNVERIFIED',flush=True)
    raise SystemExit(code if workflow.result['passed'] else 1)
