# copter-death-debris — the copter's death blast throws its metal debris

## What was wrong

`explode-debris-spawn` ported `Explode_Update` 0x17c350's debris arm (the port's
`explode::debris_arm`) but nothing ever created the car/copter death blast: the port had **no
copter death blast call site**. `Copter_Update`'s state-6 arm existed as a state transition only —
the port's `copter.rs` marked the record `landed` and the doc said "the explosion itself is not
ported (no Explode objects in the sim)". So a destroyed copter vanished silently: no blast, no
`MetalHash_132` debris, no camera shake.

## The game's chain, from the disc evidence

- **The death tail** — `Copter_Update` 0x17dbb8:358-370 (disasm 0x17e390-0x17e3c8): when the
  accumulated damage `*(float*)(rec+0x64)` passes max health `*(float*)(rec+0x68)`, every state
  except 4/5 (`sltiu(state-4, 2)` skips those two) goes to state **4** — or straight to state **6**
  when `GameState+0x12` is **0x07000012** (Phoenix Fire part 2) or **0x0700000b** (Night Shift
  part 3, Tower C) [M]. The tail only *stores* +0xF4; the `Explode_Create` is case 6's own
  dispatch on a later update.
- **The blast** — case 6 (decompile :265-275; the call prepared at **0x17e120-0x17e168**):
  write the arrival channel `rec+0x72`, then `Explode_Create` at **0x17e128**, then
  `obj+0xFE |= 1` (0x17e16c-0x17e178) — the object loop deletes the copter. The call-site
  arguments, read off the disassembly (the decompiler drops the float args): `f12 = f13 = 10.0`
  (0x41200000), `f14 = 67.4` (**0x4286cccd**), `a1 = obj+0x30` (the position), `a3 = 0x00600052`
  (the blast script), the first stack arg = **6** (the u16 debris kind), then `t0 = t1 = 0xff`,
  `t2 = 100`, `t3 = 0` (attacker NULL — the car's site 0x131ea0 passes the car itself there
  instead). `Explode_Create` 0x17cbd0 routes them: f12 → `SCRIPTINFO+0x94` (the script's
  presentation scale, `swc1 $f22, 0x94($v0)` at 0x17ccfc), f13 → `EXPLODEINFO+0x2C` (the radius —
  `Explode_Propagate`'s falloff divisor — and `Camera_Shake(pos, f13)` at 0x17cc28-0x17cc44) and
  `obj+0xE8 = f13·(1/15)`, f14 → `EXPLODEINFO+0x28` (the damage, `swc1 $f21, 0x28($s0)` at
  0x17cc9c), the stack u16 → `EXPLODEINFO+0x38` (`lhu $s6, 0xd0($sp)` / `sh $s6, 0x38($s0)`), the
  script loads via `Script_Load(0x6000052, …)` at 0x17ccbc, and the NULL attacker skips
  `Control_ObjectReferenced` (`beqz $s3` at 0x17cd68). The three u8s (0xff/0xff/100) are never
  read by the callee.
- **The position** — obj+0x30 is the object walk's copy of the flight matrix's translation row
  (`*(obj+0x30) ← *(obj+0xc0)`, control_movement_object_handler 0x1336f8:386-387), and case 6
  runs no `Copter_Move` — so the blast sits exactly where the copter's last flown pose put it,
  which is the port's `Copter::pos()`.
- **The debris** — `Explode_Update` 0x17c350:21-67: while the blast script (0x0600052, lifetime
  `SCRIPTINFO+0xAB0` = 165 [L/M 0x17cd18]) is young (`lifetime >> 2 >= age`, i.e. the first 41
  ticks) and `GameState_time % (Rand(5)+10) == 0` (one `Rand_Rand(5)` draw per tick, consumed
  even when the modulo gate then fails), the switch on `EXPLODEINFO+0x38` sends kinds 5/6 to
  `MetalHash_132` (@0x2a3718, four mesh hashes [L ee_start_05.bin]) and calls
  `Debris_CreateEx(2·radius, obj+0xC0, up, jitter, 180, 3, cel, hash, 4)` — **3 particles per
  passing tick**. Kind 6 is the copter's (and the car's) kind.

**Premise correction with the addresses**: the brief located the destruction on The Exchange
(07000005 Part 1 / 07000008 Part 4). Those are two of the four copter levels [D the placement
census — 05, 08, 0b, 12 have kind-0x2F placements], and a copter *can* be destroyed there — but
per the [M] above the game only **explodes** the copter on 0x0700000b / 0x07000012; on The
Exchange the death goes to state 4 and the copter departs, no blast. The port implements the
game's rule exactly (the pre-existing `explode_level` gate, now tested both ways). The car's
death site 0x131e70 (`Car_Deactivate` 0x131d88) is the same `Explode_Create` shape with f14 =
50.0 and the car as attacker — it stays unported: the task was the copter.

## What was ported

