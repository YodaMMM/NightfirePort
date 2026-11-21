#!/usr/bin/env python3
"""Two-instance online check on one PC: a host and a joiner over 127.0.0.1.

Runs the real nf-view exe twice (`--net-host <port> <arena>` and `--net-join 127.0.0.1:<port>`),
each in its own working directory (NF_NET_SHOT captures land in <dir>/nf-shots), with LAN-only
sessions (NF_NO_REACH=1: no UPnP / STUN; NF_LOBBY_SERVICE empty: no cloud listing) and silent
audio. Both windows are killed after --seconds. stdout/stderr go to <dir>/out.txt.

Per-instance env: --host-env K=V and --join-env K=V (repeatable); --env K=V goes to both.
The useful test aids (crates/nf-view/src/lib.rs): NF_MP_FACE_OTHER, NF_MP_DUEL, NF_NET_AUTOFIRE,
NF_NET_AUTOPICK + NF_NET_AUTOPICK_ONLY, NF_NET_SHOT, NF_NET_DROPLOG, NF_NET_TIME, NF_PICKUPLOG.

Example (a death drop seen on both machines):
  python audits/tools-wip/mp_two_instance.py --exe target-o/release/nf-view.exe --out zz_2inst/run1 \
    --env NF_MP_FACE_OTHER=1 --env NF_MP_DUEL=5 --env NF_NET_DROPLOG=60 \
    --host-env NF_NET_AUTOFIRE=120-700 --env NF_NET_SHOT=300,900
"""
import argparse, os, shutil, subprocess, sys, time

ISO = os.environ.get("NF_ISO", r"C:\Users\Yoda\Downloads\PS2\007 - Nightfire (USA).iso")


def kv(items):
    out = {}
    for it in items or []:
        k, _, v = it.partition("=")
        out[k] = v
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--arena", default="0x07000023")
    ap.add_argument("--port", default="27431")
    ap.add_argument("--seconds", type=float, default=60)
    ap.add_argument("--env", action="append")
    ap.add_argument("--host-env", action="append")
    ap.add_argument("--join-env", action="append")
    a = ap.parse_args()

    out = os.path.abspath(a.out)
    exe_src = os.path.abspath(a.exe)
    os.makedirs(out, exist_ok=True)
    # A private copy: a rebuild during the run cannot swap the binary under the windows.
    exe = os.path.join(out, "nf-view.exe")
    shutil.copy2(exe_src, exe)

    base = dict(os.environ)
    base.update({"NF_SILENT": "1", "NF_NULL_MIXER": "1", "NF_NO_REACH": "1", "NF_LOBBY_SERVICE": "",
                 "NF_ISO": ISO})
    base.update(kv(a.env))
    procs = []
    for role, extra, args in (
        ("host", kv(a.host_env), ["--net-host", a.port, a.arena]),
        ("join", kv(a.join_env), ["--net-join", f"127.0.0.1:{a.port}"]),
    ):
        d = os.path.join(out, role)
        os.makedirs(d, exist_ok=True)
        cfg = os.path.join(d, "cfg")
        os.makedirs(cfg, exist_ok=True)
        env = dict(base)
        env["NF_CONFIG_DIR"] = cfg
        env.update(extra)
        f = open(os.path.join(d, "out.txt"), "w", encoding="utf-8", errors="replace")
        p = subprocess.Popen([exe, ISO] + args, cwd=d, env=env, stdout=f, stderr=subprocess.STDOUT)
        procs.append((role, p, f))
        print(f"[2inst] {role} pid {p.pid}: {' '.join(args)}", flush=True)
        if role == "host":
            time.sleep(4)  # the host binds before the joiner dials
    t0 = time.time()
    while time.time() - t0 < a.seconds and any(p.poll() is None for _, p, _ in procs):
        time.sleep(1)
    for role, p, f in procs:
        if p.poll() is None:
            p.kill()
            p.wait()
        f.close()
        print(f"[2inst] {role} exit {p.returncode}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
