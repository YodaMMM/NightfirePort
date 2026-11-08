# 0700000d: a ~10 m drop kills Bond outright

The ignored walk `walk_arrival_to_the_c5_door_real_damage_agent` (crates/nf-view/tests/walkreal_0700000d_live.rs, ~line 836) takes a ~10 m fall that the game survives [M 0x1949a8], and the port takes Bond from 26.8 to 0. Read the game's fall-damage rule (fall height or landing speed, scale, cap, difficulty factor) at 0x1949a8 and its callers, compare with the port's landing damage, and fix it. Pin the damage for that drop and for one short and one lethal drop on another level, through real input.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binaries for every level whose walks fall (grep tests for falls), one level per tool call. Branch from origin/master, push to `s/level-0d-fall-damage`, report in docs/reports/level-0d-fall-damage.md. No draw changes.

---
claimed-by: sandbox1 at 2026-11-08T13:53Z

---
## Report (done, 2026-11-08T15:06Z)


---
## Report from the worker (done, 2026-11-08T15:07Z)

# level-0d-fall-damage — the report

Branch `s/level-0d-fall-damage` (from origin/master 6186d8530).

## What the task asked, and the answer

The ignored walk `walk_arrival_to_the_c5_door_real_damage_agent` took its ~10 m beam-slot fall
from "26.8 to 0", and the driver read that as the port over-damaging the fall against the
game's rule at `0x1949a8`. **The port's fall-damage rule is already the game's own — verified
against the decompile AND the disassembly — and the walk's death had two other causes, both
measured.** No `src/` code changed in this task.

## The game's rule, read and verified

`Player_CollisionHandler` 0x1949a8:97-127, verified in the disassembly (capstone, little-endian
MIPS, work/elf/ACTION.ELF):

- `BL+0x940` is a 4-tick grounded shift register (halved every tick; bit 8 set while the obj
  flag word's bit 8 — set by `Player_FeetOnPoint` — is up) [M 0x1949a8:78-82].
- While `BL+0x940 == 0` (airborne, 4 ticks clear): `BL+0x54` (vel.y) < 0 → `BL+0x928` +=
  FRAME_RATE_MUL; vel.y ≥ 0 → `BL+0x928` = 0 [M disasm 0x194be4-0x194c10: the FRAME_RATE_MUL
  load @0x194c04 from 0x0030_fd08, the zero store @0x194c00 in the delay slot].
- On/after landing (`BL+0x940 != 0`): timer > 60.0 (the literal 0x42700000 @0x194b74) → raw =
  (short)(int)(timer − 60); raw < 10 (the slti 0xa @0x194bb0) → the timer just zeroes; else
  `Player_HandlePain(raw, 6, −1)` @0x194bd0-d8 [M].
- FRAME_RATE_MUL = 60.0 / FRAME_RATE with FRAME_RATE = 60 on NTSC (PS2FrameRate = 0x3c
  [M PS2RegionChange 0x1094d8:9]; GS_SetRefreshRate 0x17d2b8) → 1.0 per tick [M]. The port's
  fixed 60 Hz `fall_timer += 1` matches.
- `Player_HandlePain` 0x1902c8: the difficulty factor (0/2 → Plr_DMod_Normal, 1 → Easy, 3 →
  Hard, 4 → ×2); for 0700000d the tuning group is Easy 0.4 / Normal 0.7 / Hard 1.2 [M
  ReadTuningVars 0x1d40f4]; pain types 1..7 set the armour fraction 0.0 — armour never absorbs
  a fall [M 0x1902c8:84-104]; the remainder < 1 kills (health = 0) [M 0x1902c8:131-134].

The port implements exactly this (`player.rs` fall_timer/grounded_history block, `scene.rs`
pain_generic(type 6), `damage_factor`). Gravity WldGravity.y = −9.8 set in `Player_Init`
0x1910f8:152-155 [M], terminal ±45 in `Player_Collision` 0x194df0 [M] — the port matches both.

## Why the walk died 26.8 → 0 (measured, not the fall rule)

1. **The old drop jumped after the beam's re-sweep had started.** The gantry SP parks at its
   west bottom (~137.59 [D the key trace], measured ox 137.585 in the port) then re-sweeps
   east. A jump after the re-sweep starts gets caught: the sweeping west girder crosses the
   drift lane mid-fall, the spine's horizontal-face rule parks Bond ON the girder (measured:
   landing at root 14.13 inside the swept girder at x 137.93, fall timer only 27 — no damage),
   the carry fails on the grounded flicker, and the girder's east edge drops him for a SECOND
   8.4 m fall (timer 78 → raw 18 → 12.6 at Agent) — plus awake-guard trickle during the raw
   steps. The port's slot rays at the park read open to the yard exactly like the disc:
   down-rays at x 137.5/137.94/138.2 → 3.98 [D the at-rest fine-map rays read 4.0].
