# Protected Windows migration helper

The migration controller, manager, and supervisor run with a limited user token.
Only `openergy-migration-admin.ps1` may receive elevation. It accepts fixed
Synergy operations, not script paths, task names, registry paths, or code.

A LocalAppData marker, a hash stored beside a script, or the current user's
ownership does not authenticate privileged code. The helper and its containing
directory must be owned by **Administrators or SYSTEM**. Their ACLs must prevent
non-administrator changes. Each ancestor must prevent replacement, including
`DELETE_CHILD`, and must have a trusted owner. Platform ancestors may also be
owned by TrustedInstaller. Reparse points, network paths, alternate streams,
null DACLs, and writable helper directories are refused. A protected DACL with
the current admin user's SID as owner is refused: that user's filtered token
can still change the DACL through ownership.

## Provision once through a trusted administrator

This is an explicit installation trust decision. Review the helper from a fixed
commit and obtain its SHA-256 through an independently trusted channel. Do not
accept a hash or bootstrap script from the same mutable download directory as
proof. Do not elevate the repository wrapper, a per-user manager, or a downloaded
installer script. No signer or credential changes are needed.

Use a trusted Windows PowerShell 5.1 administrator console started with
`-NoProfile` and a clean process environment. In particular, remove CLR profiler,
startup hook, `DEVPATH`, and user module settings **before starting that host**.
Do not start it from an untrusted working directory. The routine controller
performs this environment cleanup before starting its fixed System32 host.
For this first provisioning step, the administrator must establish the trusted
console and review the following commands themselves.

Paste the following reviewed commands there. Set only the source data path and
independently obtained hash. They read the source once as **data**, hash those
bytes, and write those same bytes with an admin owner and protected ACL at file
creation. They do not execute the mutable source. This example refuses an
existing helper directory; review upgrades and retain `State` before replacing
an existing helper.

```powershell
$ErrorActionPreference = 'Stop'
$PSModuleAutoLoadingPreference = 'None'
$env:PSModulePath = ''
$Source = 'C:\reviewed-download\openergy-migration-admin.ps1'
$Expected = 'REPLACE_WITH_INDEPENDENTLY_VERIFIED_64_HEX_SHA256'
if ($Expected -notmatch '^[0-9A-Fa-f]{64}$') { throw 'Supply the trusted SHA-256' }
$bytes = [IO.File]::ReadAllBytes($Source)
$sha = [Security.Cryptography.SHA256]::Create()
try { $actual = [BitConverter]::ToString($sha.ComputeHash($bytes)).Replace('-','') }
finally { $sha.Dispose() }
if ($actual -ne $Expected) { throw 'Source bytes do not match the trusted hash' }
$base = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles)
$root = [IO.Path]::Combine($base,'OpenergyMigration')
# Check the destination ancestors without loading code from the source.
$item = [IO.DirectoryInfo]::new($base)
$trusted = @('S-1-5-18','S-1-5-32-544',
 'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464')
while ($item) {
 if (!$item.Exists -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unsafe ancestor' }
 $acl = $item.GetAccessControl()
 $raw = [Security.AccessControl.RawSecurityDescriptor]::new($acl.GetSecurityDescriptorBinaryForm(),0)
 if ($null -eq $raw.DiscretionaryAcl -or $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -notin $trusted) { throw 'Untrusted ancestor ACL or owner' }
 foreach ($rule in $acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])) {
  if ($rule.AccessControlType -ne 'Allow' -or ($rule.PropagationFlags -band [Security.AccessControl.PropagationFlags]::InheritOnly)) { continue }
  if ($rule.IdentityReference.Value -notin ($trusted + @('S-1-3-4')) -and (([long]$rule.FileSystemRights -band 0xffffffffL) -band 0x500D0150L)) { throw 'Ancestor allows replacement' }
 }
 $item = $item.Parent
}
if ([IO.Directory]::Exists($root) -or [IO.File]::Exists($root)) { throw 'Review the existing protected installation; do not overwrite it' }
$owner = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-544')
$dacl = [Security.AccessControl.DirectorySecurity]::new()
$dacl.SetOwner($owner); $dacl.SetAccessRuleProtection($true,$false)
foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {
 $dacl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))
}
$dacl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-32-545'),'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow'))
$null = [IO.Directory]::CreateDirectory($root,$dacl)
$facl = [Security.AccessControl.FileSecurity]::new()
$facl.SetOwner($owner); $facl.SetAccessRuleProtection($true,$false)
foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {
 $facl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','Allow'))
}
$facl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-32-545'),'ReadAndExecute','Allow'))
$destination = [IO.Path]::Combine($root,'openergy-migration-admin.ps1')
$file = [IO.FileStream]::new($destination,'CreateNew',[Security.AccessControl.FileSystemRights]::Write,'None',4096,'WriteThrough',$facl)
try { $file.Write($bytes,0,$bytes.Length); $file.Flush($true) }
finally { $file.Dispose() }
# Only now execute the protected, hash-verified bytes, without profiles/modules.
& $destination -Action Check
```

No protected installation was performed during verification. The disposable ACL
fixture exercises the same creation APIs and read-only trust command. This does
not prove a particular machine's Program Files ACLs or approve source bytes.

## Routine operation and recovery

From a normal terminal, `switch-from-synergy.ps1 -CheckBroker` checks the protected
helper and exchanges a read-only private pipe probe without elevation. `-Start`,
`-Commit`, and `-Rollback` request UAC only for the protected helper, using the
same Windows account. Credential switching is not supported. A custom
`-BrokerPath` must satisfy the same protection rules and fixed filename.

The fixed Windows PowerShell host uses `-NoProfile`, disabled module autoload,
an empty module path, and fixed protected system dependencies. It does not
compile or load a temporary DLL, execute a per-user script, or invoke a saved
scheduled task. The normal controller restores the UI with its own limited
token. Existing limited logon tasks remain managed by the per-user manager,
which authenticates their SID, limited run level, and exact single action.

Both pipe peers check the actual kernel peer PID, creation time, image, SID,
and token against the expected launched process. The pipe uses a first-instance
handle, refuses remote clients, and denies the Network SID. The client requests
Identification, and the helper does not impersonate it. A GUID and SID alone
are not process authentication.

Saved service/startup state is stored at
`%ProgramFiles%\OpenergyMigration\State\<SID>\synergy-state.v2`, under admin-only
ACLs, before cutover. It uses a bounded data format and an exclusive lock.
The helper never trusts legacy `DataDir\synergy-state.json`; a pending legacy
migration requires a human to review the original state and recover it first.

Rollback restores service/startup independently of an Openergy stop failure.
The helper sends the UI requirement, waits for the limited controller's
`UI_RESTORED` acknowledgement, and then removes state only when Openergy also
stopped successfully. UI startup failure, missing acknowledgement, caller loss,
or a failed Openergy stop keeps recovery state. The controller verifies a live,
limited, same-user UI process before acknowledging. Repeated rollback remains
available. On caller loss after cutover, the helper can restore the service but
cannot launch a limited UI alone; it keeps state for a later `-Rollback`.
Do not treat a service-only restoration as complete UI recovery.
