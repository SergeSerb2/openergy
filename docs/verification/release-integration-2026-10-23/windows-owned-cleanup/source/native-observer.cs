using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Net;
using System.Runtime.InteropServices;

public sealed class OwnedUdpRow {
    public int Family;
    public string Address;
    public uint Scope;
    public int Port;
    public uint ProcessId;
}
public sealed class UdpFamilyQuery {
    public int Family;
    public int[] NativeReturnCodes;
    public int BufferBytes;
    public int Rows;
}
public sealed class OwnedUdpSnapshot {
    public OwnedUdpRow[] Rows;
    public UdpFamilyQuery[] Queries;
}
public static class OwnedNativeObserver {
    [DllImport("iphlpapi.dll", SetLastError=false)]
    private static extern uint GetExtendedUdpTable(IntPtr table, ref int size, bool order, int family, int tableClass, uint reserved);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern IntPtr OpenThread(uint access, bool inherit, uint id);
    [DllImport("kernel32.dll")]
    private static extern int GetThreadDescription(IntPtr thread, out IntPtr description);
    [DllImport("kernel32.dll")]
    private static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll")]
    private static extern IntPtr LocalFree(IntPtr pointer);
    public static string DescribeThread(uint id) {
        IntPtr handle = OpenThread(0x0800, false, id); // THREAD_QUERY_LIMITED_INFORMATION, read only.
        if (handle == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        try {
            IntPtr description;
            int code = GetThreadDescription(handle, out description);
            if (code < 0) Marshal.ThrowExceptionForHR(code);
            try { return Marshal.PtrToStringUni(description) ?? ""; }
            finally { if (description != IntPtr.Zero) LocalFree(description); }
        } finally { CloseHandle(handle); }
    }
    public static OwnedUdpSnapshot ReadUdp() {
        var rows = new List<OwnedUdpRow>();
        var queries = new List<UdpFamilyQuery>();
        foreach (int family in new int[] { 2, 23 }) {
            int size = 0;
            var codes = new List<int>();
            uint first = GetExtendedUdpTable(IntPtr.Zero, ref size, true, family, 1, 0); // UDP_TABLE_OWNER_PID.
            codes.Add(checked((int)first));
            if (first != 122 && first != 0) throw new Win32Exception(checked((int)first), "UDP size query failed");
            bool complete = false;
            for (int attempt = 0; attempt < 3; attempt++) {
                if (size < 4) throw new InvalidOperationException("UDP table buffer is smaller than its row count header");
                int allocated = size;
                IntPtr table = Marshal.AllocHGlobal(allocated);
                try {
                    uint result = GetExtendedUdpTable(table, ref size, true, family, 1, 0);
                    codes.Add(checked((int)result));
                    if (result == 122) continue; // A growing OS table; retry at the returned size, never report empty.
                    if (result != 0) throw new Win32Exception(checked((int)result), "UDP data query failed");
                    int count = Marshal.ReadInt32(table);
                    int stride = family == 2 ? 12 : 28;
                    if (count < 0 || 4L + (long)count * stride > allocated) throw new InvalidOperationException("Invalid UDP table length");
                    for (int i = 0; i < count; i++) {
                        IntPtr row = IntPtr.Add(table, 4 + i * stride);
                        byte[] bytes = new byte[family == 2 ? 4 : 16];
                        Marshal.Copy(row, bytes, 0, bytes.Length);
                        uint scope = family == 2 ? 0 : unchecked((uint)Marshal.ReadInt32(row, 16));
                        int portOffset = family == 2 ? 4 : 20;
                        int port = Marshal.ReadByte(row, portOffset) * 256 + Marshal.ReadByte(row, portOffset + 1);
                        uint pid = unchecked((uint)Marshal.ReadInt32(row, portOffset + 4));
                        rows.Add(new OwnedUdpRow { Family=family, Address=(family == 2 ? new IPAddress(bytes) : new IPAddress(bytes, (long)scope)).ToString(), Scope=scope, Port=port, ProcessId=pid });
                    }
                    queries.Add(new UdpFamilyQuery { Family=family, NativeReturnCodes=codes.ToArray(), BufferBytes=allocated, Rows=count });
                    complete = true;
                    break;
                } finally { Marshal.FreeHGlobal(table); }
            }
            if (!complete) throw new InvalidOperationException("UDP table kept growing; no successful data query");
        }
        if (rows.Count == 0) throw new InvalidOperationException("Global UDP inventory is empty; refuse an empty cleanup inference");
        return new OwnedUdpSnapshot { Rows=rows.ToArray(), Queries=queries.ToArray() };
    }
}
