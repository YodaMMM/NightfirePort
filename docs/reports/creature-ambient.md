# creature-ambient — the game's Creature objects (branch `s/creature-ambient`)

## What the task asked

Creature_Update, Creature_Move, Creature_Hurt and Creature_Delete were never mentioned in
`crates/*/src`. Find from the decompile and the disc which on-foot levels place creatures
[D], what they do and how they die, and port them with addresses (movement, hurt, delete;
draw stays a stub). If no on-foot level places one, prove that from the placements and
cite it in the code. Done = a real-input test on a placing level, or the citation; every
walkreal_*_live green; the full suite total line.

## The finding: no retail level places a creature — the citation path

**Placement kind 0xE0 (`Creature_Create`, 0x134ab8) ships on no level.** Proof:

- [D] `crates/nf-formats/tests/level_objects_census.rs::creature_kind_0xe0_never_ships`
  (committed on master, c5d721f1) walks every 07* archive in FILES.BIN through the port's
  placement parser and asserts zero kind-0xE0 rows; re-run green on the slim ISO this
  session (2026-11-04). The sibling `placement_kinds_match_the_disc` pin also stays green —
  the parse side is unchanged.
- [M] The only creator of a class-0x23 object (`obj+0xff == 0x23`) is `Creature_Create`
  0x134ab8:63, whose only caller is the kind-0xE0 arm of `parsemap_create_dynamic_objects`
  0x1d04f0 (0x1d04f0:351-357). No other spawner reaches it.
- [M+D] Class 0x23's row in the per-class dispatch table (base 0x26f890, stride 12) at
  0x26FA34 is {Creature_Update 0x135268, NULL, Creature_Delete 0x135230} — no collide
  handler. The census test is the tripwire that keeps the port in step if data ever
  changes (its panic message already demands this port).

So the task's "or the citation" completion path applies, and every real-input walk is
untouched by construction: the port's creature pool is empty on retail data.

## What the port carries (`crates/nf-game/src/creature.rs`, new)

One function per game address, all in ACTION.ELF [M; work/decomp/ACTION.ELF]:

- `Creature_Create` 0x134ab8 — the 0xf0-byte CREATURE instance (the port folds the obj
  header into a pool entry; fields carry the game offsets in comments). Params 0..4 →
  CREATURE+0xDC mode, +0xC8 radius (u32→f32), +0xD4 delay (×0.01), +0xE0 hurt flag,
  +0xE1 hurt_player [M 0x134b4c-0x134bd8 disasm]. Six 0x1c-byte waypoints at CREATURE+8,
  quats = Quat_EulerToQuat(placement rot) [M 0x134ab8:92-98]. Mode 1 requires
  cel+0x3c == 0xC023 else control_delete_object + return 0; spline mode 2 and the LList
  add at 0x316e90 [M 0x134ab8:102-109].
- `Creature_CalcPoint` 0x134cf0 — RotMatrix(euler), dir transformed in place, ray
  prev→prev+dir·(off+f13) with mask 0x20d; hit → out = hit + normal·f13; miss →
  out = to − dir·f13 [M 0x134d38-0x134e24]. f13 is the EXTENT (CREATURE+0xD8), not the
  roam radius [M disasm 0x13501c/0x1350c0].
- `Creature_CalcWaypoints` 0x134e58 — wp0 = current pos with wp5's quat; +0xcc = 0; rate
  = (Rand_FRand(delay·0.5)+delay·0.5)·REC_FRAME_RATE [M 0x134e58:16-46]; chase check
  (i≥2, hurt flag, player within (extent·4+1.1)²) bends the direction to the player;
  mode 1: off = extent, euler = {Rand_FRandHalf(0.2), Rand_FRandHalf(π/2), 0}; modes
  0/≥2: off = Rand_FRand(radius·0.5)+radius·0.5, euler = {s·Rand_FRandHalf(π),
  s·Rand_FRandHalf(π), 0}; the roam clamp is the mode-0/≥2 arm only [M disasm 0x135068];
  mode 1's orientation pass (cartesian_to_spherical_acc 0x1e3a70 + EulerToQuat).
