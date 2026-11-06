# void-recovery-2 — the room lookup vs `build_FindCel`

Task: `s/void-recovery-2` from `origin/s/void-recovery-held` (d51212da2 = master ab80ac6ed +
9cff44e0c recovery + d51212da2 the `build_FindCel` retry). Branch pushed; this report is the
finish record.

## 1. Where the port said None, and why (the comparison)

The port's lookup itself is **not** the problem — it matches the game's data bit for bit:

- **The cel set and the boxes agree.** The game's `gbl_room_cels` from the EE snapshot
  (`work/ee_snapshot.bin`, level 07000005 per `GameState+0x8 = 0x7000005`) holds 19 room cels;
  the port's parse of 07000005 produces the same 19 with **min and max equal on every axis**
  (probe: `dmin 0.0000`, `MAX-MATCH` on all 19) [L ee_snapshot.bin 0x1fbd800 array + port parse].
  The bbox provenance is the same too: `parsemap_block_map_data_static` 0x1d1998:98-100 memcpy's
  the object record's +0x20..+0x3c into `cel+0x40..0x58` raw (no matrix), and the port's
  `MapObject.min/max` are those same file floats [M 0x1d1998; nf-formats level.rs:363]. The room
  bit mapping agrees: `parseentity_fixup_entity` 0x1d00c0 gives ONLY 0xc023/0xc047 the 0x40000 bit
  (`0x804c023`/`0x804c047`, `0x1804c023`/`0x1804c047` when `level+0x38 != 0`); every other word
  keeps its raw 16-bit value, which cannot carry bit 0x40000 [M 0x1d00c0]. The ±1000 catch-all
  cels live in viewers 5/7's **private worlds** (`build_alloc_world`, Camera_CreateCameras
  0x1bb2b8:19-22), and the player's relink passes `glb_world` [M 0x133288:64,175,238] — they never
  join the player's candidate walk, so porting them would change nothing for Bond.
- **`build_FindCel` 0x1c8cb0 returns NULL only with zero bbox candidates** (:166-168), and with
  any candidate it returns one (the raycast tie-break's best, else the FIRST candidate :226-229).
  The port's `find_cel` has the same shape (its tie-break = the game's per-candidate up/down
  `Collide_RayIntersect` with the shrinking length; no-hit → `cands[0]`).

So at every failing spot the port's None is the game's own NULL — **no room cel holds those
points in the game either**. The spots, measured on the branch:

| test | spot | nearest room (port = game) |
|---|---|---|
| fix_blast_scripts type-0 (0700000d) | [1.9717, −0.4, −2.256] | 135.6 units (room 134); the port's 15 rooms are all x ≥ 93 |
| interact_07000007 vase stand (07000007) | [60.25, 43.0, −38.0] | 3.09 units north of room 25's +z face (−41.09) |
| interactables_07000002 load point | exit centre +0.5x | the placement helper kept Bond LINKED with `prev = new`, so the relink's recovery clamped the NEW position into the OLD cel's box |
| fix_death_restart falls (07000011) | spawn +8 / +400 | above every room at +400 (no candidate) |
| interact_07000006 well shaft | [31.08, 35.56, 17.91] | 0.44 below room 34's box bottom (36.0); no room reaches the shaft floor in the game either |

## 2. The fix: placement ≠ poke

The held branch treated **every** >3.0 move as a poke of obj+0x30 alone (obj+0x40 keeping the old
position) — the state where the game's linked-path recovery [M control_handle_cel_change
0x133288:186-231] clamps Bond back into the old cel's box (measured live on 07000003, pinned by
`void_recovery_07000003_live`). But the game's PLACEMENT — `Player_StandAtNewPosition` 0x190d88 —
writes obj+0x30 **and** obj+0x40 (:21-28), unlinks (:118), relinks through `build_LinkToRoom`
(:119), and **a FindCel miss leaves the object unlinked with no recovery** — the unlinked retry
path 0x133288:63-73 returns without one, and for the class-2 player only raises the fell-out flag
(`Drone_FellOutMap` 0x13a878: `obj+0xfe |= 1`, no reposition).

The port now has both paths:

