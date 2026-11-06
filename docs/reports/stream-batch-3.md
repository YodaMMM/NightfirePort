# stream-batch-3 — the 28 reds: reproduced, attributed, two levers tried

Branch `s/stream-batch-3` cut from `origin/s/stream-batch-2` df294b666; `origin/master` a6665d115
merged clean (WORKLOG entry pushed as c08c942d9 before any other work). Task: the joined tree's 28
reds (driver's line on stream-batch-2 + a6665d115: **4405 passed / 28 failed / 353 ignored**) —
each treated as a possible real regression until shown otherwise, cause named with a citation
before touching any test; then the walks.

## First, the regression question the task asks

**The merge itself cannot have caused any of the 28 named reds.** `git diff df294b666..9faf17a46`
(the whole tree delta the master merge added) is 3 files / +63: `crates/nf-game/src/player.rs`
(the new `void_fall` flag: set while airborne below y −10 [M `Player_CollisionHandler`
0x1949a8:97-108, disasm 0x194b64/0x194c14-0x194c50] + its unit test), `crates/nf-view/src/scene.rs`
(+10: the flag is CONSUMED only when `self.level == 0x0700_0012`, latching ch 0x7C) and
`crates/nf-view/tests/use_key_census_live.rs` (+5). On every other level the flag is computed and
`take()`n with zero effect, so the merged tree is sim-identical to stream-batch-2 for all 28 —
the reds are the stream-batch-2 tree's own, inventoried in its report. The one walk that DOES
cross the delta (see 12 below) passes.

## The non-walk reds, each reproduced this window (--test-threads=1, real ISO)

| binary (whole) | result | cause, with the citation |
| --- | --- | --- |
| hold_npcs_live | 1/2 | `unheld_guards_move` moved 0.003 over the full 600-tick window (assert hold_npcs_live.rs:57). The trench guard #6 parks at his goal — the guard-radii arrival latch (the radii tasks' measured [D]/[L] arrival behaviour), not a wander-pause: 600 ticks was the old escape. Harness premise stale; needs the wander-vs-goal ledger before any change. |
| goggles_draws_live | 4/5 | **Run with --test-threads=1 as instructed — 4 of 5 pass.** `a_captains_laser_ray` fails at :31 (mode stuck 0 after 450 presses). The in-file 30 u back-off lever (added for muzzle-repin-5) is already in the tree and no longer suffices; the press = the goggles WEAPON 0x5d request [M player.rs BL+0x961/+0x60], eaten while the start room's fight staggers Bond (health forced 100 each tick — the swallow is the stagger, not death). Same live-room fire-line premise class; a searched clear spot is the next lever. |
| guard_drops_flashbang_live (nf-game) | 5/6 | `a_flashed_guard_killed_after_drops_nothing_more` :511 — pickups [(22,12),(6,6)] vs 1. The (22,12) is pl 3's flash-dropped SG5 (the assert's own expected flash drop); the extra (6,6) is the RECOVERED-SIDEARM drop [M 0x163350 case 1 ChangeWeapon(6)] — the doc comment itself calls it the game's own object. One yard guard's stun expires during the frag fuse, he draws the sidearm, dies, drops. LEVER TRIED AND REFUTED: a searched blast offset over 12 spots (±4 u around the [S] +1.0x spot) — dx ≤2.0 all give the death-added (6,6), dx ≥2.5 leaves pl 3 un-killed (recovered, state 0x45, out of reach), dx −2 never flashes. The "kills while stunned" premise is unrealizable on this stream at any nearby spot; reverted to the parent code. Next lever is the flash→frag gap itself, or a live re-measure of the stun clock. |
| smg_close_range_live | 0/1 | The 6 u pistol leg (panic :654); the 1.0/2.0 u legs land. The cone draw is unconditional [M Bullet_init 0x12c5a0:171-180] and the in-file comment already names the mechanism: which round lands depends on where the shared stream sits (voice-trigger draws moved it). Drifted guard positions vs the fixed firing line; not a law break (the SMG asserts and the dormant-pair assert are in the passing legs' path). |
| enemy_accuracy_live | 4/5 (4i) | 5 cells now out of tolerance (was 2 in batch-2): 0x12 d1@5 20 vs 10±2, d2@5 25 vs 16±3, d2@10 18 vs 23±4, d3@20 24 vs 18±3, 0x0e d1@5 0 vs 3±2. Mixed directions (up AND down) — a stream-position shift, not a monotonic rate move; the sister LAW tests all pass (8 pellets, ±3σ rifle/chain equality, astronauts, shin-height hits). The batch-2 deferral stands: the per-lever bisect (which merged stream consumed the draws) before any re-pin. |
| upward_shot_0700000c_live | 0/2 | Both stands complete the real-input walk (legs 1-6) but land nothing: hp 10→10 (hip :104), states [(10.0,false)] (aimed :136). The in-file comment pre-documents it: the guarded [M] law is the CONE (+0x24 = 100.0 [L ee_start_05.bin @ 0x2c10b8]); the deterministic 15-draw realization moved on the joined streams. Needs the PCSX2 live re-check the file asks for, not a sim change. |
| start_corridor_guard_07000011_live | 7/8 | **REAL-REGRESSION SUSPECT.** d2: the guards deliver exactly 6 hits × 12 (health parks at 28.0 — the assert comment's own predicted number) and then hold `lost_sight=true, seen_ticks=0` for 14000+ ticks (a 20000-tick probe) at an IDLE VISIBLE Bond standing at the spawn (state 0x5c, opp_target = the spawn) — the combat sight-loss re-acquire chain stalls, so the death never comes. d1/d3 legs and all 7 sister pins (existence/pairing, GoToGoal t1, corridor box t200/t700, arrival switch, step-aim+fire) pass. Not a timing re-pin: the guards STOPPED firing. Needs the sight-loss re-see decomp review. The probe window was restored to 10000; the diag lines + the finding are kept in the test's comments (424aac6c1). |
| fix_pickups_ammo_live | 3/4 | The take HAPPENS and credits correctly (t1 = Ammo{6,20}, carried 61, type-2 untouched — every assert before the sound one passes) and the box's placement-sound field is Some(0xf5) (:97 passes), but the post-take sounds window is EMPTY (no new sounds at all since `sounds_before`, :112). The sister rocket take plays 0xF5 green in the same binary, so the take→0xF5 law [M 0x18e650:290-300] holds; the difference is this box's route (level placement + intro skip vs the synthetic rocket). Needs the box-take sound ledger (which site plays for kind-1 placements, and the drained-pool interaction) before any change. |

Not re-run this window (the two longest walks, mechanisms already characterized on this same tree
by the batch-2 runs, and the merged tree is sim-identical per the diff above): `mayhew_hold_live`
("the corridor plate never raised ch 0x33" — the escort's plate-crossing route moved with the
radii, batch-2's ledger) and `fix_npc_doors_live` ("door #2 opened with Bond too close (2.2)" —
the walk's own 2.2 arrive-radius E-press beats the escort's open; the hold-Bond-back lever is
named in batch-2's report). Both are first up next window.

## The walks (plain first, whole binaries, --test-threads=1)

- **Every plain walk passes**: 01, 06, 08, 0b, 0c, 11, 12 (d3 plain), 13, 14, 15, 16 (finale walk
  + its real_damage), 0d (green on the batch-2 run; its real_damage ran this window — see below).
- **The real-health d2/d3 walk family dies as its own queue tasks expect** (50-real-health-*):
  01 agent t5664 (shotgun #6 at 2.9 u), 06 agent t10422, 08 00agent t32731 (world −62.5,
  live-wire/hurt-volume route hit), 0b 00agent t15998 (laser pair), 11 00agent (×2 — the file's
  double-#[test] fn) + agent, 12 agent — a 6000-tick WEDGE at the north lobby 2 with NOTHING
  nearby (no door/drone/trigger, grounded, d=4.47 short; the level-12 invisible-wall wedge
  family), 15 agent t7435, 16 agent/00agent ignored with measured ledgers (in-file).
- 0d real_damage: the ~9.5 m fall check health 18.99 (pin >40, :836). The game fall law holds —
  9.1 m ≈ 24 damage [M 0x1949a8:117-126], and 18.99 + ~24 = ~43 pre-fall health: the walk ARRIVED
  at the fall with ~43 because the upstream real-damage fights bled 64.4 (ledger: fists 32.0,
  sentry 16.0, rifle 2.4). Damage-budget premise shift; the real-health-0d task owns it.
- 14 real_damage: reproduced EXACTLY on batch-2's numbers (death t5063 at (31.73,1.76,33.83),
  the drone-#1 1.0-hit bleed at d 4.2) — unchanged, inherited.
- **0c real_damage: NEW red** — dies t8216 (previously ended alive at 26.2 hp per the file doc)
  to long-range rifle plinks (#31/#25 0x12 at 47/61 u, 0.6 per hit). The file's own agent-ignore
  note documents the same crossfire-cadence class on d2; it now reaches the d1 leg.
- **13 real_damage: NEW failure mode — SUSPECT.** Death t4169 at y −368.30: a VOID FALL inside
  the warm chain (11→12→4A→13), where the file previously ended alive at 8.5 hp. The chain
  crosses 12 — the one level where master a6665d115's plunge-channel latch now fires below −10 —
  but a fall to −368 means Bond went past the handler either way. Needs the fall-site attribution
  (which chain level, and whether ch 0x7C fired) before blame: this is the only red that touches
  the merge's behavior change.

## Two levers tried and what they proved

1. guard_drops blast-spot search — REFUTED and reverted (the ledger is in the run log: recovery
   dominates the frag fuse at every lethal spot). The parent test code is restored byte-for-byte.
2. start_corridor window probe to 20000 — the probe produced the real finding (the sight
   re-acquire stall), the window was restored to 10000, the diag lines and the finding stay in
   the comments (commit 424aac6c1). No pin was loosened: the death REQUIREMENT and the per-hit
   [L] pins are untouched and the test is still red, now with the stall documented.

## No-no check

No [L] pin touched, no ignore added, no game code changed (the only game-file delta on this
branch is master's own void_fall merge). No test was made to pass by weakening it; the two
harness edits above are diagnostics, one reverted entirely.

## What's left

1. The start_corridor d2 sight re-acquire stall — the decomp review (highest value).
2. The 13-chain void-fall attribution (which level, did 0x7C fire).
3. mayhew_hold + fix_npc_doors re-runs and their named levers; the fix_pickups box-take sound
   ledger; the enemy_accuracy per-lever bisect; the guard_drops flash→frag gap lever.
4. The real-health agent/00agent walk families — owned by their own queue tasks.

## Live checks for the main agent

1. 07000011 d2, idle Bond at the spawn: watch the two start-corridor guards past 10 s of game
   time — the port fires 6 shots (72 dmg) then goes blind (lost_sight latch, no re-see). Does
   the real game's d2 pair keep firing/re-acquiring there? (Addresses of interest: the sight
   re-acquire path in the Drone2 control STANDARD chain; the port's fields are lost_sight /
   seen_ticks on the drone.)
2. upward_shot_0700000c: the file's own live re-check — the same hip-pull stand at the deck
   sentry in PCSX2 (15 shots, cone +0x24 = 100.0 [L 0x2c10b8]); does a round land on the real
   machine's stream?
3. The 13 warm chain: on the real game, does the 11→12→4A→13 chain walk survive the shaft
   transit on real health (the port now void-falls to y −368 at t4169)?

## Usage

Token/credit/context figures: not visible to me. (No token, cost or context-window display is
available in this sandbox; nothing to copy.)

## ntfy failed

All three fallbacks failed (2026-11-06 ~17:22-17:24 UTC; GitHub push from the same sandbox worked
seconds earlier, so the sandbox had network):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "stream-batch-3: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → exit 28, `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'...'), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "stream-batch-3: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, killed at the 40 s tool timeout (still hanging).
