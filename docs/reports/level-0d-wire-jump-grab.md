# level-0d-wire-jump-grab — the wire #271 jump-grab fires; the real blocker was a dead Bond and a missed deck-guard kill

## The question

The ignored real-damage Operative walk on 0700000d
(`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`) failed at the wire
jump-grab assert (line 415): "the wire #271 jump-grab must fire from the ladder top (icon
#297 in context)". Does the port's grab rule differ from the game's, or is the route wrong?

## The answer: the grab rule is right — Bond was shot dead before the jump

Instrumented on the real path (per-tick state through the grab leg), the failure is not a
grab problem at all:

- Bond arrives at the ladder top with everything armed: substate 0 (walk), **icon_ctx 0x3**
  (the wire icon in context), regrab 0 (the climb dismount's 60-tick lock long expired),
  grounded, no head bump — and the plain (non-damage) twin grabs the wire with the same
  inputs, so the grab path end-to-end is proven on this tree.
- In the real-damage run Bond's hp was **6.0** entering the grab leg, and a 6.0 hit landed
  **mid-face-loop** (the walk toward the wire's A end): hp 0.0 → `Player_CheckForDeath`
  starts the death sequence and `Player_Update`'s dead branch runs **no substate arm and no
  input** [M 0x193600:316-319 — `AnimObjectUpdate` + `Player_HandleDeath` only]. Every later
  tick is skipped — Bond froze at the ladder top and the jump never happened. The killer's
  ledger: **the deck guard #16, 64.0 in 4 hits** from d 29.7 (plus #29 16.0, #27 14.0).

## Why #16 was unkillable: the port's boxes-only bullet volume vs the game's capsule

The route's ladder-top kill on #16 (600-tick cap) landed nothing while his ray printed CLEAR
every sample. Instrumented to the bullet:

- The taps fired **perfectly converged** (err 0.0000, perr −0.0000) with the scoped bullet
  ray CLEAR; 4 rounds flew.
- The rounds passed **0.21 u from #16's origin** — inside any plausible body volume — and
  `first_box_hit` over his 16 posed boxes returned **None every time**: his fire anims
  (state 0x70/0x71, scripts 0x6000606B/0x60006137) pose the whole hit-box set 0.65-1.0
  BELOW his origin (the crouch-fire skeleton: the head-box top 14.03 vs the origin 14.04,
  the torso centre 13.72), while the chest aim rides pos+0.3.
- The game's standing shootable volume is NOT the boxes: `NDrone2_Collision` 0x140e90's
  standing branch builds the two-sphere capsule **A = pos + up·r, B = pos + up·(r − hip),
  radius r = 0.4 (0.55 when BLData+0xd8 == 0xc)** [M 0x140e90:91-121 — the +0x3f0/+0x400
  centres and the +0x45c radius; the same centres the port's own `feet_on_point` doc reads
  as "obj + up·0.4 and obj + up·(0.4 − hip)"]. A pass 0.21 from the origin can never escape
  that capsule — **the game lands this hit; the port's boxes-only refine
  (`drone_ray_part`, whose own doc calls the boxes a documented approximation of those
  spheres) drops it.**

That is a real port deviation, but it sits on the shared bullet-vs-drone path behind every
fight in every walkreal level; with the sandbox's suite not fitting (below) I did not
change it in this window. The route fix below is exact for this leg and is what a player
does anyway; the capsule fix is the flagged port work with the live checks listed.

## The route fix (what a player does)

`kill_guard_head_present` (the box act — `clear_box_aim`: aim at a box the port's own law
ray-tests clear, the part-5 head first) on #16 from the ladder top: the crouched-fire body
IS what a player sees and aims at. The kill lands at **+209 ticks**, and the wire #271
jump-grab fires at jump tick 60 (quad [0.0, 0.014], icon #297 in context) — the ride, the
deck landing (+38 settle), the E on the rotate touchplate #450 and the annex legs all pass
after it.

## How it was verified

- `walkreal_0700000d_live` whole (the only file changed): **the plain twin green (1
  passed)** with the box act in the shared body — the jump-grab, ride, deck and annex all
  pass; the two real-damage sisters stay `#[ignore]` with updated measured ledgers.
- The ignored Operative run was re-driven end to end for the numbers: the grab fires, the
  death moved from "the grab assert at ~3.7k" to **tick 12540** — alive at 4.8 health (26
  hits for 88.8, the ride exposure), stopped by the yard-slot jump-drift hop (the board
  offset 1.01 and the apex 138.19 are byte-identical to the plain twin; the box act's
  changed RNG stream diverges the guard positions the hops thread). At 4.8 health the
  ~10 m slot fall's pain gate would kill anyway — the ride-phase sniper stands (the old
  lever) remain the next route work.
- The full suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio
  -p nf-view -- --test-threads=1` / the suite.py runner) was started twice on this sandbox
  and died in its build phase both times (3 GB RAM; empty output, no OOM message — the
  pattern the guard-pane-drain-3 session also hit). **The full suite did not fit.** The
  change is test-only in one level's file (src/ reverted after the instrumentation — the
  pushed tree touches no src), and every binary that shares the changed file's helpers
  lives in that same file; its binary ran whole above.

## What's left

1. **The port-side capsule** (the real fix for fire-anim guards): add the game's standing
   capsule to `drone_ray_part`'s standing branch (the boxes refine the part, the capsule
   decides the hit) — [M 0x140e90:91-121] — then re-run every `walkreal_*_live` (the fights'
   pinned timelines will shift wherever a fire-anim guard was un-hittable).
2. The Operative walk's remaining route: the ride-phase sniper stands (the 88.8 drain),
   then the slot hop under the diverged guard state; the Agent sister's d2 levers are in
   its own ignore message.
3. The per-tick Bond-vs-wire log the task asked for is in the session's instrumentation
   (grabdbg/trace runs quoted in the WORKLOG); the port's grab constants themselves were
   not touched and cite 0x1916b0:118-147 / 0x197658 as before.

## Live checks for the main agent

1. **Fire-anim guard hittability.** In PCSX2 on 0700000d, stand at the ladder top
   (213.0, 17.4, −234.5), aim down at the deck guard #16 (198.9, 14.0, −260.3) while he
   fires (state 0x70/0x71) and shoot his chest line. Expected: rounds LAND on hardware
   (his origin-centred capsule [M 0x140e90:91-121] covers the line) — the port's boxes-only
   volume misses the same line while his fire anims play. A gdbwatch on the round's
   `NDrone2_DealWithObjHit` entry vs the port's `first_box_hit` miss pins the shape question.
2. **The wire jump-grab on hardware.** Walk the real route to the ladder top, face the
   wire's A end, jump with the stick held. Expected: the grab replaces the jump (icon #297
   in context, substate 6) — matching the port's now-green plain twin.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session; nothing to copy.

## ntfy failed

All three fallbacks failed or stayed unconfirmed from this sandbox: `curl -sS -m 20` →
`curl: (28) Connection timed out after 20002 milliseconds`; the python3 urllib fallback →
`urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`; the wget
fallback → the first `wget -q -O-` run returned rc 0 with no output, but a follow-up
without `-q` shows the TLS connect to ntfy.sh (159.203.148.75:443) stalling past 20 s —
the delivery could not be confirmed. The finish.sh step itself reported "OK done
10-level-0d-wire-jump-grab".
