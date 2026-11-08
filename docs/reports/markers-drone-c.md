# markers-drone-c — the drone.rs `[S]` markers from line 18000, settled against the game

Branch `s/markers-drone-c`: from `origin/master` 6186d8530, three work commits (9e5b516dc branch+WORKLOG, 6c02d3c8a wave-1 settlements, 64cdc7c07 the two behaviour ports), then merged `origin/master` 2998b6a76 (the guards/glass/copters batch) as 14119e3be, plus the `Bounce(..)` arity re-apply after that merge.

## What the task was

Every `[S]` (unverified) marker in `crates/nf-game/src/drone.rs` from line 18000 to the end (17 marker comments; 26 grep hits — some tags span lines and two are cross-references to other files' markers) is behaviour the port guessed. For each: read the game code it stands for, and end it one of three ways — **port** the game's behaviour and remove the marker; **prove equal** (evidence in place of the marker); or **prove unreachable**. Where a marker cannot be settled without a live read, keep it and list the exact read under "Live checks".

## The behaviour port that landed: bounce debris material

`grenade.rs` + `drone.rs` + `mp/combat.rs`: the game's contact-blast arm passes `HITDATA+0x50 & 0x3F` as `Explode_Create`'s 12th arg [M 0x12d084 `lbu $v0,0x50($s2)` / 0x12d094 `andi $v0,$v0,0x3f` / 0x12d09c `sw $v0,0($sp)` → `jal Explode_Create` 0x12d0bc], and the grenade's world contact fills `+0x50` with the triangle's surface byte [M 0x1e92d8:321]. The port's `GrenadeEvent::Bounce` now carries that byte (`segment_surface`'s mat IS it — the same byte already consumed for `bounce_coefficient` 30 lines up) and the 8th `pending_blasts` field is the real selector instead of 0. RNG: `Explode_Update` 0x17c350 draws NOTHING for material 0, one `Rand_Rand(5)` per young tick otherwise, plus 10 draws per spawned particle on the metal {5,6}/stone {8,10,0xB} selectors [M 0x1ac808:25-41] — the port now makes the game's own draw count (only 0x2a MGL / 0x1b AIMS contact blasts shift RNG; fuse arms pass 0 exactly as the game does [M 0x12ce14]). Pinned by `fix_weapon_fire_modes_live` (MGL 0x2a fired at a wall from level start): 5 passed after the change.

Mechanical test fix: `GrenadeEvent::Bounce(_)` → `Bounce(..)` at 6 sites (grenade.rs unit tests ×4, fix_weapon_fire_modes_live ×2) — pattern arity follows the enum change, no assertion weakened. Applied twice: once on the branch, again after the master merge re-introduced master's copies.

## The props marker, as merged

markers-drone-c proved the kind-0/1 prop law end to end: the dispatcher creates a Break for EVERY 0x20 row, kind-blind [M 0x1d04f0:70-78, 0x1d1b68:77-96 — kind byte merely stored 0x12c230:30]; the created object is cel-linked and picked by every collide query AND ray [M 0x1c9680:39-49, 0x1eb320:313-329, 0x1eae80:72-90, 0x1ebaa8:83-107, 0x1ec690:100-112]; `Break_Kill` removes it same-frame, kind-blind [M 0x12b970:60, 0x1336f8:70-90]; the port's static world carries none of them (breakables only draw [M level.rs:501-524]; no 0x8000/kind-0x19 double in the disc census — 55 rows on 01, 43 kind-0/1). The ready-to-go ride (kind-0/1 rows under `world::PROP_PART`, bit 44, rays NOT skipped) is committed on this branch at 64cdc7c07, dormant behind the then-false `PANE_RIDE`. Meanwhile master's glass batch turned `PANE_RIDE` ON and deliberately kept the props OUT ("making the props solid re-routes every 01 walk, a separate change" — same `Collide_PickObj` no-kind-test law cited there). The merge keeps master's `kind == 2` ride and records the verified law + the ride-ready pointer in the comment; `world::PROP_PART` stays as the reserved id. So the props marker ends as **game's answer proved (they are solid in the game); the port keeps the documented deviation** — turning the ride on is the separate change both branches point at.

