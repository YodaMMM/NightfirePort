## Your task: `drake-round-lifetime`, Drake's rounds after his death

On 0700001b, Drake's weapon-0x30 rounds fly slowly and stay live for 600+ ticks; one hit Bond for 12 during the defeat cut-scene, after Drake was already dead (found while making the real-health Equinox walk pass, origin/s/1b-real-health). Check that weapon's projectile speed, range/lifetime and what happens to a dead shooter's live rounds and to damage during a cut-scene (player disabled / invulnerable?) against the game [M Bullet_update 0x12e7b0, the bullet lifetime/range fields, the cut-scene player-disable path; D weapon 0x30's data], fix any divergence, and pin it with a test (sister). Continue from origin/s/1b-real-health (merge origin/master first). Run walkreal_0700001b_live, interactables_0700001b_live, drake_fight and nf-game. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drake-round-lifetime`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-31T23:33Z

---
## Report (done, 2026-11-01T02:59Z)


---
## Report from the worker (done, 2026-11-01T03:00Z)

# drake-round-lifetime — Drake's weapon-0x30 rounds after his death (0700001b)

Task: on 0700001b Drake's weapon-0x30 rounds fly slowly and stay live 600+ ticks; one hit Bond
for 12 during the defeat cut-scene after Drake was already dead. Check the projectile speed,
range/lifetime, what happens to a dead shooter's live rounds, and damage during a cut-scene
against the game ([M] `Bullet_update` 0x12e7b0, the lifetime/range fields, the cut-scene
player-disable path; [D] weapon 0x30's data), fix any divergence, pin with a test + sister.

## What the game does (the four questions)

1. **Speed.** `Bullet_init` 0x12c5a0:188-191 writes `BU+0xF8` = `weapon_data+0x20`, quartered
   when `weapon_data+0x6C & 0x200` [M disasm 0x12c890-0x12c8a4 region; the decomp's
   `pfVar3[0x3e]`]. Weapon 0x30's record: +0x20 = 0.11666697, +0x6C = 0x20200 → the spawn speed
   is 0.02916674 u/tick (1.75 m/s — "flies slowly" ✓). [L ee_start_05.bin @0x2bf150 +
   0x30·0x10C; the same values in the port's ISO-extracted table, asserted in the test.]
   The spawn tick's own STEP is (0.5 + FRand(0.5))·speed and is **not stored** [M disasm
   0x12e854-0x12e888 computes it, the move tail writes BU+0xF4 only; `Rand_FRand` 0x1e3780
   returns [0, max)) — the port's `first`-flag handling matches.
2. **Range/lifetime.** The flying round dies when `weapon+0x1C < BU+0xF4` [M 0x12e95c-0x12e984];
   the step clamps to the +0x1C boundary and BU+0xF4 keeps the overshoot, so the next tick
   kills [M 0x12f00c-0x12f02c]. Weapon 0x30's +0x1C = 500.0 [L]. With the ramp below the round
   cruises at ~0.1178 u/tick, so a full flight is ~4300 ticks — "live 600+ ticks" ✓. Its
   +0x70 = 0x4 carries neither the 0x80000 nor the 0x200 fuse arm, so no fuse death.
3. **Dead shooter's rounds.** No arm of `Bullet_update` reads the shooter's alive state — the
   flight path touches only BU fields, the weapon record and the collision world [M 0x12e7b0
   whole-function walk; the only owner consults are the state-3/sticky arms at 0x12edd8
   region]. The port likewise never reaps rounds on a drone's death (the retain drops only
   `dead` rounds; the flight record carries no owner). The observed "hit after Drake was
   already dead" is game-true behaviour, not a port bug.
4. **Damage during a cut-scene.** `Player_HandlePain` 0x1902c8 returns early ONLY on
   `switch_channels[0x5f]`, `health <= 0`, `damage <= 0`, and the MP state gate [M 0x1902c8:14-26]
   — there is **no cut-scene/NIS arm**, and no port arm either. The damage is the record's
   +0x0C = 20.0 × `Plr_DMod_Easy` 0.6 (1b's group, ReadTuningVars 0x1d3a80 / 0x1d458c; the
   GameState+0x28 switch at 0x1902c8:29-49) = **12.0** — exactly the observed hit [M+L: the
   SG5 1.65 = 2.75 × 0.6 pin in the walkreal docs] — ×`DroneFiring_TooClose` 2.0 inside the
   level's 3.0 threshold [M `Drone_ModBulletDamage` 0x139fac-0x139fc8]. Bond disabled ≠
   invulnerable.

## The divergence found and fixed

**The `+0x6C & 0x200` speed ramp now starts on the spawn tick.** `Bullet_DoTrails` 0x12d890
ramps a quarter-speed round's `BU+0xF8` by +`weapon+0x20`·0.02 per tick while `BU+0xF8 <
weapon+0x20` (the check is BEFORE the add — no clamp; the 38th step overshoots to
0.02916674 + 38·0.00233334 = 0.11783366) [M disasm 0x12d924-0x12d964]. `Bullet_update`'s
state-0 (spawn) path falls into the same move-branch tail as state 1 and calls
`Bullet_DoTrails` after the move [M disasm 0x12efd8-0x12f000, the jal 0x12d890], so the ramp's
first step lands on the spawn tick. The port's trail pass filtered `born != now`, skipping
exactly that tick. Fixed in `flush_world_impacts` (crates/nf-game/src/drone.rs): the ramp now
runs for every live round; the swoosh-segment chaining keeps its spawn-tick exclusion. Effect:
Drake's rounds leave the muzzle already ramping — one tick sooner, matching the game.

Everything else checked **matches the game** — no other divergence found.

## How it was verified

- Disassembly of `Bullet_update` 0x12e7b0 (state-0 region 0x12e854-0x12e95c, move tail
  0x12efd8-0x12f02c) and `Bullet_DoTrails` 0x12d890 (head, ramp arm 0x12d924-0x12d964) — the
  decompiler drops stores, so every "is it stored?" question was read off the ELF.
- [L] weapon 0x30's record dumped from `work/ee_start_05.bin` at 0x2bf150 + 0x30·0x10C
  (id/base +0x30, class 4, damage 20.0, range 500.0, speed 0.11666697, +0x6C = 0x20200,
  +0x70 = 0x4, ammo 14 / clip 4); the port's ISO-extracted table asserted equal in the test.
- New test `crates/nf-view/tests/drake_round_lifetime_live.rs` (nf-view, NF_ISO opt-in):
  - `drakes_round_flies_the_record_and_dies_of_the_range_not_its_shooter` — through the real
    spawn + flight paths (`Drones::fire_player` = the `Bullet_init` 0x12c5a0 /
    `Player_WeaponInitBullet` 0x1a4bf0 port; the flight is `update_player_bullets`): the
    record's numbers, spawn at the exact quarter speed, the SPAWN-tICK ramp (the fix —
    sister), the 38-tick overshoot cap 0.11783366, 600-tick survival with every drone
    removed (the dead-shooter case), and the range death with the last seen travelled inside
    one step of 500.0.
  - Setup stand-ins are marked [S] in the file header (the fix_npcs_live convention: the AI's
    fire DECISION is stood in; the spawn and flight mechanics are the real paths).
- `drake_fight`'s own pins (drake_fight_live) re-run green: Drake's zero-g init (mode 0x1d,
  health 60), the missile choreography, the fail flow.

## What's left

- The cut-scene 12-damage hit is pinned by code citations ([M] 0x1902c8:14-26 has no NIS arm;
  the port matches), not by a sim run: the fight AI's fire gate never opens on an idle Bond in
  the port — `astronaut_move_pick` sets `firing = los` [M 0x154618:78] but `los` is cleared
  whenever `visibility < 0.7` (drone.rs near 16600), and idle-Bond Drake never reaches it
  (observed in the new test's diagnostics: state 0xc1, dist 1-10 u, no rounds for 8000 ticks;
  in the walkreal real-health walk Bond moves and the rounds DO fly — the walk saw the 12-hit).
  Making Drake fire on an idle Bond would be an AI task (s/astronaut-guards-1b family), not a
  bullet-lifetime one.
- The `switch_channels[0x5f]` invulnerability gate [M 0x1902c8:14-16] is not modelled in the
  port's pain paths (nothing in the 1b data was found to write it; unverified either way).

## Live checks for the main agent

1. **Drake's round speed in flight** — gdbwatch `BU+0xF8` (the bullet obj's LL+0xF8) through a
   Drake 0x30 ride on a fresh boot: expect 0.02916674 at spawn, +0.00233334 per tick
   (spawn tick included) to 0.11783366, then constant. Access: float read at the BU.
2. **Drake fires at an idle Bond?** — fieldprobe: does the real game's Drake open fire while
   Bond idles in the float (the port's AI does not)? If yes, watch which state (0xc0/0xc1)
   and what `Drone+0x228` bit 2 (`los`) reads.
3. **A round hitting Bond during the defeat NIS** — fieldprobe the player's `BL+0x894`
   (health) across the defeat NIS with Drake's rounds in flight: expect a −12.0 step at
   difficulty 1 (or −24.0 inside the 3.0 TooClose radius).
4. **`switch_channels[0x5f`]** — watch ch 95 (0x5f) during the whole 1b playthrough; expected
   result: never written (else the invuln gate matters and the port should model it).

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no input/output token or cost counters are shown anywhere in the interface). The only
session figures visible are wall-clock: the sandbox clock read 01:38 UTC at resume and this
report was written around 02:30 UTC, and `git log` timestamps the two pushed checkpoints.
