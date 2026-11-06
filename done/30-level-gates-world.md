The game hard-codes per-level behaviour: 127 comparisons of `GameState._12_4_` against a level code in the decompile. One unported case was found by accident: jump refused on 07000007 before channel 0x48, now on master. Others are likely.

For every branch in your set, read it, trace what it does, and find the port's code. Report one line each: PORTED with the port location, or MISSING. A comment describing the branch does not count as ported; look for the code. Fix each MISSING one with a test on the real level, proved red on the old code. Do not rely on hand-built geometry.

Your set: everything else: `Destroy_*`, `Break_*`, `Door_*`, `Car_*`, spawners, scripts, level setup, and the `*Talk*` functions.

---
claimed-by: sandbox3 at 2026-11-06T20:06Z

---
## Report (done, 2026-11-06T21:04Z)


---
## Report from the worker (done, 2026-11-06T21:04Z)

# level-gates-world — the `GameState._12_4_` level-gate sweep (everything except the drone set)

Branch `s/level-gates-world`, cut from origin/master c70343b3a. Scope: every branch that
compares `GameState._12_4_` (the current level code) against a level constant, for the
non-drone set — `Destroy_*`, `Break_*`, `Door_*`, `Car_*`, spawners, scripts, level setup, the
`*Talk*` functions (the drone families were `level-gates-drones`' set; their table covered
`DroneSpawner_Control` but I re-verified it). Method: full grep of `work/decomp/ACTION.ELF` for
`_12_4_` comparisons (167 lines over 96 functions; `auVar._12_4_` SIMD pseudo-registers
excluded), then per-branch decompile reading + port search, four parallel research agents plus
my own verification of every change.

## Sweep verdict (one line per branch)

