# net-p2p — internet play without a server (UPnP / NAT-PMP / STUN / join codes / hole punching)

Evidence tags: [S] port-only design (the PS2 game has no online mode at all [D], docs/systems/net-online-lobby.md §1),
[R] an RFC / UPnP spec (cited by number and section). Nothing here is a game claim.

Peer-to-peer joining with UPnP on the host. Code: `crates/nf-net/src/p2p/`
(std only, no new crates). The session layer (net-lockstep.md) is unchanged: star topology, clients talk to the
host. This layer only makes the host's UDP port reachable from the internet and tells each side what to type.

## 1. What the host does (in order, first success wins)

| step | how | result state |
|---|---|---|
| 1. UPnP IGD | SSDP `M-SEARCH` to 239.255.255.250:1900 for `InternetGatewayDevice:1/2` and the `WANIPConnection` / `WANPPPConnection` services [R UPnP Device Architecture 1.1 §1.3.2]; HTTP GET the `LOCATION` description; SOAP `AddPortMapping` (UDP, external = internal = the game port, lease 3600 s, description `NightfirePort`), `GetExternalIPAddress` [R UPnP IGD WANIPConnection:1 §2.4.16/§2.4.18]. A router that refuses non-zero leases (error 725 `OnlyPermanentLeasesSupported`) gets lease 0 and the mapping is deleted on exit. A conflict (718) retries with the next 9 external ports. | `Mapped(Upnp)` |
| 2. NAT-PMP | UDP to the gateway on 5351: opcode 0 (external address), opcode 1 (map UDP, lifetime 3600) [R RFC 6886 §3.2/§3.3]. The gateway is the UPnP device's host when SSDP answered, else the LAN address's x.y.z.1 [S heuristic — std cannot read the routing table]. PCP (RFC 6887) is NOT implemented: most PCP routers also answer NAT-PMP v0 requests with a version error, which we report as "no mapping". | `Mapped(NatPmp)` |
| 3. STUN | a Binding request (RFC 5389 §6, magic cookie 0x2112A442) from the GAME socket to the configured STUN servers (setting; defaults are public Google/Cloudflare servers); the XOR-MAPPED-ADDRESS (§15.2) is the public endpoint. Two servers answering with different mapped ports = endpoint-dependent mapping ("symmetric NAT"). | `StunOnly` (+ `symmetric`) |
| — | nothing answered | `Failed` |

Renewal: a mapping is refreshed at half its lease; `Reach::close` deletes it (UPnP `DeletePortMapping`, NAT-PMP
lifetime 0). A crash leaves at most one hour of lease behind.

STUN shares the game socket: `Session::send_raw` sends from it and `Session::take_stray` hands back the datagrams
that are not session packets (a STUN response's first two bits are 00 and its magic cookie sits at offset 4 — it can
never decode as an `NF` packet, whose first bytes are 0x4E 0x46).

## 2. Join codes [S]

`XXXX-XXXX-XXXX-XXXX`: 10 bytes in Crockford base32 (16 symbols, no I/L/O/U; decoding accepts lower case, maps
I/L→1 and O→0, ignores dashes and spaces).

| bytes | field |
|---|---|
| 0..4 | IPv4 address, network order |
| 4..6 | UDP port, big-endian |
| 6 | `PROTOCOL_VERSION` (low byte) |
| 7 | flags: bit 0 = the sender has no port mapping (the other side must punch), bit 1 = symmetric NAT |
| 8..10 | CRC-16/CCITT-FALSE over bytes 0..8 |

The join page accepts a code or a raw `a.b.c.d:port`. A code with a bad checksum or another protocol version is
refused with that reason (not "timed out").

## 3. Hole punching [S, the classic UDP technique — RFC 5128 §3.3]

Used when the host is `StunOnly` (no mapping). There is no introducer server, so the two codes are swapped by hand
(chat, voice): the joiner's page shows **its own** code too (its STUN endpoint). The host types the joiner's code into
the lobby's "expect player" row; from then both sides send:

- the joiner: its normal session HELLO retries to the host's public endpoint (every 400 ms),
- the host: `PUNCH` datagrams (`4E 46 50 00` + 8-byte nonce; the session decoder ignores the unknown type 0x50) to the
  joiner's public endpoint every 100 ms for 15 s.

Each side's outgoing datagrams open its own NAT for the other's endpoint; the first HELLO that gets through
connects as usual. Both sides symmetric → `Punch::Failed(SymmetricBoth)`: a relay would be needed (out of scope —
the UI says so). One side symmetric → it is tried anyway (port prediction is not attempted).

## 4. State the front end reads

```
HostReach { state: Probing | Mapped(Upnp|NatPmp) | StunOnly | Failed,
            public: Option<SocketAddrV4>, code: Option<String>, symmetric: bool, local_port: u16, note: String }
JoinReach { state: Resolving | Connecting | Punching | Connected | Failed(reason), my_code: Option<String> }
```

## 5. Optional lobby directory (700v4 item 5)

Not built in this pass (items 1-4 first). The listing address will be a setting, default empty; the server is the
player's own and is never hard-coded.

## 6. Tests

`crates/nf-net/tests/p2p_*.rs` + unit tests in the modules: STUN encode/decode against the RFC 5769 §2.2 IPv4 sample
response (its FINGERPRINT CRC-32 is checked too, so a mistyped vector fails); join-code round trip, checksum and
version rejection; a fake IGD on 127.0.0.1 (canned SSDP reply, device XML, SOAP responses) driven through the real
HTTP/SOAP client; a fake NAT-PMP gateway on 127.0.0.1; the punch state machine on loopback sockets.

## 7. Live checks

- 2026-10-19, a home router (UPnP IGD at 192.168.1.1:5000): `cargo run -p nf-net --example upnp_probe` —
  SSDP found the IGD, `AddPortMapping` UDP 27420 (lease 60 s) succeeded, `GetExternalIPAddress` matched the public
  address, `DeletePortMapping` removed it [L].
- Same day, two game instances on that PC (`--net-host 27420` with reach on; the joiner dialled the host's printed join
  code): the host showed "Internet: open (UPnP)" and its code; the joiner decoded the code, saw the same public address
  as its own and dialled the host's LAN address (the same-network path), the match started with no desync [L]. A join
  from a different network is not yet tested (needs a second site).
