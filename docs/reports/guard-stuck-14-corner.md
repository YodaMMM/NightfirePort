# guard-stuck-14-corner — the 07000014 corner round

## What was done

**The base-branch detour.** The task said to branch from `origin/s/guard-stuck-0c-14` and read
its `docs/reports/guard-stuck-0c-14.md`. That branch no longer exists on the remote
(`git ls-remote origin | grep stuck` is empty), its report never reached master, and the queue's
`done/20-guard-stuck-0c-14.md` only says "the report is on `s/guard-stuck-0c-14`" — a branch the
driver later dropped. I branched from `origin/master` (784ba2d99) instead and reconstructed the
0c-14 context from its traces on master: the leaf-slide fix (drone.rs `door_leaf_slide`, levels
07/14) and the sweep test's own comment naming this round's item ("the game's LinkCreep_AtSpecialNode
tier-2 ... the port asks the player ... is the divergence candidate").

**1. The dropped radius, pinned [M].** `NDrone2_DroneNearPos` 0x143e68's decompile reads a float
first parameter it squares and tests `Vec_SqDist2D < r²` against (0x143fb8-0x143fc0), but the
decompiled call site `LinkCreep_AtSpecialNode` 0x159dd0:58 shows only two arguments — the
decompiler dropped the radius. The disassembly settles it:

- `00159eb8 lui $at, 0x3ecc` / `00159ebc ori $at, $at, 0xcccd` → 0x3ecccccd = 0.4f;
- `00159ec0 mtc1 $at, $f12` — the same compare at `00159ec8 c.olt.s $f1, $f12` uses it, and
  `$f12` is NOT clobbered before `00159ef8 jal 0x143e68` (the 1.0 compare at 0x159ed8-0x159ee4
  goes through `$f0`, the callee args `$a0` = the node VECTOR via the delay slot
  `addu $a0, $s1, $zero`, `$a1` = the self obj via `lw $a1, 0xf8($s0)` = route byte offset 0xF8);
- the callee reads the radius from `$f12`: `00143eac mul.s $f20, $f12, $f12`, tested at
  `00143fc0 c.olt.s $f0, $f20`; the y gate 2.0 (0x40000000) at 0x143f38/0x143fe0;
  the player tier 0.55² = 0x3e9ae148 (0.3025) at 0x143ed4-0x143edc.

So the tier-2 radius at the route call site is **0.4** [M 0x159eb8-0x159ec0 + 0x143eac].

**2. The tier-2 ported to the route-follow call sites.** `NDrone2_DroneNearPos` 0x143e68 has
exactly three callers in the decompile: `LinkCreep_AtSpecialNode` 0x159dd0 (the route follow),
`NDrone2_NearDrone` 0x14a6e8 and `DroneFunc_AnyoneNear` 0x14a718 (both script DCVars, both
passing their own radius — not the route path). The route arm was the gap: every port
`Route::follow` call site passed a player-only closure. The port's route-follow closure cannot
reach the live drone list in place (the borrow split), so the list rides the snapshot the task
suggested:

- new `Drone::near_peers: Vec<(usize, V3)>` — the (index, pos) of every drone passing the
  class-2 gate [M 0x143f58-0x143fa8: `(ext+0x4F8 & 0x600) == 0 && (& 0x100) != 0`,
  `+0xAC > 0.0`, class `+0xFF != 0x11` — that arm is `BOT_respawn` 0x124e38:90's MP respawned
  bot, SP never writes it — and obj+0xFE bit 0 clear], folded to the established door.rs
  class-2 mapping (present ∧ enabled ∧ non-corpse ∧ health > 0); built pre-walk in
  `Drones::tick` next to `peers`, lent per drone like `peers`/`others`.
- `Drone::near_pos_fn(ppos, my_index, near_peers)` — the full game law: the player tier first
  [M 0x143ec4-0x143f20], then any gate-passing peer within 0.4 2D (0.16 = r²) and 2.0 in y
  [M 0x143fb8-0x143fe8], the caller itself skipped by index [M 0x143f48]. Wired into all five
  `Route::follow` call sites (update_patrol_route, update_mission_route, move_to_goal_r,
  ENEMY_MISSION UPDATE, the combat chase) — the closures consult it exactly in the 0.4-1.0
  special-node band where `LinkCreep_AtSpecialNode` 0x159dd0 does [M 0x159ed8-0x159eec].
- **No RNG draws added** — the list walk is a pure position/flag read; the Rand count is
  unchanged by construction (the shared Rng is untouched; the diff adds no rng use).

**3. Red on the old code, then green.** The pin `nav::tests::special_node_band_counts_a_peer_on_the_node`
asserts the game law through the real `Route::follow` + `Drone::near_pos_fn`: a peer ON the node
counts the band node as reached (and only that tick — the special is consumed), a peer 0.5 away
(outside r = 0.4 [M 0x159eb8]) does not, a peer 2.5 above (outside dy 2.0 [M 0x143fd0]) does not,
the walker's own record is skipped [M 0x143f48], and Bond within 0.55 reaches it with no peers
[M 0x143ec4-0x143f20].

- RED proof on the old tree: with the drone.rs/nav.rs changes stashed and the same test body
  driven by the OLD call-site law (the player-tier-only closure the old sites passed), the
  peer-on-the-node assertion fails (`assertion left == right failed` — reached None vs Some(1)).
  The old behavior is also on record in the replaced drone.rs comment itself ("the list tier
  stays unwired here").
- GREEN on the new tree: `cargo test -p nf-game --lib special_node_band` — 1 passed.

## What ran

- `cargo test -p nf-game --lib --offline` — **777 passed, 6 failed, 4 ignored**; the 6
  (`mp::bots::fell_out_*` ×2, `mp::match_end_pins::*` ×3, `world::tests::flight_into_the_02_shallows_*`)
  were re-run on the pristine master tree (changes stashed) and fail identically there —
  pre-existing on master, not from this branch. `special_node_band_counts_a_peer_on_the_node`
  is among the green.
- `guard_stuck_sweep_live` (nf-view, live, SWEEP_LEVELS=07000014) — 2 passed; the sweep still
  logs its one HIT (below, "What is left").
- `zz_diag_corner.rs` was a throwaway diagnostic (deleted before commit; it never carried
  assertions, only eprintln logs).
- The full suite: started in the background on the final tree
  (`NF_ISO=... nohup python3 tools/test/suite.py > suite.out 2>&1 &`) — see the Suite section.

## What is left

1. **The 07000014 leaf-face freeze is NOT this fix.** The alarm-only sweep's remaining HIT is
   drone#0 (skin 0x05000090, GO_TO_GOAL 0x63, goal (56.06, 0.01, 42.91)) frozen at
   (50.38, 1.00, 52.34), `route_blocked false`, creep 3/15, `creep.special None` — the tier-2
   band never engages (1.16 from its special node; no drone within 0.4 of it; the door node
   isn't the creep's current node). Measured on the port: the raw step decays to pure east
   (0.040, 0.000) and `probe_move` rejects it every tick at the closed 0x1A leaf's west face
   (normal (−1,0,0) at both probe heights); `door_leaf_slide` correctly returns None (the
   tangent is zero). The game's order — move, then the contact pass's DeltaP push-out
   [M 0x140968:204-208, `Collide_Intersect` 0x1ebaa8:353-373] — is the remaining divergence
   candidate, as the sweep test's own comment says. Landing it is a collision-pass reorder that
   touches every level's walk battery; it did not fit this window and was not attempted.
2. **A real-play convergence pin for the tier-2.** The two alarmed 07000014 guards never share a
   node in the alarm-only sweep (drone#1 paces 3.5-4 u off in combat steps), so the red→green
   demo runs through the unit pin, not a live walk. A live pin needs the main agent's PCSX2
   (below) to measure where two route walkers actually queue in the band during real play.

## Live checks for the main agent

1. The 0x1A leaf corner, 07000014, guards alarmed (the sweep scenario): watch
   drone#0's `obj+0x30` (the position) and the door object's live matrix while the guard presses
   the leaf. Expected if the DeltaP-order reading [M 0x140968:204-208] is right: the game's
   guard does NOT freeze — its position jitters east/west by the per-tick DeltaP while its
   along-leaf drift accumulates, and it rounds the leaf end within ~10 s. Access: read watch,
   60 Hz, ~60 s from the alarm.
2. `HITTEST+0x10` (the DeltaP, quad) for that guard's capsule during the press: expected
   non-zero west-pointing x while the contact list (`obj+0xD0`) is non-empty, and the y/z
   components carrying the along-leaf drift. Access: read watch on the HITTEST the guard's
   0x800 capsule fills.
3. `NDrone2_DroneNearPos` 0x143e68's `$f12` at its 0x159ef8 call site during any mission-route
   walk approaching a door node: expected 0.4 (0x3ecccccd) every time — confirms the pinned
   radius on hardware. Access: breakpoint read, any 07000014 patrol.

## Suite

`tools/test/suite.py` was started in the background on the final tree, and the sandbox's disk
killed it twice — reported verbatim, since this is an environment limit, not a code one:

- First run: `BUILD FAILED:` / `error: linking with 'cc' failed: exit status: 1` /
  `collect2: fatal error: ld terminated with signal 7 [Bus error]` (`interactables_07000007_live`,
  the `NightfirePort` bin) — the rootfs was at 100% (3 MB free of 9.9 G; `target/` alone 6.1 G).
  Pruned ~1.9 G (`~/.cache/ms-playwright`, `~/.cache/puppeteer`, `target/debug`) and retried.
- Retry: the build ground through most of the set (fingerprints valid) and exhausted the disk
  again on nf-view's tail: `could not compile nf-view (test "probe_safe08")`,
  `(test "upward_shot_0700000c_live")`, `(test "g049_blast_light_live")` — same linker bus
  error at 100%. A third attempt needs a bigger sandbox, not a code change.
- The binaries that DID link with the new code were run directly (`--test-threads=1`):
  **walkreal_07000014_live 7 passed / 0 failed / 3 ignored** (92.75 s), **fix_npc_doors_live
  1/0** (12.8 s), **fix_guard_first_shot_live 1/0** (1.3 s), **fix_guard_weapons_live 3/0**
  (100.4 s). The drone.rs-touched guards/doors coverage that the rule names
  (`npc_doors_live`, `fix_guard_misc_live`) never reached the link stage before the disk died —
  listed here as not run, with every binary that did run green.
- nf-game's lib tests (run before the suite attempts): 777 passed / 6 failed / 4 ignored, the 6
  verified pre-existing on pristine master (see "What ran").

## ntfy failed

All three fallbacks in order, with their exact output:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "guard-stuck-14-corner: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'guard-stuck-14-corner: done …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "guard-stuck-14-corner: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, the call was killed at the 30 s tool timeout (no connection established).

## Usage

not visible to me