2. **The raw-step phases ran the guards with no damage bookkeeping.** The E-to-beam wait
   (2350 raw ticks) drained 26.8 → 13.2 (17 hits for 75.6 cumulative + other 11.2, the
   ledger's recorded hits — the bay guards keep firing); the ride 13.2 → 10.0. The real-damage
   summary never saw it (its last store was tick 16920).
3. With the drop re-timed to the park window, the clean slot fall deals **timer 86 → raw 26 →
   ×0.7 = 18.2 at Agent** (×0.4 = 10.4 at difficulty 1) — exactly the game's math. The walk
   arrives at ~10 health, so the LAWFUL fall kills; the game kills a 10-health Bond on the
   same fall too. The pre-fix assert `health > 40` was calibrated for the game's own 18.2 from
   a ~59-health arrival — the walk's route has since grown thinner arrivals.

## What changed (tests only)

`crates/nf-view/tests/walkreal_0700000d_live.rs`:
- The beam drop now polls ox until it stops decreasing (the park), asserts the park
  (`last_ox < 138.0`), then jumps inside the park window — what a player does. The jumper
  falls cleanly through the open slot.
- THE FALL PIN, real input: the landing timer == 86 (raw 26) and the health delta == raw 26 ×
  the difficulty factor minus the window's recorded guard fire (the falls do not record into
  `player_hits` — they run through the scene's pain_generic outside the drone.rs Bond-damage
  sites — so the window's hit delta is guard fire alone). Replaces the unreachable
  `health > 40` assert; the walk stays ignored (the crossfire levers in its note remain the
  blocker).
- THE PLATFORM-DROP PIN, real input: the ~11.1 m ledge slide-off deals NOTHING (the slide
  keeps BL+0x940 set, so branch B zeroes BL+0x928 every contact tick [M 0x1949a8:97-127]; the
  game's own drop read no damage [L probe4]) — health unchanged across the leg.
- THE ANNEX-DROP PIN, real input: the roof→annex push-off's landing timer == 76 (raw 16 →
  11.2 at Agent / 6.4 at difficulty 1) — the walk's only damage-dealing fall before the beam.
- The ignore note documents all three findings.

`crates/nf-view/tests/fix_death_restart_live.rs` (07000011, another level):
- THE SHORT-DROP PIN: the 8-unit placed drop deals exactly NOTHING (health == 100.0) — the
  rule's raw < 10 floor (timer under the gate → no pain) [M 0x1949a8 slti @0x194bb0].
- The lethal pin already asserted health == 0.0 for the 400-unit atrium fall — unchanged.

## How it was verified (tests run, per binary)

- `walkreal_0700000d_live` (whole binary): `walk_arrival_to_the_c5_door` (plain, top-up,
  difficulty 1) — **ok, 1 passed**; `walk_arrival_to_the_c5_door_real_damage` (Operative,
  ignored) — all three new pins green, then the walk's documented crossfire death at tick
  21270 in the yard; `walk_arrival_to_the_c5_door_real_damage_agent` (Agent, ignored) — all
  three new pins green (park 137.585, timer 86, the health math), then the lawful death at the
  drop landing (health-0 arrival, tick 20056).
- `fix_death_restart_live` (whole binary): **ok, 8 passed** (with the new exact short/lethal
  pins).
- `walkreal_0700000c_live` (whole binary, a falling-level sanity sweep): **ok, 3 passed**.
- The full suite did not fit the session (the same "did not fit" as prior sessions). No
  `crates/*/src/` file changed — the other walkreal binaries' behavior cannot move; their
  per-level runs are the next session's routine sweep. `cargo test -p nf-game --lib` and the
  port's rule code were untouched (read-only verification).

The rule's equivalence rests on the decompile + the disassembly citations above, and the
pins' numbers (timer 86 / 76, raw 26 / 16 / the < 10 floor) are the game's own arithmetic.

## What's left

- The walk's health budget: the crossfire levers in the ignore note (the d2 snipers, the head
  act's pitch convergence) are what let a walk reach the drop above 18.2 health; the fall work
  here removed the false attribution but the walk still ends at its documented death.
- The beam's catch behavior when a player jumps late (the girder sweep parks Bond ON it, the
  carry fails on the grounded flicker, the interior landing at feet 13.095 inside the girder)
  is real geometry worth a probe session — the port's spine push-up on the MOVING girder left
  Bond 0.37 below the girder top and the ground carry did not move him with the beam during
  the flicker. Not pinned here; the game-side truth needs the live rig.
- Live checks for the main agent:
  1. **The slot fall's health delta in the real game** — park the beam, jump+drift through the
     slot from the west girder, read Bond's health before the jump and after the landing
     (PCSX2 fieldprobe poke-free). Expected at Agent: −18.2 (timer 86); at Operative −18.2
     (Normal 0.7 both — difficulty 0 maps to Normal [M 0x1902c8]); at the port both measured.
     Addresses: BL+0x928 (the timer) and the health float (BL+0x894).
  2. **The platform slide-off's timer** — walk off the warehouse platform's south edge, watch
     BL+0x928 during the slide: expected to stay 0/never accumulate past 60 (the slide keeps
     BL+0x940 set). Confirms the port's 0-damage slide pin.
  3. **The late-jump girder catch** — jump off the parked beam after the re-sweep starts:
     does the game's Bond land ON the sweeping girder (root ~14.13) and ride/fall off its east
     edge like the port, and does the game's carry hold him to the beam during the flicker?

## Usage

not visible to me — this sandbox exposes no token, credit, cost or context-window figures for
the session.
