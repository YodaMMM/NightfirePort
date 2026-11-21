#!/usr/bin/env python3
"""Real-time NDJSON telemetry receiver for NightfirePort Android (over `adb reverse tcp:9876 tcp:9876`).

Telemetry is off by default: use a debug build and launch it with
`adb shell am start -n com.nightfire.android/.MainActivity --ez nf_telemetry true`.
Listens on 127.0.0.1 only.
"""

import json
import os
import socket
import sys
import time

HOST = "127.0.0.1"
PORT = int(os.environ.get("NF_TELEMETRY_PORT", "9876"))
LOG_PATH = os.environ.get("NF_TELEMETRY_LOG", "work/android_telemetry.ndjson")


def fmt_event(ev: dict) -> str:
    kind = ev.get("ev", "?")
    if kind == "boot":
        return f"[BOOT] archive={ev.get('archive')} ok={ev.get('ok')} ms={ev.get('ms')} path={ev.get('data_path')}"
    if kind == "surface":
        return f"[GPU]  {ev.get('w')}x{ev.get('h')} fmt={ev.get('format')} {ev.get('gpu', '')}"
    if kind == "level_load":
        if ev.get("ok"):
            return (
                f"[LOAD] {ev.get('archive')} OK in {ev.get('ms')}ms "
                f"(cels={ev.get('cels')} tex={ev.get('textures')} sim={ev.get('sim')})"
            )
        return f"[LOAD] {ev.get('archive')} FAIL in {ev.get('ms')}ms: {ev.get('err')}"
    if kind == "frame":
        p = ev.get("pos", [0, 0, 0])
        return (
            f"[FRM]  {ev.get('archive')} t={ev.get('tick')} "
            f"fps={ev.get('fps', 0):.1f} render={ev.get('render_ms', 0):.1f}ms "
            f"page={ev.get('page')} pos=({p[0]:.1f},{p[1]:.1f},{p[2]:.1f})"
        )
    if kind == "panic":
        return f"[PANIC] {ev.get('msg')}"
    return json.dumps(ev)


def main() -> None:
    os.makedirs(os.path.dirname(LOG_PATH) or ".", exist_ok=True)
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind((HOST, PORT))
    srv.listen(4)
    print(f"Listening for device telemetry on {HOST}:{PORT} -> {LOG_PATH}", flush=True)
    print("Run: adb reverse tcp:9876 tcp:9876", flush=True)

    with open(LOG_PATH, "a", encoding="utf-8") as logf:
        while True:
            conn, addr = srv.accept()
            print(f"--- device connected ({addr[0]}:{addr[1]}) at {time.strftime('%H:%M:%S')} ---", flush=True)
            with conn, conn.makefile("r", encoding="utf-8", errors="replace") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    logf.write(line + "\n")
                    logf.flush()
                    try:
                        ev = json.loads(line)
                        print(fmt_event(ev), flush=True)
                    except json.JSONDecodeError:
                        print(f"[RAW] {line}", flush=True)
            print("--- device disconnected ---", flush=True)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(0)
