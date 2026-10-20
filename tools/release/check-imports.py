"""Fails when a release exe imports, at load time, a DLL that Windows would look for in the exe's own folder first.

A lone NightfirePort.exe often runs from Downloads, beside other programs' DLLs: on 2026-10-19 a game mod's
`dxgi.dll` there loaded into the launcher and popped Unreal "signature failed" boxes. The allowed load-time imports
are the KnownDLLs (always System32) and the api-ms-win-* API sets; everything else must be delay-loaded
(crates/nf-view/build.rs), and its first load comes after `main`'s SetDefaultDllDirectories(System32).

Usage: python -I check-imports.py <exe>"""
import struct
import sys

# The KnownDLLs (HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\KnownDLLs, Windows 10/11) plus ntdll /
# kernel32's own forwarders: the loader maps these from System32 regardless of the exe's folder.
KNOWN = {
    "advapi32.dll", "bcrypt.dll", "bcryptprimitives.dll", "cfgmgr32.dll", "clbcatq.dll", "combase.dll",
    "comdlg32.dll", "coml2.dll", "difxapi.dll", "gdi32.dll", "gdi32full.dll", "gdiplus.dll", "imagehlp.dll",
    "imm32.dll", "kernel32.dll", "kernelbase.dll", "msctf.dll", "msvcp_win.dll", "msvcrt.dll", "normaliz.dll",
    "nsi.dll", "ntdll.dll", "ole32.dll", "oleaut32.dll", "psapi.dll", "rpcrt4.dll", "sechost.dll", "setupapi.dll",
    "shcore.dll", "shell32.dll", "shlwapi.dll", "ucrtbase.dll", "user32.dll", "win32u.dll", "wldap32.dll",
    "wow64.dll", "wow64win.dll", "ws2_32.dll",
}


def imports(path):
    d = open(path, "rb").read()
    pe = struct.unpack_from("<I", d, 0x3C)[0]
    nsec = struct.unpack_from("<H", d, pe + 6)[0]
    opt = pe + 24
    magic = struct.unpack_from("<H", d, opt)[0]
    dd = opt + (112 if magic == 0x20B else 96)
    sec0 = opt + struct.unpack_from("<H", d, pe + 20)[0]

    def off(rva):
        for i in range(nsec):
            s = sec0 + 40 * i
            vs, va, ro = struct.unpack_from("<III", d, s + 8)[0], struct.unpack_from("<I", d, s + 12)[0], struct.unpack_from("<I", d, s + 20)[0]
            if va <= rva < va + max(vs, 1):
                return rva - va + ro
        raise ValueError(f"rva {rva:#x} outside the sections")

    rva = struct.unpack_from("<I", d, dd + 8)[0]  # data directory 1: the import table
    out, o = [], off(rva)
    while True:
        name_rva = struct.unpack_from("<I", d, o + 12)[0]
        if not name_rva:
            break
        n = off(name_rva)
        out.append(d[n:d.index(b"\0", n)].decode().lower())
        o += 20
    return out


def main():
    bad = [n for n in imports(sys.argv[1]) if n not in KNOWN and not n.startswith(("api-ms-win-", "ext-ms-win-"))]
    if bad:
        print("load-time imports that resolve from the exe's folder first:", ", ".join(bad))
        sys.exit(1)
    print("imports ok (KnownDLLs and API sets only)")


if __name__ == "__main__":
    main()
