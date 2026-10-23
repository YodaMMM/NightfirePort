# net-cloud-code-stable — the host's 4-character code holds for the whole hosting session

The cloud lobby service (`tools/lobby-worker`, docs/systems/net-cloud-lobby.md, docs/tools/cloud-lobby.md)
assigns every hosting session a 4-character code (HXKR-style). The code must stay the same for the whole
hosting session: a friend who was given HXKR must still be able to join after the match starts.

## The symptom (the driver's live check against the deployed service, 2026-10-19)

Two-window test against the live Cloudflare service: the host got code `HXKR`, the entry listed and let a
joiner in. When the match started, the host's reach line showed code `CSVD` — the host re-registered under a
new code at the match start. (Observed against the deployed service; not reproducible offline.)

## The mechanism [S port design, cited to the code]

- The service keys each entry by the requester's IP + the advert's port: `const key = \`${ip}:${port}\``
  (tools/lobby-worker/src/handler.ts, POST /v1/host). An unknown key gets a fresh code
  (`assignCode`); only the SAME key refreshes into the same code.
- The client's POST body names the port a joiner must dial: the STUN-observed public UDP mapping's port
  when reachability knows one, else the game port (crates/nf-view/src/mpnet.rs, `NetCore::poll` — the
  `public_port` pick feeding `CloudAdvert.port`; crates/nf-net/src/cloud.rs `CloudAdvert.port` doc).
- The STUN-observed mapping is not known (or is a different mapping) until reachability has probed — which
  lands around the lobby → match hand-over. The next refresh POST then names a different port → a different
  key → the service assigns a NEW code. The old entry lingers until the 90 s TTL (nothing deletes it), so
  for a window the host holds two entries and the advertised code changed.

The port in the body is load-bearing (a joiner must dial the public mapping, not the game port), so the key
cannot be pinned client-side by lying about the port. The code must travel across the key change.

## The fix: the host sends its code; the service moves the entry for the same host

Client (crates/nf-net/src/cloud.rs, `CloudHost::poll`): every POST /v1/host carries the last code the
service assigned this host (`"code":"HXKR"`; `"code":null` before the first answer). `CloudHost` already
retains the code across refreshes — only the body needed it.

Service (tools/lobby-worker/src/handler.ts, POST /v1/host) — when the key is new and the body carries a
code, in order:

1. Same host, any port: the code's current owner entry has the requester's IP → move it: delete the old
   key's entry, keep the code for the new key. One entry, one code, for the session — and the stale
   pre-change entry is retired at once instead of living out its TTL. (`CF-Connecting-IP` identifies the
   host; a code can only be claimed by the same address that registered it.)
2. Free code (no live owner): granted as asked.
3. Owned by a different host: ignored — a fresh code from `assignCode`. Different hosts never share a code.
4. Absent / malformed code (not 4 alphabet symbols after uppercasing): ignored — a fresh code. Old clients
   (which never send `code`) behave exactly as before.

A refresh of the SAME key keeps its code as before (the body's code is irrelevant there). A re-listen
(`NetCore::rebind_host`, a new socket) still gets a fresh code: the old entry is keyed to a dead port and
the client object is dropped on purpose (mpnet.rs comment at the `cloud_host = None` site).

The nf-lobbyd directory REGISTER (crates/nf-net/src/directory.rs) has the same shape of issue but keys by
source address (the socket itself re-registers) and is out of scope here.

## Tests

Worker (tools/lobby-worker/test/handler.test.mjs, plain node:test against the pure handler):

- `a port-change re-register carries the code`: same IP, new port, body `code` → same code; the lookup
  answers the NEW endpoint; exactly one entry lists.
- `a code owned by another host is never stolen`: a different IP asking with someone else's code gets a
  fresh one; the owner's entry is untouched. (Sister: two different hosts still get different codes.)
- `an unknown or malformed requested code falls back to a fresh one`: non-alphabet symbols and a wrong
  length are ignored, not errors.

Client (crates/nf-net/tests/cloud_lobby.rs, the real client path over a local mock of the service):

- `the_code_survives_the_port_change_re_register`: register → code; the STUN mapping becomes known → the
  same `CloudHost` re-registers under the new port → the service answers the SAME code; the lookup resolves
  the new endpoint; sister: a second host still gets its own, different code.

The mock service in cloud_lobby.rs mirrors the worker's rule (both alphabets are the same 30-symbol set:
`SHORT_CODE_ALPHABET` in crates/nf-net/src/directory.rs = `ALPHABET` in tools/lobby-worker/src/handler.ts).

Evidence tags: the symptom is the driver's live-service observation; the mechanism and the fix are the
port's own design decisions [S port design], cited to the code above — no game decompile is involved (this
is the port's own online infrastructure; the PS2 had no internet lobby).
