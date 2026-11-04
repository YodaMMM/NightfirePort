# glass-pass-every-level — the placed-glass sweep, round one

Branch `s/glass-pass-every-level`, cut from `origin/master` (f9f6544c — the hind flight glass
pass and the 08 pane shoot-out are both already on master; no merge needed). **Status: BLOCKED by
the sandbox window (~12:54 UTC), not by code — the sweep's frame, the records/unit pins and the
first real-input crossing pin are landed green; the per-level guard-through-glass pins (06/08/14
and the charger-impact half) are the next session's work, with every lead measured and on the
branch.**

## What was done

### 1. The census [D]

`crates/nf-view/examples/breakable_census.rs` (committed) loads every story level and dumps the
PLACED breakables. Placed kind-2 (glass) panes: **01(12) 03(23) 06(1) 07(4) 08(3) 09(17) 0a(44)
12(97) 13(33) 14(3)**; none in 02/04/05/0b/0c/0d/11/15/16/1b. The sweep's universe is those ten
levels. `crates/nf-view/examples/glass_recon.rs` (committed) prints each pane's bounds + every
live guard within 25 u — the "guard behind glass" candidates per level (06 pane 376: guards 1-7 u
away; 08's three panes: the mid-station ambush 8-23 u; 14's three windows: guards 1-5 u behind).

### 2. The fists/laser exception pinned at the records and the unit [L]+[M]

- **[L] the record census** (ee_start_05.bin @0x2bf150 + id·0x10C + 0x6c, dumped 2026-11-04):
  bit 0x800 is SET for exactly 28 ids — 0x01 (fists), the beams 0x4e/0x4f (+0x6c = 0x900), and
  the mount/gadget family (0x11, 0x1b, 0x2a/0x2b, 0x34-0x37, 0x3a-0x41, 0x43, 0x44, 0x45, 0x4a-
  0x4d, 0x59, 0x5b) — CLEAR for every player gun (0x02.. = 0x00040050/0x00040040) and the
  launchers 0x2c/0x2d. `the_0x800_records_match_the_disc` asserts the whole table against the
  live `sim.weapons.bullet_flags`.
- **CORRECTION to the hind report**: 0x45 is the grenade-belt pseudo-weapon [M gun.rs:147-157],
  NOT a laser; the laser is 0x4e [L]. Both carry the bit.
- **[M] the unit law** (`the_glass_pass_unit_law`): `penetration_roll` passes material 0x10 iff
  the record's bit 0x800 is clear (pistol 0x02 and launcher 0x2d pass; fists 0x01 and the laser
  0x4e stop; a plain wall 0x0a never passes) and draws no RNG [M Collide_Update 0x1ea398:216-229].

### 3. The 01 real-input crossing pin — GREEN

`glass_pass_every_level_live::the_01_breach_shot_breaks_the_entry_glass` (3 passed / 0 failed /
1 ignored, the whole binary): Bond holds the Mayhew courtyard spawn through the intro on raw
`step_bond` (no teleport, no walking — the spawn faces the glass), the pistol is in hand from the
load [L the probe: wpn 0x2 clip 7 spare 41 at t0], the aim converges on the nearest charger
behind a LIVE entry pane (130/136/155 [D glass_breach_live]) during the cut-scene, and the taps
fire on converged post-wake ticks. **Measured: two rounds each DRAINED a live pane — pane 130
died t2131 off the t2130 tap and pane 136 died t2142 off the t2140 tap, both inside the pistol's
5.0 u/tick flight window [M bullet.rs:463-464, the record +0x20]** — the round crossed the
surface-0x10 contact and the mirrored contact killed the glass [M Break_Update 0x12bfd8:36-37],
the game's own law, on real input from the level start.

### 4. The measured leftover — the freshly-woken waiter is not yet bullet-pickable

The pin's second half (the round REACHES the charger) did not close: at wake+7..11, three
converged rounds (target lead included — the guard's tick velocity × the flight time) drained
the pane and **no `PlayerImpact::Drone` registered** — the waiters are not yet bullet-pickable in
that window [M Collide_PickObj 0x1eae80:26-30 drops obj+0xF0 bit 0x20; NDrone2_Enable(0) sets
it], while the chargers' bodies reach pane 130's plane at wake+16 [L the probe — the smash].
Whether the GAME also holds its first post-wake ticks unpickable (then the test's window moves)
or the port holds them longer (a real deviation) needs the next session: watch the woken guard's
obj+0xF0 across wake+0..30 in the port vs a PCSX2 trace. The test asserts the PROVEN half green
and carries the story in its messages.

## How it was verified

- `glass_pass_every_level_live` whole: **3 passed, 0 failed, 1 ignored** (the 14 window
  stand-walk test is `#[ignore]`d — see What is left).
- `walkreal_07000001_live`: **8 passed, 0 failed, 0 ignored** (88.7 s).
- `walkreal_07000014_live`: **9 passed, 0 failed, 1 ignored** (189.2 s).
- The branch changes NO `crates/*/src/` code (one new test binary + two load-and-print
  examples + the WORKLOG), so the shared helpers and every other binary are untouched by
  construction; the full suite total line is **not measured this session** (the window closed
  during the walkreal runs) — the next session runs it on this tree before merging.

## What is left (in order)

1. **The charger-impact half of the 01 pin** — the pickable gate above; the test's taps are
   already gated to wake+10..22, so the fix is either a later window or a sim-side enable-order
   fix with the game address.
2. **The 06 pin** — pane 376 (c 30.1, 41.5, −16.3, a z-sheet), guards 1-7 u behind it [D the
   recon]; a `walk_nav_to` stand + the same `shoot_guard_through_panes` loop (the walker-side fn
   is in the binary, ready; its span tolerance is 0.25 — the frame tris flank the glass).
3. **The 08 pin** — the three mid-station panes 170/176/177 (y 35.3) with the ambush behind
   [D the recon]; the walkreal-08 route's hall-y33 legs already walk the area.
4. **The 14 pin** — the windows 97/86/156 (x-sheets at z≈33.3) with guards 1-5 u behind; the
   `#[ignore]`d test has the stands; it needs the stand-walk debugged (the pane-side guard pick
   worked in the recon; the walk window ran out).
5. **03/07/09/0a/12/13** — the remaining placed-glass levels (the census's 260 panes); the same
   loop per level, one stand each.
6. The full suite total line + every `walkreal_*_live` on the final tree.

## Live checks for the main agent

1. **The woken waiter's pickable window** (the leftover's [M] check): in PCSX2 on 07000001, watch
   a breach-squad guard's obj+0xF0 (bit 0x20) from ch 0xB's write to the guard's first charge
   step — expected: the bit clears when the guard enables; record the tick gap. If the game's
   gap is ≤ a few ticks, the port's pick gate holds too long and the fix belongs in the port's
   pick/drop (the address above).
2. **The entry-glass drain**: stand Bond at the courtyard spawn, fire the pistol at a charger
   behind a LIVE pane the instant the wake camera releases — expected: the window shatters from
   the round (the mirrored contact) and the round continues [M 0x1ea398:216-229].

## Usage

not visible to me — no token, credit, cost or context-window figures are exposed anywhere in
this environment (no status bar, no session summary, no API dashboard access).