## Every marker, with its line on pre-merge master and its ending

| # | master line | what it guessed | the game's answer | ending |
|---|---|---|---|---|
| 1 | 18050 | turret LOS cone origin = turret body (stand-in) | cone origin `*(ext+0xb0)+0x30` is the mounted script's root translation, written once at Script_Load [M 0x1d8248:148], read back by GT_Update [M 0x180498:248-251], and the turret body object is created from the same pos/rot [M 0x17ffa0:70-71] — the port's `t.pos` equals it numerically. New residual: the LOS segment starts at the BARREL, the port fires from the body (live check) | **proved equal** (tag retired) + documented deviation |
| 2 | 18206 | destroy-burst emitter shape/life (port renders a one-shot gas puff) | the def resolves through the runtime hashtable [M Emitter_LoadDefs 0x17ab40:147] and `0x0c000004 → def @0x0174c740` in the boot snapshots is a 20-particle burst (+0x38(u16)=20; +0x0c 0.667, +0x10 0.0167, +0x14 0.612, +0x18 1.0, +0x1c 2.0) [L ee_start_05.bin @0x0031fc10 → 0x0174c740; re-verified twice this session; identical in 01/0a] | **residual [S]** kept (per-particle emitter not ported); live check renders the game's burst |
| 3 | 18573 | InitComms all-clear: the `+0x4F8 & 0x100` arm "not modelled" | the scan is decompile-exact [M 0x139520:164-201]: no 0x600 filter, arm 1 = disabled team-1 drone (+0x44 is the team byte [M docs/fields/hitdata-spawner-ally.md:196]; Enable(1) sets 0x100 [M 0x149dd0:14,27]) breaks all-clear; arm 2 = enabled, +0x50C ≥ 1.0, +0x1A0 < 30.0, +0x44 ≠ 2 → alarm latch + `switch_channels[0x97] = 1` + `[0x98] = flag` every frame, scan stopped | **ported** (rewritten to the game's rule, tag removed) |
| 4 | 18666 | NDrone2_PreDroneControl per-player distance: "one player" | SP discs run one player object; slots 1..3 are the net mode's and hold the game's own null-slot fill 100000.0 [M 0x148db0:35-48] — the port's slot-0 fold equals SP exactly | **proved equal for SP**; MP residual noted (live check) |
| 5 | 18688 | Lock_Activate's player-class-3 pause guard "same documented limitation" | the game refuses only while pause manager 1 exists [M 0x1aed00:12]; the port's sim never ticks while the pause menu stands → refused path unreachable here | **proved unreachable** |
| 6 | 18824 | drone LOS via static collision world instead of AINetwork_TestRayCels | the cel-graph ray is unported; the swap is the same documented deviation as nav `move_test` | **residual [S]** kept; live check pins spots where the two worlds must disagree |
| 7 | 18949 | opponentless team-2 drones: forced lost-counter 16 | the game's lost arm runs EVERY frame uncapped (+0x274++), +0x270 = 0 [M 0x176a28:196-203]; the forced 16 sits inside every reader's band (lost-arms 0x148338:67/151/235, fire gate 0x17a080:24-25, hold 0x13cfb0:73) but deviates at the ends (1..15 ramp, 180 expiry) | **residual [S]** kept (porting re-routes ally combat states); live check watches +0x274 |
| 8 | 18988 | FindAlertedDrones all-pairs scan "budget" | a fresh pass covers ALL i<j pairs in one frame (the per-ROW unit budget N covers the N−1 rows [M 0x1767d0:93-113]); the cursors matter only after a 5-alerted-pair stop [M :75-80] — the port's scan IS the game's fresh pass; the 0x07000004/0x0700001b early-out [M :28-33] unmodelled | **proved equal** (fresh-pass equivalence), early-out noted |
| 9 | 19013 | speech.md §2.3 universal live gate cross-ref | cross-reference to speech.md's own marker, not a drone.rs guess | **cross-ref** (unchanged) |
| 10 | 19437 | triggers.rs 0x645 beam scope cross-ref | cross-reference to triggers.rs's own [S] scope | **cross-ref** (unchanged) |
| 11 | 19464 | standing trigger toucher upper reach (measured box) | the game's trigger contact IS the movement HITTEST capsule [M Player_Collision 0x194df0:313-338,405-408, mirrored by Collide_Update 0x1ea398:160-215, read by Trigger_Hit 0x1b1fc8:18-53] — spans feet+0.0..feet+1.86, which is how the 07000005 gatehouse quad at y 27.05 fires [D fix-truck] | **residual [S]** kept (the box stands in for the capsule refine — the Intersect_CylGeom gap); live check pins the 0x5C quad |
| 12 | 19719 | GLASS panes skipped by the RAY sets (PANE_PART stand-in) | unchanged; master's glass batch documented the same skip as "a port stand-in [S], not a game rule" in world.rs and added SIGHT_THROUGH_PART for the sight-ray case | **residual [S]** kept (now master-worded) |
| 13 | 19737 | kind-0/1 pane-row props "not proven either way" | see "The props marker, as merged" above | **proved** (game reads them solid; port deviation documented; ride ready @64cdc7c07) |
| 14 | 19935 | MP bodies' door-skip lifecycle | SP path proven (bit-CLEAR from the intro hand-back [M 0x195498:53,77, 0x1da880:44-48]); MP-only residual | **residual [S]** narrowed to MP; live check |
| 15 | 20314 | spawner stinger at template spot (old fallback) | the game passes a NULL position and `NDrone2_PlaySFX` only fills from non-NULL [M 0x141848:99-118] → the stinger plays at the world origin [M 0x137e68:191-200] — already settled on master; comment updated to "retired" | **already settled** (confirmed) |
| 16 | 20765 | sight timers `[S] fold, 0c/0d only` | seen/unseen ticks ARE the game's per-drone sight timers — every active frame each drone takes exactly one arm, uncapped [M 0x176a28:196-223]; the talk gates read the same 60/120 thresholds [M :204-205, :225] behind the sticky fa8/fa9 latches [M :206-210, 229-236] | **proved equal** (the one residual is marker 7's forced-16 arm) |
| 17 | 20781 | drone's cel = world's containing cel (no relink in port) | obj+0x20 is refreshed every moved tick (portal-crossing walk, else same-tick re-find) [M control_handle_cel_change 0x133288:135-175 from 0x1336f8:209-210] — the stored cel IS the containing cel for walking drones; only a sphere straddling two cels diverges [M 0x133288:103-111] | **proved equal** (window documented) |
| 18 | 21125 | footstep ground hit = this tick's feet ray (no stored hitdata) | the game uses the STORED hitdata at obj+0xdc that `Drone_FeetOnPoint` refreshes every collide tick (ray from obj+0x30 down −(+0xa0+0.6), mask 0xc, scoped to the drone's cel, accepted within 0.2, material zeroed on contact loss [M 0x140660:87, 99-102, 202-205]; copied into the anim from the collide arms [M 0x140968:76/197]) | **residual [S]** kept (per-drone hitdata cache not ported); live check pins a footstep frame |
| 19 | 21152 | 0xD floors ripple, game can also hit 0xE | the port drops 0xE by MATERIAL while the game excludes water spatially: the mask-0xc feet ray does NOT drop water (the water drop is HITTEST+0x98 bit 0x1 [M 0x1e92d8:273, 0x1ebaa8:310-312], never set here [M 0x140660:99]) | **residual [S]** kept, mechanism corrected; live check on a water plane |
| 20 | 21158 | footstep sound at the ray's position | the game plays at the STORED hitdata+0x40 [M 0x1cec48:68-72], refreshed the same tick (marker 18) — the two agree at a footstep tick when the ray hits the same surface | **residual [S]** kept (agreement argument recorded) |
| 21 | 21167 | decal `align2dir` stands in for `Mat_Align2Up` | `Mat_Align2Up` puts the normal on the UP row raw, X = normalize(normal×foot_dir), Z = normalize(X×normal) [M 0x1df4b0:20-85] — different rows, cross order and normalization from align2dir 0x1df2a8:21-125 → the decal ORIENTATION differs | **residual [S]** kept; live check renders both |
| 22 | 21852 | bounce blast debris stand-in 0 (no material) | machine-level: 12th arg = `HITDATA+0x50 & 0x3F` [M 0x12d084-0x12d09c]; the event now threads the surface byte | **ported** |
| 23 | 22035 | trip/mine cap tie eviction order | UNREACHABLE: same-tick ties need two same-weapdef sticks inside one refire interval (≥26 ticks apart in flight time) [M 0x12ce80:296-330 + the refire gap] — newest-vs-oldest never diverges | **proved unreachable** |
| 24 | 22170 | camera shake decay "the old shape" | the game decays viewer_tag+0x200 by −Rand_FRand(shake·0.25)/tick, clamp 0, plus a per-tick eye-jitter Vec draw while shaking [M Camera_Update 0x1bb6d8:34-45 (re-verified this session), Camera_Shake 0x1bc430:12-24, eye jitter 0x1e00d0:15-22] — porting it adds the game's per-shaking-tick draws | **residual [S]** kept (RNG-stream shift deferred with the shake work); live check |
| 25 | 22253 | psi clock = frame-derived stand-in | the game's clock IS frame-derived: psiGetTimeIn100ths = (VBlankCount − RealTimeCount)·10/6 [M 0x1094a0:5-9], re-based at every level load [M ResetMap_Load 0x1d6360:125]; floor(f·100/60) = floor(f·10/6) — the same rational, tick-for-tick equal | **proved equal** |
| 26 | 22295 | rejected-step drain 0 (waits on pane ride) | the game drains by the CORRECTED +0x30 step (breakable.rs cites); plumbing it shifts which panes shatter; master's glass batch landed the pressed-pane arm but kept 0.0 here | **residual [S]** kept (rides with the pane-ride drain work) |

