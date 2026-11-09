# level-11-strip-fight — the 23.0-strip guard pair on 00 Agent

## The ask

On 00 Agent, guard #33 parks point-blank on the 23.0 strip of 07000011 (Phoenix Fire 1, the
tower stairwell). Check the pair #33/#34's engage and stand rules there against the game (does
the game close a routed guard to contact distance and fire at that range?), fix the port if they
differ, and otherwise accept the walk's mouth stand as the player's answer.

## What the pair is

The walk's #33/#34 are sim indexes → **placements #233/#234** [D audit rows: the ch-0x13
waiters at (22.93, 24.27, −3.49) / (22.96, 24.30, −1.65), the 23.0 strip's east end — where the
walked strip leg ends at (22.4, 23.2, −3.0)]. They wake disabled in WaitSwitch (hidden, the
not-drawn bit [M NDrone2_Enable 0x149dd0:8-15]) and engage when Bond's descent crosses their
plate channel 0x13 [M DSTATE_WaitSwitch 0x15b960:22-35]. (Index 33/34 of the EXTENSION's
07000012 load are different drones — placements 278/280 there — which is why the recorder
phase-tags by scene.)

## The game's chain (all [M], re-read this session from work/decomp/ACTION.ELF)

- `DroneFunc_CombatState` 0x148338:47-49 moves the routed combat guard toward the player at
  **min(engage, 6.0)** — 6.0 on this level (the anim-set override [M NDrone2_DefaultInit
  0x14b300:429-457: 07000011 sets engage 6 / combat 9, 5/8 for set 9]). The 6.0 is the route's
  GOAL radius: `SetupRouteToObject` 0x158948 raw 0x158a90 stores it to AITarget+0x14 (the
  zero-check + 2.0 default at raw 0x158a44-0x158a60), while the NodesForPosition node search
  itself runs radius 0 ($f12 = 0.0 at the raw call site). It is NOT a stand-off: the
  0x58/0x5a flips ride straight-line dist [M 0x148338:63-64, 88-90] and both states keep
  following the route (case 2 arms 0/1/2 play the run-and-aim move anims).
- The route-follow STOP is **remaining < 2.0** [M NDrone2_MoveToObject 0x151618 raw
  0x151750-0x151768: `if (GetRouteDistance(route) < route+0x8C) return 3`, disassembly-checked
  (lwc1 0x8c($s1) / c.olt.s / bc1f); route+0x8C ≡ drone+0x8EC = 2.0f, written only by
  DefaultInit 0x14b300:579]. `GetRouteDistance` 0x156320 sums guard→next-node + remaining links
  + target→last-node, so the stop lands within ~2.0 of the player target (≤ 2.0 stale [M
  BuildAITarget 0x15ac78: the target rebuilds when Bond moved ≥ 2.0]).
- The mv-3 arrival arm stands the guard and fires there [M 0x148338 case-2 arms 3/0xb:
  SetAngleToObj, then the 0x21 aim anims]; a no-route guard holds and fires from its post [M
  case-2 default → 0x5f — the hold the engage-range test pins for the start corridor].
- A standing firer resumes the chase when Bond draws past the engage line [M case 0x10:
  routed && engage ≤ dist → 0x5a]; the sub-engage NewSighting flip picks AimStandFire below
  6.0 [M 0x173280:ENTER; the class-2 byte gates only the grenade/move choice at 0x148338:215,
  not the stand-fire].

So the game's own rule chain closes a routed stairwell guard to contact distance and fires
there. The engage 6.0 never holds a chaser at 6; it only re-arms the chase from the fire state.

## The port's measured behavior (real input, the walked 00-Agent route)

New test `the_23_0_strip_pair_engages_per_the_game_chain` (walkreal_07000011_live) arms an
opt-in, read-only per-tick recorder (common/walker_b2.rs `pair_trace_on` — flagged, phase-tagged
by scene identity, no RNG draws, so the walk replays bit-exact) and re-runs the existing
`..._real_damage_00agent` walk body unchanged. Measured run (2026-11-09):