- `Creature_Init` 0x135208 — LList_Init(0x316e90, 0xf0, 0); folded into the pool's
  `llist` [M].
- `Creature_Delete` 0x135230 — mode-1 creatures leave the LList [M 0x135230:6-8].
- `Creature_Update` 0x135268 — the cel gate (`cel == 0 || cel+0xa0 != 0`, folded open —
  deviation 2) and the hurt arm; the game runs hurt-then-move; the port's `hurt` is the
  walk+apply half driven by the contact set the caller supplies (the port keeps no
  obj+0xD0 contact list for creatures; no retail creature, no filler).
- `Creature_Hurt` 0x1352d0 — for each obj+0xD0 mover with class byte 3:
  Player_Hurt(2.0f, mover, creature pos) [M 0x1352d0:9-25], folded to
  `PlayerHealth::pain_generic` (the fusebox live-wire precedent).
- `Creature_Move` 0x135340 — the 4-state machine: state 1 sets obj+0xFA bit 0x20,
  Spline_Interp(+0xcc, +8, 0, 5, obj+0x90, 0, +0xc0) (position Catmull-Rom in both modes
  [M 0x1e1858:216-229]; spline mode 0 swaps only the rotation to identity
  [M 0x1e1858:241/275/334-345]), +0xcc += rate·FRAME_RATE_MUL (1.0 at the fixed 60 Hz
  [L ee_start_05.bin @0x30d0d8]); t > 5 → state 0; state 0 → calc → 1; state 3 → 4 →
  obj+0xF0 bit 0x20 (the bullet-passthrough/dead bit).
- **How they die: they don't.** Nothing in ACTION.ELF ever writes state 3 to CREATURE+0xDE
  — a whole-ELF scan of `sh …,0x00DE` finds exactly five sites {0x134b48=0, 0x134cbc=1,
  0x1353a8=1, 0x13540c=0, 0x135418=4} — and class 0x23 has no collide handler
  (0x26FA34+4 = NULL), so nothing reaches Creature_Hurt's receiver side either. A creature
  wanders its spline forever; `Creature_Delete` fires only from the control walk's
  delete flag. The port carries the 3→4 transition for completeness.