Tally: **ported 2** (markers 3, 22), **game's answer proved 9** (1, 4-SP, 5, 8, 13, 16, 17, 23, 25), **already settled/cross-ref 3** (9, 10, 15), **residual [S] with live checks 12** (2, 6, 7, 11, 12, 14, 18, 19, 20, 21, 24, 26).

## How it was verified this session

Environment notes: this sandbox killed every backgrounded process within seconds (the fast runner cannot be left running — `setsid` included), and its overlay filesystem cannot reclaim image-layer space (disk filled three times; the mass relink needed ~5 GB). The four-crate test build was therefore driven in foreground chunks and the binaries run directly with `--test-threads=1` on the final tree (merge 14119e3be + the arity fix):

| binary | result |
|---|---|
| nf_game lib (unit tests, incl. grenade.rs's bounce arms) | **748 passed, 0 failed**, 5 ignored |
| fix_weapon_fire_modes_live (the debris-material pin: MGL 0x2a contact blast from level start) | **5 passed, 0 failed** |
| npc_doors_live | **6 passed, 0 failed**, 1 ignored |
| fix_guard_misc_live | **6 passed, 0 failed** |
| pane_pairing_census | **2 passed, 0 failed** |
| walkreal_07000001_live | **7 passed, 0 failed**, 1 ignored |
| walkreal_07000003_live | **6 passed, 0 failed**, 2 ignored (388 s level) |
| walkreal_07000004_live | **9 passed, 0 failed**, 1 ignored |
| walkreal_07000005_live | **14 passed, 0 failed** |
| walkreal_07000006_live | **5 passed, 0 failed**, 2 ignored |
| walkreal_07000007_live | **5 passed, 0 failed**, 1 ignored |
| walkreal_07000008_live | **5 passed, 0 failed**, 1 ignored |
| walkreal_07000009_live | **9 passed, 0 failed** |
| walkreal_0700000a_live | **13 passed, 0 failed**, 2 ignored |
| walkreal_0700000b_live | **7 passed, 0 failed** |
| walkreal_0700000c_live | **3 passed, 0 failed**, 1 ignored |
| walkreal_0700000c_barrelrow_live | **1 passed, 0 failed** |
| walkreal_0700000c_eastroute_live | **1 passed, 0 failed** |
| walkreal_07000011_live | **5 passed, 0 failed** |
| walkreal_07000011_route_live | **1 passed, 0 failed** |
| walkreal_07000012_live | **4 passed, 0 failed** |
| walkreal_07000013_live | **3 passed, 0 failed**, 1 ignored |
| walkreal_07000014_live | **7 passed, 0 failed**, 3 ignored |
| walkreal_07000015_live | **7 passed, 0 failed**, 2 ignored |
| walkreal_07000016_live | **15 passed, 0 failed**, 3 ignored |
| walkreal_0700001b_live | **16 passed, 0 failed**, 3 ignored |
| walkreal_07000041_live | **15 passed, 0 failed** |
| walkreal_07000041_wall_live | **2 passed, 0 failed** |
| walkreal_0700004a_live | **4 passed, 0 failed** |
| walkreal_0700004a_satchel_live | **2 passed, 0 failed** |
| walkreal_dump_live | **1 passed, 0 failed** |
| pane_solid_06_live | **2 passed, 0 failed** |
| dead_pane_live | **3 passed, 0 failed** |
| ammo_pane_live | **3 passed, 0 failed**, 1 ignored |
| skybridge_panes_live | **1 passed, 0 failed** |

Every main-level `walkreal_*_live` plus the door/guard tests and the pane-adjacent tests ran green on the final tree — **zero failures anywhere**.

Not run (what's left):
- The remaining ~30 `walkreal_probe0c*`/probe binaries (debug probes for past one-off issues, not level walks).
- The full four-crate suite via `tools/test/suite.py` (no background processes survive here; the nf-formats/nf-audio test binaries never got linked). The next message should run `NF_ISO=<abs> python3 tools/test/suite.py` on this branch — everything else is already built.
- The `walkreal_probe0c_door5_live` compile error seen at 19:12 was never diagnosed (it is a probe binary, pre-existing on master — it did not block the level walks).

## What's left for the next message

- Run the fast runner on this branch (binaries mostly cached).
- The 12 residual [S] markers' live checks (below).
- If the driver wants the props ride ON, the change is one filter edit in drone.rs's PANE_RIDE block + the 01-walk repairs (branch history 64cdc7c07 has the exact code).

## Live checks for the main agent

1. **Turret LOS barrel origin** (marker 1): gdbwatch on `*(ext+0xc4)`'s +0xc0 and `ext+0xd0` during a GT track on 07000000c; expect the LOS segment to start at the barrel tip, not the body.
2. **Destroy burst** (marker 2): screenshot of a GT turret destruction frame; expect the game's 20-particle burst (def 0x0c000004) vs the port's single gas puff.
3. **Cel ray vs collision world** (marker 6): two spots (one 07000001, one 0700000a) where the AI cel graph and the static collision world disagree on a sight line; record which one `Collide_LineOfSight` 0x15ab78 uses.
4. **Opponentless +0x274** (marker 7): watch a team-1 ally with no drone opponent for 200 frames; expect +0x274 counting 1,2,3,… uncapped (port holds 16).
5. **0x5C quad upper contact** (marker 11): fieldprobe 07000005 gatehouse ch 0x5C standing vs crouched under the plate; expect the game's capsule to reach y 27.05 while standing.
6. **MP door-skip lifecycle** (marker 14): local MP match; watch obj+0xF0 bit set/clear per slot at spawn/death.
7. **Footstep stored hitdata** (markers 18/20): gdbwatch obj+0xdc + HITDATA+0x40 on a footstep frame in 07000001; expect the Drone_FeetOnPoint refresh before the marker consumption, and the sound at the stored point.
8. **Water-plane ripple** (marker 19): drone standing over a water plane inside its cel (07000016 harbour); expect a ripple in the game, none in the port.
9. **Decal orientation** (marker 21): screenshot a foot decal at a slope in the game vs the port.
10. **Shake draws** (marker 24): on an explosion shake, count Rand draws per tick from Camera_Update; expect 2 (Vec + FRand) while shaking.
11. **Pane-ride drain** (marker 26): break two panes with a rejected-step walk; record the game's drain step (corrected +0x30) vs the port's 0.0.

## Usage

not visible to me (no token/context-window/cost figures are exposed in this sandbox session; the subagents used in the first session reported their own internal token counts, which are not this session's).
