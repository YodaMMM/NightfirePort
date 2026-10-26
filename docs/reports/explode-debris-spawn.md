# explode-debris-spawn — report

Branch `s/explode-debris-spawn` (from origin/master d4f15b9e). Task: give each explosion a
persistent object (age/kind/script) ticked at the game's object-update position and wire
`Explode_Update` 0x17c350's debris arm over `Debris_CreateEx` 0x1ac808.

## What's done

1. **The debris arm, ported with the game's gate order** — `explode::debris_arm`
   (crates/nf-game/src/explode.rs): kind == 0 → none, no draw [M 0x17c350:25-27]; loaded script
   with `SCRIPTINFO+0x98 != 0x0600052` → none, no draw [M :28-30]; `lifetime>>2 < age` → none,
   no draw [M :31-33]; then ONE `Rand_Rand(5)` draw consumed even when `GameState_time % (r+10)`
   then fails [M :34-40]; kinds 5/6 → `MetalHash_132`, 8/10/0xB → `StoneHash_133` only on level
   0x0700008 with `switch_channels[0x53]` on [M :41-62]; a passing tick calls the create path
   with `(2·radius, pos, up, jitter, 180, 3, 4)` [M :63-67] — 3 particles, 10 draws each
   [M 0x1ac808:25-41]. The MP gate (MPSettings == 1 [M :22-24]) has no port branch to gate.
2. **Persistent blast objects** — `Drones::blasts: Vec<Explosion>` (explode.rs gained
   `script_hash`/`debris_dir`/`debris_jitter` on their own lines at the struct end);
   `spawn_blast` now records every blast (drone.rs). Ticked — with the debris particles
   `Drones::blast_debris` — immediately before the ordered object walk in `tick_inner`, newest
   first: the game keeps blasts (class 0x0F) and debris (class 0x10) in the single newest-first
   `DynamicObjList` walk [M 0x1336f8:54-68 + control_add_object_to_list 0x134698's prepend], so a
   blast created late tick N ticks first in tick N+1 and debris created during a blast's update
   tick before the blast from the next tick. Blast death = script end [M 0x17c394-0x17c3a0],
   modelled by the script's `+0xAB0` lifetime (165 for 0x0600052 [M 0x17cd18], 15 when no script
   [M 0x17cd38]).
3. **Debris integration** — `shatter::update_debris_list` (extracted from `Shatter::update`, no
   behaviour change there): `Debris_Update` 0x1aac00's life−1/fade−1/gravity/spin/move, shared
   with the breakable path.
