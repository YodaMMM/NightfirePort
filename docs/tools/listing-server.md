# nf-lobbyd — the optional lobby directory for a home server

`nf-lobbyd` lists open NightfirePort lobbies so players outside your network can find them from Join Online, and —
since net-short-codes — hands every host a **4-character join code** (docs/systems/net-short-codes.md) so nobody has
to type the 16-symbol self-contained code. It never relays game traffic: the games themselves stay peer-to-peer
(docs/systems/net-p2p.md). Protocol: `crates/nf-net/src/directory.rs`; the game side: docs/systems/net-lobby-list.md
§3.

## Build

    cargo build --release -p nf-net --bin nf-lobbyd

The binary is `target/release/nf-lobbyd` (`.exe` on Windows). std only, no config file, no disk writes.

## Run

    nf-lobbyd                 # listens on UDP 0.0.0.0:27500
    nf-lobbyd 0.0.0.0:27600   # another port

It prints one line when it starts and one whenever the number of listed lobbies changes. A lobby drops out 20 s after
its host stops refreshing it (hosts refresh every 5 s). At most 256 lobbies are kept.

### As a service

- **Linux (systemd):** `/etc/systemd/system/nf-lobbyd.service`

      [Unit]
      Description=NightfirePort lobby directory
      After=network-online.target

      [Service]
      ExecStart=/opt/nf-lobbyd/nf-lobbyd 0.0.0.0:27500
      Restart=always
      DynamicUser=yes

      [Install]
      WantedBy=multi-user.target

  then `systemctl enable --now nf-lobbyd`.
- **Windows:** a Task Scheduler task "At startup", action `nf-lobbyd.exe 0.0.0.0:27500`, "Run whether user is logged
  on or not". Allow `nf-lobbyd.exe` through Windows Defender Firewall for UDP.

## Router

Forward **UDP 27500** (or your chosen port) on the router to the home server's LAN address. Nothing else is needed for
the directory; each game host opens its own game port (UPnP / NAT-PMP, or a manual forward).

## Point the game at it

On every player's PC, either set the environment variable `NF_LOBBY_SERVER=<your server's public name or IP>[:port]`
or create `%APPDATA%\NightfirePort\net.txt` with one line:

    lobby_server = your.server.address:27500

Empty or missing = no directory (LAN lobbies and join codes still work). The address is never built into the game.
Hosts then register automatically while hosting, and Join Online lists the directory's lobbies beside the LAN ones.

## Short codes (net-short-codes)

A host whose REGISTER is answered gets a 4-character code (e.g. `H4XK`) from the unambiguous alphabet `23456789
ABCDEFGHJKMNPQRSTVWXYZ` — no 0/O, no 1/I/L — and its lobby shows that instead of the 19-character code. The join box
takes either form (case does not matter; dashes and spaces are ignored): 4 symbols are looked up on the joiner's own
lobby server, which answers with the host's public endpoint as the server saw it (plus the host's LAN endpoint, so
the same-network path still works) and the joiner dials it like a long code.

- The code is unique among the live entries, stays the same while the host keeps registering, and dies with the
  entry (20 s after the host stops refreshing).
- With no server set, or when the server does not answer, hosts show the long code as before — nothing breaks.
- LOOKUP is rate limited per source address: 30 lookups per 10 s (the codes are brute-forceable at 810 k). Requests
  with a protocol version this server does not know are answered "unsupported" cleanly.

**Upgrading an existing deployment:** rebuild and restart `nf-lobbyd` (`cargo build --release -p nf-net --bin
nf-lobbyd`, then restart the service as above). Nothing else changes: no config file, the same UDP port, and the
protocol is compatible in both directions — an old client registers and lists exactly as before (it just never asks
for a code), and a new client pointed at an old server falls back to the long code, its 4-symbol lookups answered
by silence it treats as "no answer".

## Checks

`crates/nf-net/tests/lobby_list.rs::the_directory_lists_a_registered_host` (a server loop, a registering host, a
listing joiner, on loopback) and the unit test in `directory.rs` (register / list / expire / split across datagrams).
The short-code path: `crates/nf-net/tests/short_codes.rs` (7 tests: register → 4-character code, LOOKUP → a real
session handshake on 127.0.0.1, clear not-found, old-server silence and old-host compatibility, "unsupported" for an
unknown version, the per-source rate limit, and the display fallback when the server stops answering).
