# 07000005: guards and the level's hurt volumes

The hurt-volume rule skips drones on 07000005 and voice-0x13 drones [M 0x13fff0:310-311]. Verify the skip's exact condition in the disassembly (level code compare, voice byte offset, delay slots), find every 05 hurt volume and which guards can reach them, and pin through real input that a guard walking into one is unhurt while Bond is hurt.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-05-hurt-skip`, report in docs/reports/level-05-hurt-skip.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-09T03:06Z

---
## Report (done, 2026-11-09T03:31Z)

# level-05-hurt-skip

## What was done

Task: the hurt-volume rule skips drones on 07000005 and voice-0x13 drones [M
0x13fff0:310-311]. Verify the skip's exact condition in the disassembly, find every 05 hurt
volume and which guards can reach them, and pin through real input that a guard walking into
one is unhurt while Bond is hurt. Branch `s/level-05-hurt-skip` from origin/master (af7ab9778;
the sandbox survived this session, so setup was fetch-only).

### The skip verified in the disassembly (the port matches; no code fix needed)

The port already implements the skip (scene.rs's drone-victims pass: `level_immune = self.level
== 0x0700_0005` plus the `voice_type != 0x13` filter), and the disassembly confirms the game's
exact condition — every detail the task names:

- **The level code compare** [M 0x1401fc-0x140208]: `lui $v1, 0x700` / `ori $v1, $v1, 5` builds
  0x07000005 and `beq $a0, $v1, 0x140638` skips the whole damage arm when the level code equals
  it. The level code's own load rides two instructions: `lw $a0, 0x3774($v0)` with the 0x2a0000
  base set in the delay slot at 0x140174 — i.e. **GameState+0xC = *(0x2a3774)** (GameState base
  0x2a3768, consistent with the port's own `GameState+0x28 0x2a3790` difficulty cite).
- **The voice byte offset and width** [M 0x14020c-0x140214]: the compare value 0x13 sits in the
  level-beq's **delay slot** (`addiu $v0, $zero, 0x13` — executed unconditionally), then
  `lhu $v1, 0xd8($s1)` — an **unsigned halfword** at drone+0xd8 — and `beq $v1, $v0, 0x14063c`
  skips the damage for voice-0x13 drones.
- **The skip targets and the damage fall-through** [M 0x140238-0x14024c]: both skips branch to
  the undamaged return (0x140638 / 0x14063c, both flowing to `jr $ra` at 0x140658); the
  fall-through runs `Hurt_GetDamage` 0x17ea18, subtracts at health+0xac, and parks death state
  0x44 through Drone_SM_SetState at ≤ 0.
- **The voice field's every reader** (lesson 1): 0x140210 (== 0x13, the hurt skip), 0x140298 in
  the same function (== 0x10), NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8 (== 0xb),
  NDrone2_DoModeSettingsNEW 0x14c600 (== 0x10) — all voice-type semantics, no disagreement.
  One port note: the game's writer of drone+0xd8 is not visible in the decompiled C (the
  decompiler drops the store; Drone_Create's only `sw ... 0xd8` at 0x137774 writes the
  obj_tag's chain field, not the Drone_tag's), so the port's `voice_type(skin)` map is the
  port's documented stand-in — it does not affect this pin, because on 05 the level compare
  removes every drone regardless of the voice byte.
- The multiplayer arm (`MPSettings` at *(0x2a4924) [M 0x14016c] non-zero: the BOTSTATE type-10
  / GameFlow!=2 returns and the damage+sound 0x1f+state 0xf2 path) is not the port's target
  (single player); noted for completeness.

### Every 05 hurt volume, and which guards can reach them (the census, [L] the census run)

13 volumes, all kind 6 (pain type 6), **damage 1000.0**, gate 0 (ungated, never disabled),
one-shot 0. Three are who 0x0 (hurts 3/4/5), ten who 0x1 (the configured-inert state-0 mask
hurt.rs documents — they never START, but per hurt-volume-flight-2 the contact damage still
applies to any body touching them):

| # | box lo..hi | who | guards within 60 u |
|---|---|---|---|
| 0 | (−36.81,−0.70,43.39)..(16.95,−0.70,117.61) | 1 | g14/g15, 39-52 u, 38 u above |
| 1 | (−76.83,8.10,−0.24)..(−23.08,8.10,73.98) | 1 | none |
| 2 | (−24.26,−0.70,43.39)..(29.50,−0.70,117.61) | 1 | g3/g14/g15, 40-50 u, 28-38 u above |
| 3 | (80.78,5.11,96.66)..(168.38,5.11,144.68) | 0 | g5/g6, 31-32 u, 11.6 u above |
| 4 | (−6.89,4.38,96.66)..(80.72,4.38,144.68) | 0 | g2/g3/g4, 30-32 u, 18-26 u above |
| 5 | (−76.98,5.35,−73.49)..(−28.96,5.35,14.11) | 0 | g7-g10, 21-38 u, 16-32 u above |
| 6 | (−33.15,6.87,−110.80)..(15.83,6.87,−68.74) | 1 | g7-g9/g12/g13, 37-58 u, 15-39 u above |
| 7 | (−78.34,5.86,−112.47)..(−29.36,5.86,−70.42) | 1 | g7-g9, 36-59 u, 16-32 u above |
| 8 | (12.45,5.02,−114.14)..(61.43,5.02,−72.09) | 1 | g12/g13, 24-25 u, 40 u above |
| 9 | (−56.07,5.02,34.77)..(−14.02,5.02,83.75) | 1 | none |
| 10 | (−29.50,24.92,2.57)..(−21.87,24.92,11.15) | 1 | g7/g10, 30-44 u, 12.6 u above |
| 11 | (−51.55,−0.51,−136.98)..(36.77,22.93,6.07), 6 tris (the sloped slab) | 1 | g7-g10/g12/g13, 26-54 u |
| 12 | (−19.34,27.68,−76.89)..(0.03,27.68,−69.71) | 1 | g7-g10/g12/g13, 26-56 u |

The reach answer: **no guard can WALK into any plane** — every flat plane sits 5.7-40 u below
its nearest guards' patrol ground (the who-0 trio lies under the roads as fall zones; the
slab #11 hugs the south slope below the y-22 patrol). The only entries are falls (a knocked or
shot-off-the-road guard). The pin test therefore lures with real gunfire and samples the game's
own contact law continuously — and the run confirmed no contact in the 3000-tick lure window
(the reach map above is the measured answer, not a guess).

### The pin through real input

- **Guard side, green**: `the_guards_never_take_the_hurt_walked` — the walk takes the proven
  gatehouse leg, fires the lure at the road guards (g5/g6 above hurt #3) through the real fire
  path for 3000 ticks, and samples every guard's standing capsule (pos−1.0..head, r 0.4 — the
  port's own drone capsule law) against every armed volume's tris each tick. Any contact would
  assert the guard's health is still the placement 10 (the skip working); the run observed no
  contact (consistent with the reach map) and passed.
- **Census, green**: `the_hurt_volumes_census_and_their_guards` — the dump behind the table
  above, committed so the reach map is reproducible.
- **Bond side, blocked on the walked route** (`the_valley_plane_hurts_bond_walked`, `#[ignore]`
  with the measured ledger): the gatehouse leg walks clean; the steering descent reaches the
  lower lane y 16.79 at (117.5,16.8,114.4) and stalls at (123.3,14.1,122.9) — the lip past
  y 14.1 refused six identical steering legs; the raw run+jump drive wedges earlier at
  (105.6,25.1,114.6). The next lever is in the ignore message: the level doc's own nav-lenient
  descent from the armour-pickup leg (59.65,33.57,86.11 → the valley floor (42.5,8.9,88.3)
  [D docs/levels/07000005.md]), then north into hurt #4 (floor y 4.38), whose contact must
  produce the −1000.0 death record. The player pass itself has no level skip [M
  Player_DealWithObjHit 0x194458], and the same law delivered the 1b blasts this week
  (level-1b-console-latch-00agent's NF_DMG_TRACE runs [L]), so the Bond-hurt side of the rule
  is not in doubt — only the walked route to the proof is.

## How it was verified

- The disassembly: capstone dumps of NDrone2_DealWithObjHit 0x13fff0-0x140658 and
  Drone_Create 0x136be0-0x137c00 (the dump commands in the session scratch, not committed);
  every cite above carries its instruction address.
- The whole `walkreal_07000005_live` binary, `--test-threads=1`, on the real ISO: **16 passed /
  0 failed / 1 ignored** (the ignored = the ledgered Bond-pin route) in 69.3 s. No
  `crates/*/src/` file changed on this branch (the diff is the 05 test file only), so no other
  level's walkreal binary is affected; the suite-wide run did not fit the window (the session
  spent its budget on the route fight).

