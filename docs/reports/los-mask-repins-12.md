# los-mask-repins-12 — report

Branch `s/los-mask-repins-12` = origin/master `0e1371451` + one work commit (44d864640). Task: the
07000012 walks green again under the game's guard sight; FIRST the door plc 543 behaviour check.

## What's done

**The door plc 543 behaviour check (the task's FIRST item) — resolved: the port is not wrong.**

- The old ignore's "holds state 0x2 at 0.50 for ~200 ticks after the E press" does **not**
  reproduce on master tip 0e1371451: with `NF_DOORTRACE` the Operative run's door #15 opens fully
  (state 0x8, pos01 0.5 → 1.0 over 30 ticks, no `Door_ReactToHit` reversal) and the run still
  wedges — the failure is the ROUTE, not the door state machine.
- The port replicates the task's live fact end to end. [L PCSX2 2026-11-10, fresh load, guards
  asleep] Bond at (9.9, 1.1, −13.0) facing south, Use pressed, stick forward 240 frames → the
  game ends at (6.21, 1.13, −25.40); the port's identical walk ends at (6.25, 1.13, −25.39) —
  within 0.05 u. The door lets him through on that approach, as in the game.
- The door laws this rests on: for kind-0 (swing) doors pos01 0.5 is the SHUT pose (frame 0/1.0
  are the two open extremes) and a CLOSING leaf snaps to 0.5 + `Door_SetState(2)` [M Door_Interp
  0x135738]; `swing_from_side`'s side is `sign(local.z)` with the shut matrix's column 2 = **+x** —
  i.e. the swing side is EAST/WEST of the hinge at x 9.95, not north/south [M Door_Activate
  0x135be0:71-74 + the shut matrix read in the probe]. The anti-crush itself matches the
  decompile [M Door_ReactToHit 0x135980].
- The real blocker: the route's (11,−12) → (8.5,−13.5) diagonal runs against the corridor's west
  wall north of the doorway hole (hole z −13.86..−15.15 [D the static collision tris]); Bond gets
  stuck there and the walker's stuck-E lands **east** of the hinge (x ≈ 10.3–11), which swings the
  leaf into the corridor's west mouth; the straight crossing then wedges 6000 ticks [L the d1
  run, the STUCK dump].

**The re-route (gated, real input only).** The Operative route now takes a west-stand crossing:
walk to (9.3, −12.85) (Bond's natural rest west of the hinge — same swing side as the live fact),
press E there (`use_door`, with an `assert state == OPEN`), then cross straight south on the live
fact's line to (9.0, −17.5). The legs are gated on `DOOR_543_WEST_STAND`, set only at difficulty
1 (`walkreal_07000012_live.rs`) — the d3/d2 diagonal is the proven route there, and an ungated
switch turned all three of them red, so the gate is load-bearing. No damage/health writes, no
top-ups, no teleports after the load; the E press is the walker's real use path.

## How it was verified

| binary | result |
|---|---|
| walkreal_07000012_live (whole, before the gate) | 3 passed / 1 ignored — the d3, d3-real-damage, d2 walks green on master tip |
| walkreal_07000012_live (whole, after the gate) | 3 passed / 1 ignored — the gate restored the three; the ignored one is the Operative |
| interactables_07000012_live | 12 passed / 0 failed |
| the live-fact probe (scratch, not committed) | the port's answer (6.25, 1.13, −25.39) vs the game's (6.21, 1.13, −25.40) [L] |

## What's left

- **The Operative walk is still ignored** — but for a NEW, later reason. It now crosses door 543
  and dies at t7167: drone #2 (skin 0x5000008, weapon 0x19) at (−9.1, −4.1, −21.0), 20.8 u away,
  57 hits for 100.0, in the south hall before door plc 545. The ignore message carries this
  ledger. Next window: re-tune that fight on real input (the level's own head-hunting +
  unseen-cover levers are already armed at d1), then remove the ignore. Nothing was lowered: no
  damage, no health, no top-ups.
- **The full suite did not run.** The tree settled past the 23:03 UTC stop line, and background
  jobs die between tool calls in this sandbox, so `tools/test/suite.py` could not stay up (the
  same constraint the los-mask-repins-08-agent worker recorded). Next window should run it on
  this branch before more code moves.
- The sister binaries beyond the level's own two were not run: the only shared file touched is
  `crates/nf-view/tests/common/tower2.rs`, and the change is a new opt-in flag whose only setter
  is `walk_spawn_to_the_shaft_exit_at(difficulty==1)` in `walkreal_07000012_live.rs` — the 07000013
  warm chain calls `tower2::walk_12` with the flag unset and keeps the old diagonal (verified:
  the d3/d2/d3-real-damage walks are the ones that exercise it here, all green).

## Live checks for the main agent

1. Door plc 543, east-side E: with Bond walking the (11,−12) → (8.5,−13.5) diagonal in the real
   game (fresh load, guards asleep), press E when he stalls against the corridor's west wall
   (x ≈ 10.3–11, z ≈ −12.7..−13.5) and record (a) which way the leaf swings (port prediction: to
   pos01 1.0), (b) whether Bond can reach (8.5, −13.5) on that approach. If the game's leaf ALSO
   swings to 1.0 and also wedges him, the d1 wedge is game behaviour and the route fix is the
   player's real answer; if the game passes, the port's swing-side pick from east of the hinge is
   a port bug worth a decompile pass on `Door_Activate` 0x135be0's disasm side tests.

## Usage

not visible to me — this environment exposes no token, credit, cost or context-window figures for
the session, and I will not estimate them.

## ntfy failed

All three fallbacks failed; the sandbox has no outbound route to ntfy.sh this session:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "los-mask-repins-12: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "… urlopen(Request('https://ntfy.sh/nightfire-yoda-8k3x', data=…, headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -S -O /tmp/ntfy_resp.txt --post-data "los-mask-repins-12: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → exit 124 (timed out): `Resolving ntfy.sh (ntfy.sh)... 159.203.148.75, 2604:a880:800:14:0:1:73c0:2000`
   then `Connecting to ntfy.sh (ntfy.sh)|159.203.148.75|:443...` and no response within 25 s.
