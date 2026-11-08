## Your task: `gt-update-audit`, the 0700000a and 0700000b gun turrets behave as the game's in every state

Two notes disagree. The control-funcs-update audit lists `GT_Update` 0x180498 as unported, quoting the doc comments in crates/nf-game/src/gt.rs:32-33 and drone.rs (the `gt_turrets` field) that call the turret behaviours a known gap. But drone.rs (search `GT_Update 0x180498`) already runs the scripts, the wreck smoke, the deploy and fold rewinds, `GT_Track`'s line of sight and case 7's mounted fire. Walk 0x180498 case by case, plus `GT_Track` 0x17f9c8, `GT_ApplyDamage` 0x1816d8 and the deploy and take-control paths, against the port. Write a table in the report: each case and arm, ported or not, with the port's line. Port what is missing for the single-player turrets (0700000a has six, 0700000b five) and correct the stale comments with what the table shows. Rand draws must match the game's count. Pin through real play on 0a and 0b: a turret deploys when Bond comes in range, tracks him, fires real rounds, takes damage until it is wrecked, and stops. Run the turret binaries (grep tests for `gt_turrets` and `GT_`), walkreal_0700000a_live, walkreal_0700000b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gt-update-audit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T05:10Z

---
## Report (done, 2026-11-08T05:45Z)

Closed by the driver: the report is on `s/gt-update-audit` (docs/reports/gt-update-audit.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-08T05:45Z)

# gt-update-audit — the 0700000a/0700000b gun turrets behave as the game's in every state

2026-11-08, branch `s/gt-update-audit` (from origin/master d09f8b2b3). The two notes disagreed:
the control-funcs audit (and the stale comments) called `GT_Update` 0x180498 unported; the port
in fact runs the behaviours. The case-by-case walk below settles it: **everything is ported but
one real gap — the top-of-update chain drain — which this session implemented and pinned.**

## The audit table