- both pair members wake into the engaged family at t5438 (state 0x58) during the descent;
- the chase is ROUTED: 965/966 (idx 33) and 832/966 (idx 34) post-wake ticks carry a combat
  route;
- closest engaged: **0.81 / 0.79** from Bond (t5744), in state 0x68 AimStandFire playing the
  0xa AIM_STAND anim (danim class 2 on #233's skin, 7 on #234's — per-skin, see above), route
  remaining 4.17/4.15 at that tick;
- their rounds land point-blank: 3 and 4 hits on Bond with the shooter at 1.07–2.09 / 1.11–5.64.

That is the game's chain exactly: the port's routed pair closes to contact and fires there. The
hidden pre-wake waiters sit 0.63/0.23 off Bond's walking line [D audit positions] — Bond passes
right through the pair before the plate wakes them, which is the audit's "hidden until woken"
rule, not a fight bug.

**Verdict: no port change.** The pair's point-blank parking is the game's own engage/stand
chain; the walk's mouth stand is the player's answer, as the task anticipated.

## Pins added

`the_23_0_strip_pair_engages_per_the_game_chain` asserts, per pair member (identity pinned to
placements 233/234 [D audit]): the wake into the engaged family on the real walk; a routed
chase; engaged-phase closest ≤ 4.5 (the remaining < 2.0 + ≤ 2.0-stale bound [M 0x151618 raw +
0x15ac78]); a 0x68/0x70 fire state inside that range [M 0x173280:ENTER]; and a hit landed while
inside it [M 0x148338 case 0x10 stand-fire]. The measured values print in the run.

The recorder under `crates/nf-view/tests/common/walker_b2.rs` is opt-in per the shared-helper
rule: `pair_trace_on()` sets thread-local sinks, `after_step` checks them, and an unset sink
costs one thread-local read per tick; nothing else reads or writes the sim there.

## Verification

- `walkreal_07000011_live` whole binary, this tree: **6 passed / 0 failed** (the plain d1 replay
  walk, the real-damage walk, the Agent and 00-Agent walks, and the new pin test) — the recorder
  changes nothing for the existing walks (bit-exact replay).
- The decompile side was re-read this session (the [M] list above), with the two load-bearing
  comparisons disassembly-checked (`MoveToObject`'s 0x8C compare; `SetupRouteToObject`'s
  AITarget+0x14 store and the radius-0 call).

## What's left

- The full suite did not fit this session: the sandbox disk (9.9 GB) filled during the
  four-crate build (the game data + target tree exhausted it twice; I cleared ~14 GB of stale
  material and the build then completed, but the run window closed first). **Every other
  `walkreal_*_live` binary and the full four-crate suite still need one run on this branch** —
  the only shared change is the opt-in read-only recorder above, and its flag is false for all
  of them.
- The nf-game/nf-formats/nf-audio trees are untouched by this branch.

## Live checks for the main agent

One capture would firm the [M] chain end-to-end on real hardware:

1. Load 07000011 on 00 Agent, walk the stairwell descent to the 23.0 strip (the route of
   walkreal_07000011: spawn → stairwell door → flights to 27.1 → west flight B → north strip
   23.0 east), and watch drones #233/#234 (their +0x1A0 dist-to-opponent and +0x10C state):
   expected per the chain — they wake on the ch-0x13 plate, chase with a route, and stand
   firing within ~1–2.5 of Bond (state 0x68, +0x56A = 0xA) rather than holding a 6–9 stand-off.
2. A gdbwatch on drone #233's `Drone+0x8EC` (the route arrival field): expected 2.0 (float)
   the whole level [M 0x14b300:579] — confirming nothing rewrites the stop distance.

## Usage

not visible to me — this environment shows no token, context-window or cost figures for the
session, and I will not estimate.