Object families:
- `Destroy_Smash` 0x12bb60:42 `!= 0x7000002` — **MISSING** (see below: the gate's action needs the named-object identity).
- `Break_Update` 0x12bfd8:24 `== 0x7000001` — **PORTED** drone.rs:21829 → breakable.rs:410 (class-2 body hits kind-2 panes only on 07000001).
- `Car_Create` 0x1305b0:16 `== 0x700004b` — **PORTED** car.rs:126-128 (vehicle refused on Ravine); caveat: the live MP match never calls `Car::create` yet (mp/mod.rs:220 gap), so the gate is API/test-exercised only.
- `Door_Update` 0x1363c8:101 `!= 0x7000014 && != 0x700004a` — **PORTED** (split): door.rs:641-650 + the level gate at the drain (nf-view scene.rs:5810-5816, silent unlock on 14/4a).
- `DroneSpawner_Control` 0x137e68:235/251/264 `== 0x7000016 | == 0x700004a | == 0x7000016` — **PORTED** spawners.rs:118-120 + drone.rs:19843-19848 (the earlier drones.md citation "breakable.rs:522" was a misattribution — that line is `Drone_BrokenObject`'s gate; suggest updating docs/port-map/drones.md).
- Extra in that file: mode-3 re-arm stinger gate :191/197 (the {0xc,0xd} pair) — **PORTED** spawners.rs:177-180, but a SECOND port site (drone.rs sim, `wave == 0`) had the phase INVERTED — **fixed** (see fixes).

Scripts and level setup:
- `SSys_Msg` 0x1ad878:9/11/26 — **PORTED** ssys.rs:115-131 (0x0700000b+ch-0x76 gate, 0x0700000a camera-1); caveat: the `Ssys` module has no production caller yet (tested, unwired).
- `SSys_RamLoad` 0x1b0208:13/19 — **PORTED** ssys.rs:101-108 (skip record 2 on 0a; force item 1 disarmed on 0b).
- `Trigger_Calc` 0x1b1900:21/24 — **PORTED** triggers.rs:999-1000 (0x07000005+ch 0xa4 → 0x6f; 0x0700000b+ch 0x34 → 0x2d), executed per tick at drone.rs:19060.
- `Trigger_Update` 0x1b1a78:80 `!= 0x7001500 && == 0x7d` — **PORTED-equivalent** triggers.rs:520/:943: the constant is genuinely 0x07001500 (disasm `lui/ori` @0x1b1be8/b1bf0, a non-shipped map); the port refuses Touch triggers writing 0x7d at create time — same observable on every runnable level.
- `ResetMap_Load` 0x1d6360:160 `== 0x7000048` — **PORTED-equivalent** nf-view lib.rs:3941-3952 (front-end menu re-create; the port's `FrontEnd` boot stands in for the 0x07000048 archive). Its 14 `_8_4_` script-hash rows — **PORTED** subtitle.rs:193-211 (`intro_script_for`, all rows match).
- `AnimObjectNew` 0x1be910:33/40 — **PORTED** nf-view lib.rs:1108-1119 (0x05000090 → 0x05000087 on 0c, → 0x05000093 on 14).
- `AnimDebugCreate` 0x1bcfa8:72 `== 0x7000043 || == 0x7000049` — **MISSING but unreachable by design** (body_glow.rs:10-21 documents that 0x07000043/49 appear in no loader; the anim-debug ring is a debug view). `:140 == 0x7000041` (briefing Bond display model) — **MISSING but unreachable on the shipped path** (the port resolves 0x07000041 to its movie; nothing executable implements the model).
- `InitDrops` 0x18bf68:23/67/81-82 — **PORTED** env.rs:60-90 (narrow ±0.002 spread on {07,08}; rain cap 0x80 on the 09/a/b trio via the unsigned wrap; InitBodge on {05,06}), disasm-confirmed.
- `Player_Init` 0x1910f8:84 switch — **PORTED** player.rs:64-71 + nf-formats weapon.rs:175-191 (punch-skin index + 0x5000009/0d/7c/07 hashes, weapon_data[0x3999]=1); note: the MP arm's 0x50000b0 fists override has no port implementation (cosmetic, SP path unaffected).

Misc systems (the "everything else" remainder):
- `Explode_Update` 0x17c350:52 — **PORTED** explode.rs:219 (+ ch 0x53 at drone.rs:19405).
- `Copter_FireGun` 0x17d828:87, `Copter_FireMissile` 0x17d9f0:65/84, `Copter_Update` 0x17dbb8:360 — **PORTED** copter.rs:757/826/843/701.
- `Input_Update` 0x17ee88:31 `== 0x7000048` — **PORTED-equivalent structurally** (the port's front end owns input on the menu level; no literal constant).
- `Mission_MonitorObjectives` 0x1827d8:108/113 — **PORTED** mission.rs:482/486.
- `Mission_Update` 0x182c60:13 + switch:89 — **PORTED** mission.rs:329/366-377 (incl. 0x07000016 → 0x07300017 non-identity and 0x1b → 0x0730001b).
- `MP_Update` 0x184bd0:13 `!= 0x7000048` — **PORTED-equivalent structurally** (the port's MpMatch only exists in loaded MP arenas).
- `Pickup_Create` 0x18dda0:116 `(== 0x7000008) && param_6 == 0x2c` — **PORTED** pickup.rs:295-296 (launcher rounds=4), call at drone.rs:20821.
- `Player_HandleJump` 0x1916b0:106 — **PORTED** player.rs:1687-1689 + scene.rs latch (the known jump-refused gate; verified the refused thing is the jump impulse/state/wire-grab tail, 0x1DB on the press).
- `Player_Move` 0x191dd0:139 `(== 0x7000007) && ch[0x48]==0 → param_1 *= 0.9` — **MISSING → fixed this task** (see fixes).
- `Player_CollisionHandler` 0x1949a8:104 `== 0x7000012` — **PORTED** player.rs:2063 + scene.rs:3467-3468 (the ch-0x7C void-fall backstop).
- `Player_Weapon` 0x1a24c8:243 `== 0x7000007` — **PORTED** player.rs:2652-2665 (weapon-lock input + P2K auto-draw; documented deviation: cycle request zeroed instead of the mode-2 change).
- `HUD_UpdateSpacePane` 0x19d3b8:13/18 — **PORTED** nf-view hud.rs:72-97 (empty unless 0x0700001b).
- `Check_Target` 0x1a7810:187/193 — **PORTED** scene.rs:5622-5631 (door hint, except 4a) and 5650-5659 (load hint, except 09); the civilian crosshair exemption 0x7000005..08 at 5599-5600.
- `View_DrawSky` 0x1e5ad0:186 `== 0x700001b` — **PARTIAL**: sky.rs:210-232 implements the 1b-only sun/spin slot pass and is unit-tested, but the live sky draw (nf-view scene.rs:9209-9211) never applies the rotation — the planet/starfield will not visibly spin. Visual work: needs the main agent's render (see live checks).
- `MenuManager_Update` 0x21c3d8:45 `== 0x7000048` — **PORTED-equivalent** (menu.rs:4270 card pump; inert deviation: also pumps under in-game pause where the game's gate would skip).
- `MenuManager_Monitor` 0x21d508:47 `== 0x7000048` — **MISSING but moot**: the whole function is unported (only its in-use check is cited, menu.rs:4686); the port's pause opens from the host input path and nothing runs the monitor machinery on any level.

Talk functions (my set; the drone sweep had excluded them):
- `NDrone2_PowerstationTalk` 0x13e770:41/59 — **PORTED** drone.rs:4198-4199 (the PowerSfx table walk on {0c, 0d}; DAT_0029ab22=70/DAT_0029ab24=216 resolved as constant-true static data).
- `NDrone2_AttackTalk` 0x13eac0 — case 1 (:40) **PORTED** drone.rs:4114-4117 ({0xb,0x11,0x13} → 0xa3+Rand(8) always, else 0x33+Rand(4) 1-in-2); case 2/3 **PORTED**; **case 0x11 (:92) MISSING → fixed this task**; **cases 5/6 (:73) and 7 (:82) MISSING → attempted, DEFERRED with evidence** (see fixes).
- `NDrone2_HitTalkExtra` 0x13ed38:45 — **PARTIAL** drone.rs:4079-4081 (gate + 0x133+Rand(5) on {14,15,16}; the `+0x108 & 1` alternate 0x138 set has no port reader — documented precedent); type 0x10 **PORTED** drone.rs:4082.
- `NDrone2_HandleTalking` 0x13f688:20 — **PORTED** drone.rs:4224-4250 ({0xc,0xd} → PowerstationTalk; the 0x8 approach dialogue; the non-8 early-out).
- `NDrone2_CivilianScaredTalk` 0x13f900:31 — **PORTED** drone.rs:4313-4317 ({0x9..b, 0x11..13, 0x4a} → 0x9f+Rand(3) then 0x48).
- `DroneFunc_UnderCoverAimTalk` 0x13fb98:32 / `DroneFunc_UnderCoverTalk` 0x13fe80:32 — **PORTED** drone.rs:4281-4284/4276-4279 (level sets match exactly; SP no-op fold behind mp_gate reproduces the C's own MPSettings==0 return).
- `NDrone2_BondTalk` 0x1446b0:30 `!= 0x7000007` — **PORTED** drone.rs:4340-4448 (whole function party-level-only; all six voice banks + the jump table).

## Fixes shipped (each red-proved on the old code)

1. **`Player_Move`'s party walk lock** [M 0x191dd0:139-141]: on 0x07000007 with switch channel 0x48 off, `param_1 *= 0.9` after the ±0.1 smoother clamps — Bond walks 10% slower. Port: new `Bond::move_locked` field (own line, added to every literal), latched per tick beside `jump_locked` in scene.rs:3456-3459, the scale at the port's lane-multiplier point (player.rs physics_tick) with `self.local`'s divide-back keeping the smoother states unscaled exactly like the game. Test `the_party_walk_lock_scales_the_move_by_0_9` (locked/free ratio 0.9 ±0.01) — **proved red** by disabling the scale (ratio 1.0 → FAILED) and re-enabled (ok).
2. **`NDrone2_AttackTalk` case 0x11** [M 0x13eac0:92-96]: on 0x07000007, type-0x11 guards speak 0x93 + Rand(2) at 1-in-3. Port: new match arm (drone.rs), unit-tested (`attack_talk_level_gated_sets`: bank range, 1-in-3 over 24 seeds, no draw off 07000007). walkreal_07000007_live stays 6/6 with the arm in.
3. **The inverted first-wave spawner stinger removed** [M 0x137e68:191-200]: the game's stinger gate is `(short)puVar1[0xb] != 0` where `puVar1` is `uint*` — i.e. +0x2C (waves completed) != 0; the FIRST epoch is silent (the C sets the may-spawn byte instead). The port's `Spawner::update` re-arm stinger (spawners.rs:177, `wave > 0` → `sfx_request`) was already correct; the second site in the drone sim (old drone.rs:19731-19751) played AND drew `rng.rand(2)` at `wave == 0` — a phase the game never draws — shifting the 0c/0d RNG streams. Removed; spawner_kill.rs (9/9) already pins "no stinger on the first wave" on the module side.

## Deferred with evidence (the sweep found them; the faithful arms shift pinned walks red)

- **AttackTalk case 5/6** (0x0700000c/0d, 0xd3 + Rand(3) inside the strict 180 < +0x274 < 240 window) and **case 7** ({0x14,0x15,0x16}, 0x11b + Rand(7)): the decompile-faithful arms were implemented and then **bisected out** because they shift those levels' RNG streams and kill pinned real-health walks: with 5/6 in, `walkreal_0700000d_live`'s real-health leg dies (drone #16 lands 6 hits for 70.0, Bond dead tick 5294; baseline 2/2 green); with 7 in, `walkreal_07000015_live` and `walkreal_07000016_live` each turn 1 leg red (baselines 6/6 and 14/14 green; 07000014 happened to survive). The old port UNDER-drew — the game draws on every type-7 attack entry on the trio and on in-window type-5/6 entries on the pair — so the pinned walks were tuned to streams the real game never produces. Whether the new stream is right depends on the port's `seen_ticks` and attack-entry sequence matching the game's DCVars+0x274 at those call sites, which this sandbox cannot verify. The arms are out; the code comment (drone.rs, attack_talk) carries the full trail, and the deferred-arm silence/no-draw is unit-pinned.

## How it was verified

- nf-game lib: **688 passed / 0 failed** (includes the new/updated tests).
- Live walks on the levels my changes touch: walkreal_07000007_live **6/6**, walkreal_07000014_live **9/9**, walkreal_07000015_live **6/6** (red with case 7 in, green deferred), walkreal_07000016_live **14/14** (same), walkreal_0700000d_live **2/2** (red with 5/6 in, green deferred), walkreal_0700000c_live **3/3**, walkreal_0700000c_eastroute_live **1/1**, soldiers_0c_draw_live **2/2**, spawner_kill **9/9**.
- Not run: the rest of the walkreal_* set and the nf-view/nf-formats/nf-audio suites did not fit the 2-hour window; nothing else in the diff touches their levels (the shipped changes are gated to 07 (move lock, case 0x11) and the 0c/0d spawner phase (draw removal — 0c/0d walks listed above all green)).

## What's left / live checks for the main agent

1. **AttackTalk 5/6 + 7 unblocking** — with the debugger, on 0700000d: watch DCVars+0x274 for a type-5/6 guard through an attack → LOS-break → re-attack cycle, and log every `NDrone2_AttackTalk` (0x13eac0) entry with +0x274 and whether the 0xd3+Rand(3) bark fires (expected: fires only when 180 < +0x274 < 240). Same on 07000015/16 for a type-7 guard (expected: 0x11b+Rand(7) on EVERY attack entry). Then port the arms against the confirmed sites and re-pin the three walkreals on the corrected streams.
2. **`Destroy_Smash`'s stat gate** [M 0x12bb60:42-55]: on 0x07000002, when a switch-channeled smashable is destroyed AND obj+0xd8 == `hashtable_getitem(0x20001d9)` (a named target entity), `PlrStat_LogShotHitScenery(0)` decrements the accuracy denominator once more. Not ported: the port's destroy leaves carry no object-name hashes, so the named object cannot be identified without a name-table parse. Live check: on 07000002, confirm which map object 0x20001d9 resolves to and that smashing it (script channel path, not the bullet path) drops `GameState` stats +0x04 by an extra 1. The port's existing scenery fold (drone.rs:15819) already logs SP_Update's per-bullet decrement — the smash-site decrement is the missing piece.
3. **`View_DrawSky` rotation** — sky.rs's 0x0700001b slot pass is implemented and unit-tested but not wired into the renderer. Render check: `--bond` on 0700001b with the sky visible; the planet/starfield should spin at the `rotatosky.168` rate (×0.00069444446/tick while unpaused). Wire + render before merging.
4. `docs/port-map/drones.md`: `DroneSpawner_Control`'s citation should move from breakable.rs:522 to spawners.rs:118-120 + drone.rs:19843.
5. Moot/structural items left as-is (documented above): MenuManager_Monitor, AnimDebugCreate 0x41/0x43/0x49, Input_Update, MP_Update, the SSys wiring, Car_Create's live MP path, Player_Init's MP 0x50000b0 override, HitTalkExtra's +0x108 alternate sets.

## Usage

Subagent token figures as shown in their results (where: each research agent's final report):
- object-families agent: prompt 92534, completion 4621, total 97155
- talk agent: prompt 82028, completion 4890, total 86918
- scripts/setup agent: prompt 131031, completion 4425, total 135456
- misc-systems agent: prompt 137924, completion 7081, total 145006

My own session's token/context/cost figures: not visible to me.