- `Sim::place_bond(p, yaw)` [crates/nf-view/src/scene.rs]: pos = prev = p, vel zeroed (the
  game's `Player_ClearInertia`), unlink, one `find_cel(p + 0.1y)` relink — a miss leaves Bond
  unlinked AT the spot, where the per-tick relink keeps retrying and `out_of_cel_recovery(None, …)`
  is a no-op. Deviation [S], cited in the doc: the game's `build_PointOnFloor` floor snap
  (y = floor+1.1, 0x190d88:104-114) is not taken here — the placement tests' spots stand near
  floors and settle by physics; the snap is the load spawn's `Bond::stand_at` shape.
- The >3.0 arm keeps the poke semantics (the measured recovery) — `void_recovery_07000003_live`
  still passes.

The five placement-driven failing tests now place through `place_bond`:
fix_blast_scripts_live, fix_death_restart_live, interact_07000006_live (settle_near),
interact_07000007_live (fire_sweep; only on a real jump — the second sweep's `pos = from` is a
no-move on the settled spot and must not shift the shot sequence), interactables_07000002_live
(place).

## 3. The gate

- The five binaries above: **all green** on the branch (3 + 8 + 9 + 7 + 18 tests; run at
  20:31-20:36 UTC with `--test-threads=1`).
- Full suite + walkreal set + recovery counts: see "Suite run" below.
- `Sim::recovery_count` (and `recovery_last`, and `NF_RECOVERY_LOG=1` eprintln per fire) is the
  durable instrument for the walks' recovery-fire count.

### Suite run

(filled after the run)

## 4. What's left

1. **use_census_07000007** (a real-input walk): a recovery LOOP at [94.84, ~43.1, −72.19] —
   Bond's cel 15 box is [80.75, 42.75, −73.25]..[98.5, 54.25, −59.17] [M debug_scope_cells]; the
   walk's push-out puts him outside the box, `build_FindCel` misses (no other room holds the
   point), and the recovery yanks him back every ~2 ticks (dozens of fires from tick 3688), so
   the walker's E press at the door lands out of range ("E on a door must move it"). This is a
   REAL walk — if the game's player walks there without being shoved, the port's push-out or the
   box is the divergence. NOT fixed.
2. **urn_break_live `shooting_the_sky_bridge_starts_its_break_script`** (07000012): NO recovery
   fires (0 over 120 standing ticks at the spawn) — this failure is NOT recovery-related; it is
   in the driver's failing list for the held branch and its cause is elsewhere in the held
   branch's scene.rs changes. NOT investigated further for time.
3. The interact_07000006 well test now passes with Bond UNLINKED on the shaft floor — the game's
   placement would leave him unlinked there too, but the game's unlinked player raises the
   fell-out flag every tick; whether the game's well shaft is actually enterable needs a live
   look (below).

## Live checks for the main agent

1. **The vase stand (07000007)**: teleport Bond (a real placement, e.g. a checkpoint-style
   `Player_StandAtNewPosition` call, NOT a raw memory poke) to (60.25, 43.0, −38.0). Expected if
   this report is right: Bond stays there, unlinked (obj+0x20 == 0), obj+0xfe bit 0 set; NO
   position clamp. Then a raw poke of obj+0x30 to (60.25, 43.0, −38.0): expected — Bond clamped
   back toward the pre-poke position inside ~1 s (the poke recovery).
2. **The well shaft (07000006)**: walk Bond into the shaft mouth 0.7 off the barrel (31.12,
   40.5, 17.26) with a real pad. Expected per the decompile: the recovery pins Bond at y ≈
   37.06 (room 34's min 36.0 + radius 1.0576) and he cannot reach the 33.59 shaft floor. If the
   game lets him stand at 33.59, some room coverage the port lacks exists at runtime.
3. **The census loop (07000007)**: with gdbwatch on Bond's obj+0x30/0x40 during a walk along the
   corridor at (94.84, ~43, −72.19): does the real game's `control_handle_cel_change` run the
   recovery there (obj+0x30 yanked to 43.81), or does Bond walk through?

## Usage

(not visible to me — this sandbox exposes no token, cost or context-window figures for the
session; nothing to copy)

## ntfy failed

curl: `curl: (28) Connection timed out after 20001 milliseconds`. python urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`. wget: `WGET_REAL_EXIT 4` (network failure), no body. All three fallbacks failed.
