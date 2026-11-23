# retune-nb-16-9 — 16 chasers-defeat 00 Agent real-damage walk, round 9

Branch `s/retune-nb-16-9`, from origin/master 7562f1e70, with `origin/s/retune-nb-16-8` merged
in first (clean — the 16-8 report, the 16-8 WORKLOG entries and the 16-8 walk-test levers came
with it). The walk is still red, but **the walk reached the FINALE SQUAD PHASE — the deepest
measured point ever** (the prior best, round 8, died AT the shelter entry t9383; this round's
best run died t15655 INSIDE the squad phase, past both chaser fights, with the armour fetched).
The refreshed ledger and the round-10 levers are in the ignore message.

## Done — the two ledger levers landed, plus the unseen-pourer answer

Each lever judged on its own real run (NF_DMG_TRACE; the traces kept in the session, not
committed).

1. **The shelter-barrel lever (the ledger's next lever (a)) — LANDED.** The kind-27 barrel at
   (−12.4, −21.4, −169.4) (radius 4.0, damage 50.0 — the round-8 killing blow at the shelter
   entry) is detonated BEFORE the walk-in: the walk stops at the INTO_THE_SHELTER route's own
   first-leg stand (−8.5, −22.2, −173.5), which is 5.65 u OUTSIDE the barrel's 4.0 radius, and
   `shoot_point`s the barrel (the route's own first leg — no route change). The blast fires
   harmlessly at that range and the walk-in passes a dead barrel. Measured properties:
   - the converged detonation takes ~250 ticks [L the runs];
   - the arm is SELF-SAFE: a failed aim (the give-up at the 700-tick bound) still walked in
     alive — any landed near-miss round lights the fuse while Bond is outside the radius, and
     an unlit barrel cannot blow [L the run];
   - the give-up bound was tightened 700 → 450 after the 700 bound blew the door-shut assert
     once (inside_at 1029 > the 1000-tick door law [D the leaves move 1000..1350]); the door
     assert then held at +778 [L the runs].
2. **The ring fights' wound-abort composition (the ledger's lever (b)) — LANDED.**
   `abort_on_wound = Some(6.0)` scoped to the d3 route loop (the sisters' `walk_route` branch
   never runs it), plus the ESCALATION made abort-aware in the shared damage_log: the
   determined pass runs a bounded 300-tick kill window instead of the 1800-tick undo when the
   flag is set (the flag's only other user, the 01 hall test, sets it only inside
   `finish_fights = false` windows — the arm is dead for every existing test, verified by the
   flag grep and the 01 binary re-run). MEASURED: the ring fights killed #4 and #5 at ZERO
   health cost (the determined-pass prints `health 30.0 -> 30.0`) and the pool at the hand-back
   moved 9.0 → 30.0 [L the runs].
3. **The pulse-1 sit's wall diagnosed and answered.** The sit's pick found NO exposed drone at
   all (the `[sit]` diagnostic never printed) — the chaser-class heavy (skin 0x500001e, hp 100)
   pours from a spot the strict-ray `exposed_part` never opens, at 2.6 u point-blank inside the
   door. The hits-evidence picks also came back empty (the grazes never register in
   player_hits — "no recent hitter" twice with the heavy's burst inside any window [L]). The
   answer: **`duel_the_muzzle_16`** (test-local, the 06 file's muzzle-duel idiom): the
   NEAREST-ALIVE hostile within 45 u, no enabled gate (the heavy flips dormant after his burst
   — the game's bullets hit ANY present drone [M Drone_CollisionHandler 0x140968:85-95]), the
   measured chest, the symmetric hit model decides. The duel ground the nearest drone dry and
   the walk stormed through the sits, BOTH chasers (down +2986/+3407), the pit-room armour #648
   and the east armour #649 (health 24.7, armour 50.0 at +4391), pulse 2 over at +7270 — into
   the squad phase.
4. **A gate bug caught by the whole binary, fixed.** The first sister sweep turned the d1/d2
   walks red (the barrel arm and the sit variant sat under `re_arm && !tops_up`, which the
   real-damage sisters also pass). Re-gated to `difficulty >= 3 && !tops_up()` (the file's
   established d3 pattern), the sit-pick variant reverted (it measured byte-identical — the
   pick order was not the flaw; kept as a `#[allow(dead_code)]` ledger note), and the whole
   binary re-verified 18/0/1 twice.

## The remaining wall (the round-10 levers, in the ignore message)

The squad phase's chaser-class clone #12 (skin 0x500001e, hp 37) poured 17.5 in 15 hits at
2.0 u point-blank INSIDE the room; the pool 24.7 health + 50 armour died t15655 at
(−12.85, −22.01, −170.50) — the same unseen-pourer class at the same door-adjacent geometry.
Next: (a) the squad phase's own nearest-alive muzzle-duel pass from the room interior; (b) the
#596 station armour's spend window (the armour 43.0 at the entry was spent to 0.0 by the sit's
pours — the pours' true output ~73/window including the soak); (c) a live check on the heavy's
exposure (why `exposed_part` never opens at 2.0-2.6 u).

## How it was verified

- Every lever ran on its own real run through the real input path; the determinism of the
  landed shape held (the byte-identical repeats across the probe runs).
- **walkreal_07000016_live at the final tree: 18 passed / 0 failed / 1 ignored** (141.6 s,
  `--test-threads=1`) — the d1 real-damage walk, the d2 agent walk, the tops-up sisters, the
  punch pin, the door/interactable tests all green; the 1 ignored is the d3 walk (the ledgered
  one). Re-verified twice (after the gate fix and after the duel landed).
- **walkreal_07000001_live: 7 passed / 0 failed / 1 ignored** (488 s) — the abort_on_wound
  flag's only other user (the escalation arm is dead for it: the flag is set only inside
  `finish_fights = false` windows there).
- **walkreal_07000015_live: 9 passed / 0 failed.**
- The diff touches `crates/nf-view/tests/walkreal_07000016_live.rs`,
  `crates/nf-view/tests/common/damage_log.rs` (the abort-aware escalation — opt-in via the
  existing `abort_on_wound` flag, dead for every other test) and the docs. No `crates/*/src/`.
- The fast suite runner was started in the background three times (pids 1653, 5458, 6359); the
  sandbox kills background jobs between tool calls (the wall the −5/−6/−7/−8 reports
  documented) — the last start (6359) was alive at the checkpoint; its partial output is not a
  verification set. The full 80-file walkreal sweep did not fit the window; the affected set
  above (the 16 binary twice, the 01 binary, the 15 binary) is what ran, with the flag-grep
  proof that the damage_log arm is dead for every other test.

## Live checks for the main agent

- The chaser-class heavy's exposure (skin 0x500001e, the shelter door's inside at
  (−13.5, −22.0, −169.4)): gdbwatch on the eye-ray/box trace during the pulse-1 sit names why
  `exposed_part` never opens at 2.0-2.6 u (the door-leaf geometry vs the eye-ray rule) — it
  decides whether the sit can ever fight him or the muzzle-duel pass is the only answer.
- The shelter barrel's ignition owner: gdbwatch/fieldprobe on the barrel's state during the
  hand-back-to-entry window names what lights the fuse when the detonation arm is absent (the
  soldiers' return lines cross it westward [S the round-8 reading; still unproven]).

## What's left

- The squad phase's point-blank clone members (above) — the round-10 levers (a)/(b)/(c).
- After the squad phase: the pit-squad stage's own laws (the 16-5 green shape: the east-armour
  fetch — measured landed this round — the member pick, the slice hold), the lift-out hand-back,
  the button, then un-ignore.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed against https://ntfy.sh/nightfire-yoda-8k3x from this sandbox at
finish time (2026-11-23 ~10:35 UTC) — the same ntfy.sh egress failure the −1/−2/−8 reports
documented (the GitHub pushes and the game-file downloads all worked all session):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` —
   `curl: (28) Connection timed out after 20002 milliseconds`.
2. the python urllib fallback — hung past the 20 s timeout, the tool call killed at 40 s.
3. `wget -q -O- --post-data "..." --timeout=15 https://ntfy.sh/nightfire-yoda-8k3x` — hung,
   the tool call killed at 30 s, no output.
