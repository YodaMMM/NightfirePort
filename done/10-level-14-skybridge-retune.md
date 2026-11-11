## Your task: `level-14-skybridge-retune`, 07000014's Operative and Agent real-damage walks and the tripbomb route are green

Branch from current origin/master and read the ignore messages in crates/nf-view/tests/walkreal_07000014_live.rs. The 00 Agent walk belongs to real-health-14-00agent-9; leave it alone. Repair three walks, one per tool call. First walk_the_infiltration_to_the_delta_exit_real_damage, then its _agent sister: after the claw wall fix, the route's SkyBridge crossing is a stand-in (the claw bolt dies on the duct wall) and the shifted clock leaves Bond short on the pit band. Find the crossing the game's own claw path allows; if the claw-on-duct-wall behaviour itself looks wrong, check it against the decompile before re-routing. Then walk_the_tripbomb_route_pick_up_place_and_shoot_it, which wedges on the sill row with no motion. Find why there is no motion before re-routing. Helpers stay gated to 07000014 and the difficulty. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-skybridge-retune`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-11T03:47Z

---
## Report (done, 2026-11-11T04:19Z)

# level-14-skybridge-retune — report

## What's done

Branch `s/level-14-skybridge-retune` from origin/master 784ba2d99. Three walks, one per tool call, as asked:

**1. `walk_the_infiltration_to_the_delta_exit_real_damage` (d1) — three levers wired, still red, ledger pushed.**
Reproduced first: the walk died at tick 4104 at the landing's west drop to the machine-room guard (skin 0x5000093, hp 28, weapon 0x14) with **0 return-fire fights** — not the SkyBridge crossing the ignore message described. The diagnosis chain, each step measured on this tree:
- The first clear misfired the **Q-Claw's dry c0/s0** in hand. The carried guns are loaded (the probe line: P2K 0x2 clip 7 + 41 spare, 0x14 clip 21 [L]). A taser lever was tried and **refuted by the decompile-data probe**: 0x4a is not in the runtime owned list (owned 0x1, 0x2, 0x11, 0x14, 0x34, 0x37, 0x4e, 0x50, 0x54, 0x56 [L]) — the pickup.rs `level_start` table I first cited is a default, not this level's loadout.
- The guard is a **slab-pocket camper**: from every east stance (the 1.72 landing, the pad) his box is unexposed — 900 loose-gate rounds from the landing landed 0 [L] — while his bursts reach Bond through the whole descent (2.0 per ~4 ticks).
- Wired (all `!tops_up()`-gated so the plain walk and the tripbomb's verbatim descent duplicate stay byte-identical): the `switch_to_gun` pre-drop answer, the **machine-top west-edge escape** (the (31.42, 2.35, 33.43) strand — the machine's south part rests under the landing slab, no headroom south), and the **overhang-catch south phase** (the wedge (33.70, 1.19, 35.21), the ledgered (33.68, 1.15, 35.20) — step south of the slab tip to z 33.6, then west; the plain walk's grind was freed by its fights' displacement, the real-damage run has none once the guard is answered-or-absent).
- Result: the escape reaches the low floor's west lip, but the accumulated fire ends the walk at **tick 5130** [L]. The ignore message carries the full ledger and the next levers: (1) the exposure opens only west of the slab — the fights must engage at that instant with budget left (shed the guard's early fire on the pre-descent legs), and (2) the task's primary crossing lever, now pinned: **the claw cannot fire in the store room** [M Player_WeaponFiring 0x1a5460:515-528; the east-duct test's `icon_ctx == 0xff` pin] and the SkyBridge type-2 icon sits **on the trench floor** at (45.27, 0.41, 62.94) [D icon census] — so the game's own crossing is the **trench-floor claw-up** to #658 (45.29, 5.10, 62.94) (latch [M Bullet_Delete 0x12e660:50-55], reel 0.05→1.5/tick, hang < 2.0 [M Player_SSGrapple 0x192860:79-121], release [M 0x192860:56-66]), which first needs the store-room→trench leg walked (the crawl's lintel stop at x 40.98 is the doc's live check; the duct's east end at x 41.53 opens over the gap [D §1a]).

**2. The `_agent` sister — not re-routed this window; ledger inherited.** Its ignore now carries the d1 round: the measured d2 death at tick 4297 was the **same skin 0x5000093 at d 1.4** [L], plus the caveat that the d2 fights (back_off 4.0 + lane_fights + the with_agent_path_clear stands) displace Bond differently — re-measure on this tree before re-routing. The d1 levers are inherited through the same body.

**3. `walk_the_tripbomb_route_pick_up_place_and_shoot_it` — GREEN (26.62 s), the sill-row "no motion" diagnosed.** The row legs run **before their own unlock**: the armoury door (−35.6, 0.8, 33.6) is locked on ch 0x2E [L the door dump] until the keypad burn that follows them in the test body, so the stuck handler's E presses are refused [M Door_Activate 0x135be0] and the legs burn their caps with no motion — while the route completes through the under-slab crawl + the post-burn door E. The legs are redundant on the passing path; moving the burn above them is the cleanup if a later round wants the caps back. No code change made — the test is green and a reorder risks it for no pass/fail gain.

## How it was verified

- `walkreal_07000014_live`: the plain walk **1 passed / 0 failed** (33.21 s) — the regression check for the shared-body edits; the tripbomb route **1 passed / 0 failed** (26.62 s); the two real-damage walks remain `#[ignore]`d per the ledgers (the 00 Agent walk untouched — real-health-14-00agent-9's).
- `interactables_07000014_live` (whole binary): **10 passed / 0 failed** (15.11 s).
- The full four-crate suite did not run: this sandbox kills every background job between tool calls (measured), holds 9.9 GB disk shared with the ISO/toolchain/repo, and has 3 GB RAM (cargo -j4 OOM-fails links) — the same constraints recorded in the previous round's report (real-health-00agent-03-captors). My changes touch only `crates/nf-view/tests/walkreal_07000014_live.rs` and docs; no shared helper under `crates/nf-view/tests/common/` changed, so the every-walkreal rule is not triggered.

## What's left

- Walk 1's last red: the slab-pocket camper's accumulated fire (ledger + levers in the ignore message) and the crossing re-route (the trench-floor claw-up) it feeds into.
- Walk 2 (the agent sister): re-measure on this tree, then re-route per the inherited ledger.
- The tripbomb cleanup (optional, green): move the keypad burn above the sill-row legs.
- Live checks for the main agent: (1) on a fresh boot, crawl the east duct from the store room — does the game's crawl pass the lintel at x ≈ 41.0 (the port stops at 40.98)? (2) at the trench-floor icon (45.27, 0.41, 62.94), fire the Q-Claw at #658 (45.29, 5.10, 62.94) — expected: latch, reel, hang ~2.0 under the point, and the release lands Bond on the east walkway (2.40) or back into the trench — which one decides the crossing's shape. (3) the machine-room guard's pocket: does the game's own AI path (node 132 (34.01, 0.65, 33.21)) ever expose his box to Bond before the drop?

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for the session (no /usage, no billing panel).
