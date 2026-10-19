# nf-lobbyd — the optional lobby directory for a home server

`nf-lobbyd` lists open NightfirePort lobbies so players outside your network can find them from Join Online. It only
lists: the games themselves stay peer-to-peer (docs/systems/net-p2p.md). Protocol: `crates/nf-net/src/directory.rs`;
the game side: docs/systems/net-lobby-list.md §3.

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

## Checks

`crates/nf-net/tests/lobby_list.rs::the_directory_lists_a_registered_host` (a server loop, a registering host, a
listing joiner, on loopback) and the unit test in `directory.rs` (register / list / expire / split across datagrams).
