# Windows route and installed pinned tools

Two bounded read-only SSH metadata commands succeeded with strict host-key checks.
The target reports Windows 10.0.26340.0 and PowerShell 5.1.26100.9750, session 0.
Both command processes ran BelowNormal. The installed default toolchain is 1.98.0.
A command-local override confirmed installed Rust/Cargo 1.99.1 without downloading
or changing defaults. Auto-install and network use by Cargo were disabled.

No candidate controller, compiler build, daemon launch, stop request or native
flow ran. The native Off stop path remains UNVERIFIED. The old failed run with
zero stop requests remains unchanged. A separate fresh-source controller repair
is under independent review before any new native attempt.

[result.json](result.json) binds the private metadata receipts. The raw stderr
and stdout remain private. No endpoint, account, authentication material or
transport selector is published. This check establishes reachability and named
tool versions only; it does not establish every Windows launcher layer or any
native lifecycle behavior.
