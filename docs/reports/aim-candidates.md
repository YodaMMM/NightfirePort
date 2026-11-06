# aim-candidates — the aim snap's object stage stops on objects the game does not have

Branch `s/aim-candidates` (from `origin/s/aim-finish` 81616ca, merged `origin/master` c2c4a6e — clean). Started 2026-11-06 08:28 UTC, reported 09:47 UTC.

## What the task was

`s/aim-finish` regresses 8 walks (08 x1, 0c x1, 14 x3, 16 x1, 1b x2). The aim snap's object stage snaps to script stream entities the game's aim ray does not stop on. The task ordered four steps: fix the pre-filter constants, add the `Collide_PickObj` gates, make the candidate set the game's linked set, and re-check `Door.bullet_pass` — keeping the [L] 08 pins and quoting the six aim binaries plus walkreal 08/0c/14/16/1b.

## What is done

**1. The pre-filter constants are fixed at both sites** (`aim_snap`, `crates/nf-game/src/drone.rs`, the script-entity loop and the doors loop): `f0 & 0x3000` → `f0 & 0x30000000`.
[M] `Collide_Pick` disasm 0x1eb910: `lui $s1,0x3000` (s1 = 0x30000000), used at 0x1eb918 (`and $v0,$v0,$s1`) before the per-object `Collide_PickObj` call in the cel+8 walk (0x1eb8e8-0x1eb93c; head `cel+8`, next `obj+8`, confirmed in the disasm).
[M] the f0 word source: `Control_SetGList` 0x134960:13 `obj+0xF0 = glist+0x44 | old & 0x30008000` (the port's `world.glist_flags(hash)` = the glist+0x44 word).

**2. The hittest the object stage models is now pinned from the disassembly**, and the gate set is documented in the code:
[M] `Player_WeaponInitBullet` 0x1a4d5c-0x1a4d88: `t3 = 0x20a`, `movn t3,0x208` on `weapon+0x6c & 0x800`, passed in the **o7 slot = the hittest FLAGS word (+0x98)**; `sw $zero,($sp)` — the refine type arg is **0**; return checked at 0x1a4d90.
[M] `Collide_RayIntersect` 0x1ec690 stores the **type word 0x201** at hittest+0x94 and `param_8` (the flags) at +0x98; the 0x201 goes to the 0x200-arm dispatch, `param_9` (0 here) only to the refine's `& 0x10` early stop.
Consequences, now written into the stage's comments:
- the `Collide_PickObj` f0 mask is `0x20 | (0x40 on flags bit 0x8)` = **0x60** for both aim words — the port's existing mask was right (0x20a/0x208 have bit 0x8 set, bit 0x20 clear);
- the flags' **0x200 class-5 skip** (bullets, `obj+0xff == 5` [M Bullet_init 0x12c5a0:87]) is real for the aim ray but **vacuous in the port** — its candidate set holds no bullet object;
- the flags' 0x400/0x100 class gates (players/drones) are clear for the aim words — not active;
- the **`obj+0xfa & 8` OR `glist+8 != 0` gate** (type 0x201 ≠ 0x100) is active: script entities pass via `Script_EntityStart`'s `obj+0xfa |= 0xe4` [M 0x1d8d78:109]; for every candidate the existing `world.shape(hash)` requirement is exactly the glist+8 half [M `parsemap_block_Coll_Data_New` 0x1d1810:23-40 stores the 0x2E collision header at glist+8; the port's `shape()` is non-empty iff that block exists];
- the doors loop keeps the same mask + pre-filter (now 0x30000000).

**3. The baseline was reproduced before the fix** [M `walkreal_0700001b_live` on this branch, pre-fix]: 15 passed / 2 failed — `console 1 latched by the real shot` (walkreal_0700001b_live.rs:644) in both `walk_arrival_to_drakes_defeat` and `..._on_real_health`; consoles 3/4/5/6 latched.

**4. The chain-membership question (step 3) is evidence-pinned but NOT fixed.** The game's candidate set is the per-cel chains plus ForcedList [M 0x1eb8e8-0x1eb93c, 0x1eba38-0x1eba5c]. A script entity joins a chain only when its EntityStart arm has positioned it and its world sphere lands in a room cel [M Script_EntityStart 0x1d8d78:111-115 → build_LinkToRoom 0x1c9680:39-49 (build_FindCel); unlinked retry per tick 0x133288:61-67; ForcedList insert only for `f0 & 0x20000000` 0x1c9680:62-67; straddle rule ≥2 cels 0x133288:103-111]. I built the port-side model of that membership — `world.find_cel(sphere centre)` — as a candidate gate: on 0700001b it is a **measured no-op** (an env-gated probe printed every candidate in-cel, including the f0 0x81 panels), so it does not explain the game's NULL, and it was **not shipped** (unverified against the 14 wire legs). The real discriminator is still open (below).

**5. `Door.bullet_pass` was not changed.** The door stage inside `aim_snap` already gates on the real f0 word (`world.glist_flags`) plus the fixed pre-filter; the `Door.bullet_pass` field (the placement-flags stand-in at door creation) is untouched. What the 08 pins measured with it untouched: rocket_glass08 3/0, rocket_stop_plane08 4/0, walkreal 08 5/1 — identical to the aim-finish baseline, i.e. no pin evidence was gathered for or against the f0-word change; that experiment is still to do.