4. **Debris kinds plumbed at every blast source** (the 8-tuple now carries the 12th arg):
   - mines (07000015's tripbombs): **0** — the inline `Explode_Create` stores it explicitly
     [M 0x17c930:128];
   - destroyable smashes (bullet + channel paths, incl. nf-view's look push): **0** —
     `sw $zero,($sp)` [M 0x12bcf8];
   - grenade fuse death + the 0x3a tripbomb destruct: **0** [M 0x12ce14];
   - flesh-impact explode: **0** — the anim-coll hit writes HITDATA+0x50 = 0
     [M Collide_Jointy 0x1ec180:125/173];
   - world impact explode (the `+0x70 & 4 | 0x100000 | 0x200000` arms): **HITDATA+0x50 & 0x3F**
     [M 0x12d084-0x12d0bc, disassembled]; the port passes its surface material id `m & 0x3f`
     [S: the m ↔ hitdata+0x50 identity rests on both indexing the 0x3037c0 EffectInfo rows —
     live check below];
   - satchel timer death (Bullet_DoTrails 0x12d890): **0 stand-in** [S: the 12th-arg stack slot
     is never written before the jal @0x12e1f0 — runtime residue, indeterminate statically].
5. **Fixed a real constant bug**: `create_debris`'s spin draw used 0.24414063 (0x3E7A0000); the
   game's word is 0x3E71463B = 0.2356194704771042 [L ee_start_05.bin; M 0x1ac808:40]. Test moved
   with it.
6. **Rand_Rand draw order pinned against the ledger before asserting**: `tests/diff_calls.rs`
   replays all 40 recorded `Rand_Rand` calls bit-exact (state in/out 0x30d0a0/0x30d0a4, y-then-x
   steps) — the arm's draws go through that verified primitive. The ledger (level 07000005) has
   no call from 0x17c3xx itself (no debris-kind blast fired in that capture), so the arm's
   per-tick draw counts are pinned by the decompile addresses + the unit tests' `rng.draws`
   deltas.

### The task premise, corrected by the decompile

The task said "debris objects exist after a mine/barrel blast on 07000015 with the game's kinds
and counts". They do not: the mine's blast carries kind 0 [M 0x17c930:128] and the barrels'
`Destroy_Smash` passes kind 0 [M 0x12bcf8], and `Explode_Update` returns BEFORE the Rand_Rand(5)
draw when kind == 0 [M 0x17c350:25-27]. Grenade fuse blasts are kind 0 too [M 0x12ce14]; the
only debris producers are the bullet impact-explode arms (kind = hit material & 0x3F, metal for
materials 5/6, stone 8/10/0xB on 0x0700008 + ch 0x53) and car/copter death blasts (kind 6
[M 0x131e70, 0x17e128] — not plumbed: the port has no car/copter death blast call sites yet).
The tests pin the evidence-backed behaviour instead, per the "the TEST is wrong" rule; the
audit's GAP-15-1 premise (07000015-audit.md) needs the same correction.

## How verified

All new assertions cite their [M]/[L]/[D] addresses; sisters included so each pin can fail:
- nf-game lib tests **561 passed / 0 failed** (`cargo test --offline --release -p nf-game --lib`),
  including 6 new: kind-0 blasts never draw/spawn (ages 1..41), non-0x0600052 scripts never
  draw/spawn, the first-quarter + modulo gates with the 1-draw/tick + 31-draw pass counts, stone
  level/channel gates + fall-through kinds 7/9/others, MetalHash_132/StoneHash_133 contents
  [L ee_start_05.bin @0x2a3718/@0x2a3728], the spin word bit-equal.
- Live (NF_ISO, slim ISO): `interactables_07000015_live` **12/12** — new
  `the_mine_blast_is_a_persistent_object_that_spawns_no_debris` walks into the z-18 fence, pins
  the persistent blast (kind 0, script 0x0600052, lifetime 165, radius 10 [D p1]) and asserts
  zero debris across the blast's whole life until the record drains; `fix_blast_scripts_live`
  3/3 (per-source scripts intact through the changed spawn_blast); `diff_calls` 6/6 (the
  Rand_Rand ledger replay).
- All four crates build (`nf-game`, `nf-formats`, `nf-audio`, `nf-view`).
- The full `tools/test/suite.py` fast runner did not run in this window (setup consumed most of
  it: fresh sandbox — rustup, py7zr, both archives, cargo fetch); the crates my tests touch plus
  the live blast tests were run as above, all green.

## What's left

- The car/copter death blasts (kind 6 [M 0x131e70/0x17e128]) have no port call sites yet — when
  ported, push kind 6 into the queue.
- The view renders no blast debris yet: draw-side needs the meshes 0x02000414/15/16/1B (metal)
  and 0x0200664/65/66 (stone) with the class-0x10 objects' pos/rot/scale — the sim state
  (`blast_debris`: model/pos/rot/vel/spin/life/fade/e8) is already there for it.
- `Bullet_DoTrails`'s timer-death kind (satchel etc.) is runtime stack residue; the port
  stand-in is 0.
- Other blast scripts' `+0xAB0` lifetimes stand in at 165 [S] (only affects record expiry; their
  debris arm is dead by the +0x98 gate).

## Live checks for the main agent

1. `gdbwatch.py` on **HITDATA+0x50** (the HITDATA_tag the bullet pass fills) during a
   bullet-surface impact on any level: expected — the byte equals the collision material, and
   `Explode_Create`'s 12th stack arg at 0x12d0bc's call equals `value & 0x3F` [M 0x12d084-0x12d0bc].
   Compare against the port's surface material id for the same surface — resolves the [S]
   m-identity above.
2. `gdbwatch.py` on the 12th stack arg (`0($sp)`) at the `jal 0x17cbd0` @0x12e1f0 (Bullet_DoTrails
   timer death, satchel 0x3b): expected — whatever residue the frame holds; pins the [S] 0
   stand-in (do the satchels spawn debris in vivo?).
3. `gdbcall.py` on **Rand_Rand** with a kind-6 blast firing in vivo (car/copter death, or an
   impact explode on a material-5/6 surface): expected — one call per blast tick inside the
   lifetime>>2 window, `GameState+0x52 % (r+10) == 0` on the calls that pass, ra 0x17c3xx
   (the ledger's ra column). This pins the modulo's absolute phase.
4. `gdbwatch.py` on **GameState+0x52** (u32) at level start on 07000015: expected — its value
   vs the level's frame counter; the port assumes the port frame == GameState+0x52 for the
   modulo [S].

## Usage

not visible to me — this session exposes no token, context-window or cost figures (no usage
panel in the environment; nothing to copy).