1. `crates/nf-game/src/copter.rs` — `Copter::update` returns a third value: `Some(pos)` on the
   state-6 dispatch (the game's `Explode_Create` fires inside `Copter_Update`; the port's caller
   creates the blast object because the sim owns it). The state-6 arm writes the channel, reports
   the blast at `self.pos()` (= obj+0x30) and sets `landed` — in the game's order. The death
   tail's transition tick reports `None` (the tail only stores +0xF4). The call-site constants
   are named (`DEATH_BLAST_RADIUS 10.0`, `DEATH_BLAST_DAMAGE 67.4`, `DEATH_BLAST_SCRIPT
   0x0600_0052`, `DEATH_BLAST_DEBRIS 6`) with the disassembly addresses in their docs.
2. `crates/nf-game/src/drone.rs` (the walk head) — a case-6 dispatch queues the death blast into
   the shared `pending_blasts` slot with the exact call-site values, attacker `usize::MAX - 1`
   (the port's not-the-player, not-a-drone mark — the game passes NULL, so no drone in the
   falloff credits Bond; the same mark the beam blasts already use). Queued rather than spawned
   mid-loop, so the blast's first tick is the next frame's age-1 like every other blast's [M
   0x134630 stores obj+0xEC = the creation time], and the debris arm then runs on the existing
   `explode-debris-spawn` path (kinds 5/6 → `METAL_HASH_132`, 3 particles per passing tick).

No RNG was added: the death blast itself draws nothing; the debris arm's single `Rand_Rand(5)`
per gated tick was already ported and now simply receives a record whose kind is 6.

## How it was verified

- **Unit tests** (`crates/nf-game/src/copter.rs`):
  - `the_death_blast_args_match_the_call_site` — the constants against the disassembly's bit
    patterns (0x41200000, 0x4286cccd, 0x06000052, 6). This test caught a transcription error
    during development: 0x4286cccd is **67.4**, not 69.4 — the constant was fixed before push.
  - `the_state6_dispatch_creates_the_death_blast_at_the_object_position` — case 6 reports the
    blast at the matrix row (obj+0x30) exactly on its dispatch, sets obj+0xFE bit 1, writes the
    arrival channel; the death tail's tick reports `None`; a non-explode level's death goes to
    state 4 with `None`.
  - The full `nf-game` lib suite: **584 passed, 0 failed** (5 pre-existing ignores).
- **Real-path live tests** (`crates/nf-view/tests/copter_death_debris_live.rs`, NF_ISO):
  - 0700000b: the roof helicopter (#328, gate 0x2A [D the census; the poke stands in for the
    level's own ch-0x2A writer at 0x990]), plain 60 Hz steps, frag-grenade detonations (radius 8
    / damage 100 [L weapon record 0x34]) at the body — the same `pending_blasts` entry the
    grenade path feeds — take rec+0x64 past hp; the death tail goes to 6; the next tick deletes
    the record and `blasts` holds the death blast with debris 6, script 0x0600052, radius 10.0,
    damage 67.4, attacker MAX-1, at the copter's last position; over the 41-tick window the
    cadence spawns (this seed: 1 passing tick) exactly 3 `MetalHash_132` particles per passing
    tick and nothing outside the window; the blast dies (script lifetime 165) and throws no more.
  - 07000005 (sister): the same destruction takes the gate-0 fly-by copter to state 4, the record
    stays and flies on, and no debris-6 blast ever appears.
- **Full suite** (`tools/test/suite.py`, the four crates, `--test-threads=1`): run on the final
  tree (branch `s/copter-death-debris` at the report commit; origin/master 8a966fa3 was already
  the branch base, nothing to merge). Total line: see below.

## Suite

```
(to be filled from suite.out's total line before the finish step)
```

## What's left

- Nothing for the copter chain. The **car** death blast (`Car_Deactivate` 0x131d88 → 0x131e70)
  is the identical one-line shape (f14 = 50.0, attacker = the car) and stays unported — no car
  death call site exists in the port yet; the DEATH_BLAST_* constants and the state-6 pattern
  here are the template for it.
- The camera shake is the port's coarse stand-in (`self.shake = max(radius)` — the exact
  `Camera_Shake`/`Camera_Update` port is an open question inherited from the earlier blast
  work, documented at the `spawn_blast` site).

## Live checks for the main agent

Not required for this task's pass/fail — the chain is pinned by [M] disassembly end to end and
the live tests run the real level data. Two optional confirmations if a PCSX2 session is open on
0700000b (fresh boot, copter destroyed on the roof):

1. **The debris gate's clock**: watch `GameState+0x52` (0x2a379c) and the death blast object's
   `obj+0xEC` when the copter dies — the port stands its frame counter in for GameState+0x52
   [S, inherited from explode-debris-spawn; a save-state resume skips the boot path, so note it
   only from a fresh boot]. Expected: the first debris burst lands while
   `(now − obj+0xEC) <= *(u32*)(EXPLODEINFO+0x30) >> 2` and `GameState+0x52 % (r+10) == 0` for
   the tick's `Rand_Rand(5)` draw r.
2. **The blast position**: the game's blast object `obj+0x30` at creation should equal the
   copter's last flight position (the matrix row it flew to), not the placement spot — confirms
   the walk's obj+0x30 ← obj+0xc0 sync [M 0x1336f8:386-387] against the live run.