## How it was verified (each binary whole, `--test-threads=1`)

Post-fix tree, slim ISO:
- `fusebox_geometry_07000014_live`: **1 passed / 0 failed** (1.76 s class)
- `interactables_07000014_live`: **10 / 0** (15.4 s)
- `rocket_glass08_live`: **3 / 0** (2.5 s)
- `rocket_origin_live`: **1 / 0** (1.3 s)
- `rocket_stop_plane08_live`: **4 / 0** (4.2 s)
- `solar_panel_hits_live`: **1 / 0** (6.8 s)
- `walkreal_07000008_live`: **5 / 1** — `walk_the_whole_mission_on_real_input_real_damage_00agent` (Bond died tick 28756; last hits frames 1422/1564 from drones #9/#10 at d≈92)
- `walkreal_0700000c_live`: **2 / 1** — `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage` (real-damage death, walker_b2.rs:170)
- `walkreal_07000014_live`: **6 / 3 / 1 ignored** — the three `walk_to "the corridor south to the leg-2 spot": 6000-tick cap` stick failures
- `walkreal_07000016_live`: **13 / 1** — `walk_the_chasers_defeat_completes_the_mission_on_real_damage` (died to drone #12, d 0.8)
- `walkreal_0700001b_live`: **15 / 2** — console 1 / console 2 latch (unchanged from the pre-fix repro)
- `nf-game --lib`: **683 / 0** (lesson 9 check — the lib compiles and passes with the changes)

The five walkreal profiles are **identical to aim-finish's measured regression profile** (08 5/1, 0c 2/1, 14 6/3/1, 16 13/1, 1b 15/2): the constant fix is safe everywhere measured and fixes none of the 8 — the regression lives in the candidate set (step 3), not in the constants or the mask. The full four-crate suite did not fit the session; the binaries above are every one that touches `drone.rs`'s changed function plus the lib.

## What's left (the next message's work, in order)

1. **Pin the candidate discriminator.** Facts in hand: the port's `sp_bullet_targets` returns only CREATED stream entities (`streams[i].obj` set by the create op, cleared by kill [M nis.rs:851-854, 995]); on 1b every candidate's sphere centre is inside a room cel (the probe), so `find_cel` membership does not separate them; the game's dynamic list at level start held 370 objects, none within 1.0 of the snap positions [L the PCSX2 read]; the game's refine requires the object's glist and tests its triangles, with the Jointy fallback when `obj+0xfa & 8 && glist+8 == 0` [M Collide_Intersect 0x1ebaa8 ~101-115]. The three hypotheses (a script that has not created them / a wrong pose / a list the game keeps apart) need one of: the game's dynamic list sampled AT the console-shot moment, or the port's snap victims identified to a stream (player, kind, creation tick) and their game-side counterparts found. Extend the env-gated probe to print the snap victims' stream identity, then decide the gate.
2. Wire the winning discriminator, re-run the six aim binaries + the five walkreals, expect the 8 regressions to close without moving the six green aim binaries.
3. The `Door.bullet_pass` f0-word experiment behind the 08 pins (step 4 of the task).
4. Full four-crate suite before the final push.

## Live checks for the main agent

1. **Dynamic list at the 1b console-shot moment** (the decisive one): when the walk reaches the console-1 stance (the shooter near x 20, head ≈ (20, -5.2, 0) aiming at the console), walk the dynamic object list (head 0x2705b4, next +0x14) and print every object's position (+0x80 sphere centre or +0x90·translation) within 5.0 of (38.74, -5.21, ±9.415), (25.72, -2.91, ±9.415), (25.72, -2.90, ±9.60), (25.95, -4.35, -11.37), (25.72, -2.90, -11.39). Expected if "not created yet": still nothing there; if "wrong pose": objects with those hashes at OTHER positions; if "kept apart": the objects there but absent from the cel+8 chains — also walk cel 26's +8 chain (the shooter's cel) and print its members' hashes.
2. **Panel pickability**: for the panel object at (36.41, -6.36, -4.62) (class 0, f0 0x81): read `obj+0xD8` (glist) and `glist+8` — expected: either `glist+8 == 0` (no collision block → the aim ray can never stop on it, and the port's `shape()` for that hash is the bug) or non-zero (then the game's ray SHOULD stop on it and the 1b NULL needs a different explanation). Also read `obj+0xfa` (expect bit 0x8 clear for a placement object).
3. **Aim ray return at the same stance**: repeat the 0x1a4d90 check from head (20, -5.2, 0) aimed at console 1's station — expected `hit = NULL` (the port's snap must then also be NULL after the fix).

## Usage

Not visible to me — my environment shows no token, credit or context-window figures for this session. The only figures visible anywhere were the subagents' own totals, read from their Task-tool result trailers: one reported `prompt:220917 completion:10221 total:231138`, the other `prompt:185363 completion:11092 total:196455`. These are not this session's figures.