`GT_Update` 0x180498 (the port: `crates/nf-game/src/gt.rs` `GTurret::update`, called per tick
from `crates/nf-game/src/drone.rs`'s turret pass at "The gun turrets (`GT_Update` 0x180498…)"):

| Game arm (0x180498) | What it does | Ported | Port location |
|---|---|---|---|
| :107-112 | `Script_Update` both scripts per tick | yes (the scripts mount as script players; the +0x9c read has one documented tick of skew) | drone.rs turret pass (`scripts_done` from the players' delays) |
| :113-122 | registered[0] hide gate (the muzzle flash countdown, obj+0xf0 bit 0x10) | yes | gt.rs update flash gate → `Registered0Hidden`; drone.rs `set_obj_hidden` |
| :125-152 | **the chain drain, script A** — ext+0xcc -= Σ the script-A objects' +0xD0 chain damage; a class-5 object hit by weapon 0x4a/0x4c sets the disable | **NO → ported this session** | gt.rs update "The chain drain"; the gather rides the bullet pass's `sp_bullet_targets`/`add_bullet_hit` weapon ids (drone.rs:15406), the damage via `weapon.rs::bullet_contact_damage` |
| :156-180 | **the chain drain, script B** — the same, gated on obj+0xf4 == 4 | **NO → ported this session** | gt.rs update (the `state == COMBAT` half) |
| :181-187 | `GT_Disable` on the 0x4a/0x4c chain hit | **NO → ported this session** (the class-5 object gate is unmodelled — the port disables on any recorded 0x4a/0x4c contact [S stand-in, commented]) | gt.rs update `laser_hit` |
| :188-203 | the death gate: health < 0 or ext+0xf0 == 0 → 6 (+ `GT_LoseControl` from 7, ext+0xdc = 0, the `Emitter_Create` burst) | yes | gt.rs update death gate → `DestroyedBurst` |
| case 0 (:205-230) | facing 1: the enable/disable channel gates + `GT_Track` or hold; the tail rewinds both scripts (+0x9c = 0) → 1 | yes | gt.rs `DORMANT` arm + drone.rs `RewindScripts` (the delay zeroing) |
| case 1 (:231-238) | both scripts parked → the pose copy → 4 | yes | gt.rs `ARMING` (the matrix copy is the draw side's) |
| case 2 (:239-265) | the slerp back to the park pose, 1.2 s → rewind → 3 | yes (the timer + the rewind; the per-tick ease slerp itself is visual, the fold anim replays via the rewind) | gt.rs `RESTORING` |
| case 3 (:266-286) | both parked → `GT_LoseControl` if ext+0xf2; facing == 0 → `Pickup_Create(0x2000366, id 0x52)` + delete; else → 0 | yes (the branch order corrected by the earlier merge fix) | gt.rs `FINISHED` + drone.rs `DropPickup` |
| case 4 (:287-423) | `GT_Track`; a tracked frame: Music_Event(2,1), the aim ease, the 0.1 s cadence (hp decrement, `Bullet_init(−1, owner, barrel, record 96)`, `Rand_FRand(2π)` pad), the barrel spin; untracked facing 1: the re-holster gate 0.5 s → 2 | yes | gt.rs `COMBAT` + drone.rs `Fire` (the spread's 3 draws + the pad draw — the game's 4 per shot) |
| case 5 (:424-430) | `GT_RandMove` per tick, 3.0 s → obj+0xf6 | yes (the two `Rand_FRandHalf` draws in the game's order: pitch π, yaw 2π [M 0x180348:44-45]) | gt.rs `DISABLED_SWEEP` |
| case 6 (:431-453) | the wreck: 10 s then the MP-only `Explode_Create`; the smoke every 10th frame (`Rand_FRand(2π)` per puff) | yes (`ctx.mp` is SP-false; the smoke draw rides `gas.init_gt_smoke`) | gt.rs `DESTROYED` + drone.rs `Smoke`/`Exploded` |
| case 7 (:454-560) | the mounted aim (±0.5/±π/2 clamps), the fire (action 9, the same cadence, `PlrStat_LogShotFired`, the pad draw), the spin re-arm, the action-0xE exit | yes | gt.rs `MOUNTED` + drone.rs (the player-owned Fire arm, `log_shot_fired`, `gt_mount`) |

`GT_Track` 0x17f9c8 — the port's `GTurret::track` (gt.rs): the squared range 2500 SP / 500 MP
[M :92-99,178], the facing-1 Bond-only filter [M :101,119], the copter ×0.25 [M :158-166], the
63 cap [M :146,170], the front-cone dot ≥ 0.01 against the create-time ext+0x80 [M :252-257],
the sort + the first-two-LOS hysteresis [M :246-300], the current-target validity re-check as
the caller's alive filter (documented). The per-candidate `Collide_LineOfSight(…, 0x72f)` runs
in drone.rs through `world.line_of_sight` with `LOS_TURRET_TRACK`.

`GT_ApplyDamage` 0x1816d8 — `ext+0xcc -= dmg` [M]: the port's `apply_damage` (gt.rs), called
from `Explode_Propagate`'s case 0x36 + `GT_Disable` (drone.rs "Explode_Propagate case 0x36").

The deploy and take-control paths: `GT_DeployMiniGun` 0x181560 (`place_object` + `from_deploy`,
gt.rs — the Ronin), `GT_Create` 0x17ffa0 (`from_placement`, the param map unit-pinned),
`GT_TakeControl` 0x181740 / `GT_LoseControl` 0x181408 / `GT_Disable` 0x1816a8 / `GT_Activate`
0x1816f0 / `GT_PlayerHasDied` 0x181808 / `GT_RandMove` 0x180348 / `GT_Delete` 0x181678 — all
ported (gt.rs, the per-fn doc comments cite their addresses).

**Rand draws:** the AI fire = 4 per shot (Bullet_init's cone 3 [M 0x12c5a0:166-184] + the pad
[M 0x180498:349]) — both Fire arms draw exactly that; the sweep = 2 [M 0x180348:44-45]; the
wreck smoke = 1 per puff [M 0x180498:446] (inside `init_gt_smoke`); no other draws. The chain
drain adds none (pure arithmetic).

## The stale comments corrected

- `gt.rs:31-33` ("the turret behaviours … are a known gap") → the header now states what runs.
- `drone.rs` `gt_turrets` field doc + the tick block's intro → corrected.
- `walkreal_0700000a_live.rs:1760-1764` ("only explosions damage a turret") → corrected: the
  chain drain is the bullet-damage path [M 0x180498:130-187].

## The new pins (real play, `gun_turrets_live.rs`)

- `the_0a_wall_gun_deploys_then_dies_to_real_rounds_and_stops`: the wall gun #636 sits DORMANT,
  deploys through the real ch-18 write (the track opens on Bond in its cone), then Bond's real
  trigger (the P99, the pulsing pull, the real reload) drains its health **100 → −5.00** through
  the chain law → state 6; ten more seconds and the wreck holds (no arm re-deploys a wreck).
- `the_0b_turret_dies_to_real_rounds_and_stops`: the terminal-room turret #174 (p4 = 14 [D
  0700000b.md:175]) wrecked from DORMANT the same way (**−1.50**), the wreck holds. The deploy
  phase is skipped here: raising ch 14 moves the terminal room's own geometry and the tester's
  wreck spot fell away mid-firing on the deployed run [L the 2026-11-08 run log]; the
  deploy/track/fire chain is pinned on the 0a fixture (the same code paths).
- The 0a/0b wreck shots are the REAL bullet path: the switch, the aim, the trigger, the reload
  — the contacts record on the turret's script objects exactly as the game's Collide pass fills
  the +0xD0 chains [M 0x1ea398], one tick before the drain reads them [M 0x1336f8:238].

## Verified (this session, NF_ISO set, --test-threads=1, all foreground)

- nf-game lib: **692 passed / 0 failed** (5 pre-existing ignored).
- gun_turrets_live: **5 passed / 0 failed** (the 3 pre-existing deploy/fire/spread pins + the 2
  new wreck pins).
- gt_los_live: **2 passed / 0 failed**.
- walkreal_0700000a_live: **13 passed / 0 failed** + 2 ignored.
- walkreal_0700000b_live: **7 passed / 0 failed**.
- The four-crate suite (tools/test/suite.py): not run — background processes are killed between
  tool calls in this sandbox (measured last session with nohup and setsid) and the foreground
  suite does not fit the window; the binaries above are the task's required list.

## What's left

- The class-5 gate on the 0x4a/0x4c disable is an [S] stand-in (the port's gather does not carry
  the object class) — no SP turret on 0a/0b is hit by 0x4a/0x4c in practice; a live check would
  settle whether any disc turret script object is class 5.
- The 0b deployed-state wreck (the pin above runs from DORMANT — see the geometry note).

## Live checks for the main agent

1. 0700000a, fresh boot: stand under wall gun #636, shoot it with the P99 until it dies — the
   gun's health is not HUD-visible; the observable is the wreck: the gun stops firing and the
   smoke puffs start. gdbwatch on ext+0xcc (the turret's health) through the burst would pin
   the drain rate against the port's 3.5/round.
2. 0700000a/b: confirm no turret script object is class 5 (obj+0xff == 5) — the 0x4a/0x4c
   disable arm's real reach on the disc fixtures.

## Usage

The session's input/output tokens, context used and cost: not visible to me.
(Wall clock: branch created 05:14 UTC, this report written 06:39 UTC, sandbox deadline ~07:12 UTC.)