Wiring: `Drones.creatures` (own line, end of struct), the kind-0xE0 attach
(`creature::attach(level)`, the exact dispatch the game runs — creates nothing on
retail), and the class-0x23 control dispatch (`what == 8` in the control walk, after the
props' kind 7).

**Draw: stub.** The game has no creature draw function (drawing is class-agnostic,
View_AddObjects 0x1e62d8 queues any glist-backed object); the port's viewer mesh path for
the placement glist is not built — the merge side tracks the missing mesh pose. No retail
creature exists to render (census above).

## The independent audit and its fixes

A second agent re-derived all eight functions from the decompile + disassembly and
confirmed the load-bearing claims (param map, gates, class table, hurt path, state
machine, the no-state-3-writer scan, the LList). It found six real divergences in the
first port draft; ALL are fixed (commit c5220fe9) and the audit's evidence is cited
in-line:

- A the create-tail stall (the game sets state 1 THEN calcs at create, 0x134cbc/0x134cc0;
  the port's deferred calc left rate 0 forever) — initial state is 0 now (deviation 3).
- B mode-1's euler ignores the chase 0.2 scale.
- C CalcPoint's f13 and mode-1's off are the extent (+0xD8), not the roam radius.
- D the roam clamp is the mode-0/≥2 arm only.
- E Spline_Interp's position is Catmull-Rom in both modes; mode 0 only identities the
  rotation.
- F deviation 1's "dead downstream" claim was false — the Mat_GetNorm dir realign
  persists across the CalcWaypoints loop; the doc now says so.

## Documented deviations (all no-ops on retail — the pool is empty)

1. CalcPoint's hit branch: the port lands the waypoint ON the hit (the ray API returns no
   normal) and leaves the dir un-realigned — live downstream, bounded by the mode-0/≥2
   roam clamp.
2. The per-cel reads (mode-1 admission gate cel+0x3c, update gate cel+0xa0, mode-1 Y
   clamp cel+0x54/0x90, extent max(|glist+0x38|,|glist+0x28|)) — the port has no cel
   link; gates fold open, clamp off, extent 0.0.
3. The create-time CalcWaypoints is deferred to the first Move tick (no CollisionWorld at
   attach); identical waypoints, motion starts one tick later.

## How it was verified

- `cargo test -p nf-game --lib -- creature`: 7/7 (create param map, mode-1 gate+LList,
  the state machine loop-and-die, hurt through the game's damage factor, the mode-0 roam
  clamp, mode-1 extent steps + clamp skip, delete unregisters).
- Census on the slim ISO: `creature_kind_0xe0_never_ships` + `placement_kinds_match_the_disc`
  green.
- The full suite, run per crate under one frozen env (RUSTFLAGS/ALSA path — see the
  infra note): **nf-view 948 binaries — 2912 passed / 0 failed / 318 ignored** (every
  walkreal_*_live, npc_doors_live 6+1i, fix_npc_doors_live 1, fix_guard_misc_live 6 all
  ok); **nf-game 102 binaries — 1054 / 0 / 8**; **nf-formats + nf-audio 10 binaries —
  136 / 3 / 0**, the 3 being exactly the allowed slim-ISO `disc_golden` failures.
  **Total: 1060 binaries — 4102 passed / 3 failed (allowed) / 326 ignored.**
- Post-audit-fix re-run of all 81 walkreal_*_live + npc_doors_live + fix_npc_doors_live +
  fix_guard_misc_live binaries: **233 passed / 0 failed / 13 ignored** (the creature
  pool is empty on retail, so the fixes are dead code there — re-run anyway per the
  changed-src rule).
- Note on method: `cargo test` restarts from the first binary each invocation and the
  948-binary nf-view run does not fit one tool window, so the run went through a
  resumable per-binary runner (the same binaries cargo builds, `--test-threads=1`,
  cwd = crates/nf-view, NF_ISO set). Two early windows ran without NF_ISO and produced
  instant env-panic failures; every affected binary was re-run with the env and the red
  lines replaced — the totals above are the with-env runs. An earlier 2-way parallel
  pass was also discarded for the same reason (resource contention); the kept numbers
  are serial.

## Infra notes for the next session

- Frozen env that worked (never commit it): `PKG_CONFIG_PATH=/home/z/alsa-prefix/usr/lib/x86_64-linux-gnu/pkgconfig`,
  `RUSTFLAGS=-L native=/home/z/alsa-prefix/usr/lib/x86_64-linux-gnu`, `NF_ISO=<repo>/Nightfire-onfoot.iso`.
  Changing RUSTFLAGS between runs multiplies every dep artifact by a new hash — this
  filled the 10 GB disk twice; stale `target/release/deps` files older than the current
  env's builds are safe to delete.
- Tool windows cap at ~9.5 minutes; long loops must checkpoint to a file and resume.

## What's left

- The draw stub (above) — merge-side item; nothing to render on retail.
- Deviations 1-3 if the census ever trips (the tests then demand real placement data).

## Live checks for the main agent

Nothing is load-bearing here (the pool is empty on retail), so no live check is required
for this merge. If one is ever wanted: place a kind-0xE0 object with params 0=0, 1=200,
2=3000, 3=0, 4=0 in a level archive, boot the level, and gdbwatch CREATURE+0xDE
(the instance is at obj+0x110, obj at the placement's dynamic-object slot): expected
writes 1 (0x134cbc), then 0 on each spline completion (0x13540c) — never 3.

## Usage

not visible to me — this session's tool UI shows no token, credit, context-window or
cost figures anywhere I can read (the only numbers I can see are per-subagent token
counts in task results, e.g. the audit task reported "Tokens -> prompt:107288
completion:8859 total:116147", which is that subagent's own usage, not this session's
budget).

## ntfy failed

- curl: `curl: (28) Connection timed out after 20001 milliseconds` (and a second attempt
  at 15 s: same timeout).
- python3 urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`.
- wget: produced no output (silent with -q; the sandbox has no route to ntfy.sh — the
  same record as the earlier tasks, e.g. real-health-00agent-02-03-3's ntfy note).