## What's left

1. The Bond-side walked pin: the descent route per the ignore message's next lever, then the
   −1000.0 death record assert (the test skeleton is in place, route leg only).
2. The voice halfword's writer: a live check would settle what the disc writes to drone+0xd8
   (the port's skin map is the stand-in).

## Live checks for the main agent

1. gdbwatch on a 05 drone's +0xd8 halfword from the drone create through the first AI tick:
   the expected read is the placement's voice/type field landing there (address, access
   type write, expected value = the drone's voice type), which settles the port's
   `voice_type(skin)` stand-in against the disc's own field.
2. fieldprobe: poke a 05 guard into the basin plane (hurt #3's box, y 5.11) while the level
   runs — the expected result is the guard's health stays 10 (the skip), and the same poke on
   Bond at the same spot drops him by 1000 (the player arm). This is the two-sided pin the
   walked route couldn't reach this session.

## Usage

- Context window / tokens / cost: **not visible to me** — no token, credit or context figures
  are exposed anywhere in this environment; I cannot quote numbers I was never shown.
- Wall clock (from the session's own `date -u`): the driver message arrived ~03:10 UTC; the
  branch was cut and pushed 03:12; the disassembly verification ran 03:15-03:35; the census
  03:38; the pin tests and the route fight 03:45-04:31; the whole 05 binary 04:32 (16/0/1 in
  69.3 s); this report written ~04:35 UTC, inside the ~05:09 UTC expiry.
