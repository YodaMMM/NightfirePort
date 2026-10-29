# level-0a-every-objective-2 — the ring mouth and the Q-Worm cubicle: both blocker readings corrected, the install proven real, the walk-reachability gap left pinned

## What was asked

Continue `level-0a-every-objective`: un-ignore `walkreal_0700000a_live::walk_office_floor_
qworm_keyboard_hatch`. Two blockers: the ring's mouth — the prior round read the wedge at
(−12.06, −13.65) as "every static ray clear, a dynamic blocker: door #540 office_b_door's leaf"
(the port's kind-0xDB doors carry no leaf collision; the game picks doors through the ForcedList
[M `build_LinkToRoom` 0x1c9680:63-66, `Collide_Pick` 0x1eb320:369-375]) — and the Q-Worm cubicle
(objective 4's stand-in: every eye-ray to the PC blocked). Fix, walk the ring legs east and the
hatch E on real input, un-ignore, FULL suite.

## What the probes found (scratch probe_l0a2/b/c, this round — findings committed in the test comments)

**1. The mouth's "dynamic leaf" reading is DISPROVEN.** Door #540 (the door list's index 15,
kind 0 swing, glist 0x802501dd, at (−14.18, −12.96)) E-opens cleanly to pos01 1.0 (state 0x4,
the leaf matrix swung −92° along z at x −14.17) — and NO ray across the mouth changes across the
E [D probe: the lane rays at x −12.06, y 0.35/1.0/1.9 read t 0.207 before AND after]. The real
blocker is STATIC cel geometry: a pillar block whose south face sits at z −13.10 (hit normal
(0,0,−1)) spanning x −12.98..−10.9, with a west face at x −12.79..−12.98 over z −13.0..−12.1 [D
probe ray grid], and the NS wall's west face at x −10.6955 runs CONTINUOUSLY z −17.5..−11.0
(every eastward ray from z −11 to −17.5 hits it — the prior round's "wall spans z −16..−13.2" was
short). The old ring leg 23's waypoint (−12.0, −12.8) sat INSIDE that block — the walk wedged
exactly one capsule radius south of the face. The crossing is the OPEN SOUTH: every southward
ray from the pocket (x −12..−8.5, z −13.65 → −18.0) is clear [D probe], joining the game's own
N-S ai lane at node 130 (−8.52, −15.39) [D 0700000a ai_network]. The ring route is re-derived
that way (46 → 42 legs; the three gap legs deleted).

**2. The Q-Worm install never had a chance — the gadget was never equipped.** The old install
leg fired `shoot_at` with whatever the spawn loadout held; the PC's writer
[useobj.rs `PCQWorm_Update` 0x1aee20's port: a class-5 BULLET with weapon 0x58 in the PC's
contact list] needs the Q-Worm gadget itself. With 0x58 equipped through the real gadget-cycle
path, the BULLET ray passes OVER the cubicle's east partition from the stand (−18.8, 1.2,
−32.2) — the sight rays stay blocked everywhere [D the ring probes stand; the port's sight and
bullet paths differ by the pass-flag drop, world.rs `segment_bullet`] — and ch 0x48 lands from
the REAL shots in 44 ticks [D probe_l0a2c]. The cubicle blocker's mechanism is solved; the
set_channel(0x48) stand-in's reason is stale.

**3. The remaining gap is WALK-REACHABILITY of a bullet-clear stand.** The east stand's walk is
furniture-blocked (every south ray x −18..−20 from z −29 hits within 40% [D probe_l0a2c]); the
desk-north band's bullet rays are desk-lip-blocked at every dz 1.9..2.8 and dx ±0.4 [D]; the
south stand (−16.9, −34.4, bullet-clear [D]) is blocked from the nav route's end at z ≈ −31.5
[D]. docs/levels/0700000a.md's route says the vent shortcut is the way in — the port has no vent
crawl. So the mission test stays `#[ignore]` with THIS round's evidence in the ignore reason,
and the shared active test keeps the equip + the best-effort real shots, falling back to the
documented stand-in when the walked stand can't reach (it did this round — the active test is
green). The mission walk beyond the install is untested this round.

## What changed

- `crates/nf-view/tests/walkreal_0700000a_live.rs` only: the ring route's mouth legs re-derived
  (the south crossing, with the probe evidence in the comments); the install leg equips the
  Q-Worm 0x58 through the real gadget cycle, walks the east-stand approach, fires the real
  shots, and falls back to the documented stand-in (never silently — it says so in the log); the
  mission test's ignore reason rewritten with the corrected evidence. No port-code change — both
  prior blocker readings were test-side misreads; the port's collision and door systems behaved
  as documented.
- `docs/WORKLOG.md`: this task's entries.

## How it was verified (what I ran)

- `walkreal_0700000a_live`: **6 passed / 0 failed / 3 ignored** on the final tree (the mission
  test is one of the 3 ignored — re-ignored with the corrected evidence; the two glide pins are
  the other two, unchanged).
- `cargo check --offline --release -p nf-view --test walkreal_0700000a_live` clean;
  `cargo test --no-run` over the four crates clean (1048 test binaries).
- The probe binaries (probe_l0a2/b/c, scratch, not committed): the ray grids cited above.

## The FULL suite (fast runner, chunked foreground — background jobs die between tool calls here)

The sandbox restarted mid-task (the toolchain and target/ rebuilt from scratch this window). The
chunked runner recorded **425/907 binaries — 1392 passed, 2 failed, 134 ignored** (16:45 UTC,
state `/home/z/my-project/scratch/suite_l0a2_state.json`). The 481 unrecorded binaries are the
slow live tail the window couldn't fit — rerun `chunk_suite.py` until ALL DONE.

The 2 failures are environment-only: `fix_0c_pylon_live` (2 — the wgpu `adapter: NotFound`, the
headless sandbox has no VULKAN/GL device; the same trio's member the earlier reports carry).
One CRASH line: `mp_online_failures` — the binary SIGKILLs (exit 137) inside
`join_in_progress_from_the_menus_then_leave_and_reconnect` hanging on its real socket pair
(reproduced directly, exit 137 at the 100 s kill) — the sandbox's network namespace blocks it;
not a port failure, but worth a main-agent rerun on a networked host.

## What's left

1. A walk-reachable bullet-clear stand for the install — most likely the VENT CRAWL port (the
   game's own route into the cubicle's south side), then the real install replaces the last
   stand-in and the mission test un-ignores end to end.
2. The mission walk's tail (the ring legs, the hatch E) — the route is re-derived and committed
   but unexercised past the install this round.
3. The suite's slow tail (481 binaries) — rerun chunk_suite until ALL DONE.

## Live checks for the main agent

1. The cubicle approach (PCSX2, fresh 0700000a boot): walk Bond from the nav route's end
   (−14.88, 0.4, −29.15) toward the PC cubicle — record the actual walkable band around it
   (the port's furniture-blocked east lane x −18..−20 south of z −29 and the z −31.5 wall south
   of the route end are the spots to compare), and whether the real game reaches the south stand
   (−16.9, −34.4) by walking (if yes, the vent/door between is the unported mechanic).
2. The mouth pillar: confirm in the real game that the block at x −12.98..−10.9 / z −13.1..−12.4
   (south face) is solid art (a planter/column?) and that the player crosses the mouth on the
   SOUTH side as the re-derived route does.
3. `mp_online_failures`: rerun on a networked host — the port's socket pair hang (exit 137) here
   is sandbox-network-shaped.

## Usage

not visible to me (this environment exposes no token, cost or context-window figures to the agent).
