# real-health-00agent-03-captors-4 — report

Task: continue the 07000003 (Countdown 15) 00 Agent real-damage walk
(`walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent`,
ignored). The walk is NOT green; per the task's "otherwise push the refreshed ledger and the
next lever into it", the ignore message on `s/real-health-00agent-03-captors-4` (commit
76e4d602a) now carries this round's full ledger and the next levers. The walk stays ignored.

## What was done

- Branch setup: created `s/real-health-00agent-03-captors-4` from origin/master 385ebb66d and
  merged `origin/s/real-health-00agent-03-captors-3` (a283cb626) in first — captors-3 was not
  on master; the merge was clean (walkreal_07000003_live.rs, WORKLOG, the captors-3 report).
- Sandbox reset redone: rustup stable 1.100.0, libasound2 1.2.14 user-prefix (trixie's deb,
  extracted under /home/z/my-project/alsa-prefix per the -3 report's recipe), work-slim + the
  ISO downloaded and extracted (369823203 bytes verified), cargo fetch.
- Measured this round (runs 18-21, all on real input from the level load, no top-ups, no
  damage/health writes; helpers gated to 07000003 + difficulty >= 3; scratch evidence
  /tmp/d3run18.out - d3run21.out, not committed):
  - run 18 (10463, REGRESSION): the -3 ledger's lever 1 as written — a resupply from the lane
    node (the walked-path corridor reaches (14.2,-28.0) d 7.8) + `switch_to_armed` + the
    re-take — walked Bond INTO the uncleared lane cluster with the fights on: drone #10
    (skin 0x5000039 weapon 0x12) reached d 2.1 and dealt 44.8 in 18 hits. The sprint shape is
    wrong BEFORE the stands, not after.
  - run 19 (10764): the same sprint COLD (return_fire off, the crossing-legs doctrine). The
    walk survived to the pickup, but the corridor's only pickup is an EMPTY gun (no ammo gain;
    run 13's `cycling weapons (0) (weapon 0x1c clip 0 spare 0)` is the same box) and attempt
    1's second target (14.5,-29.0) sits behind the (14.0,-30.3) door — the cold walk ate 25
    hits. Lever 1 pivoted: the kill-loot cycle the file already runs on the servants' ground
    (stands -> loot_the_drops_03 -> re-stand) replaced the resupply sprint.
  - run 20 (80288, the round's best, +5x over the -3 ledger's 15555-15776): the lane crossed
    at 41.2 hp with the re-armed clear engaging #6 (no 40-class round landed after the lane),
    the servants' phase reached — and the end was the 0x23 plate's trio wake MID-mouth-nav:
    the si-1 saddle nav (from the spot the failed "before the servants' rescue (agent)"
    resupply dragged Bond to, the moat west end (-10.26,-37.82)) detoured WEST through the
    whole moat band and wedged 7x3000 on the lane doors, so the mouth nav rode the saddle zone
    at (25.30,7.37,8.79) with the plate firing mid-climb; trio shotgun #5 (skin 0x500003a
    weapon 0x1c) charged to d 2.5 for 13.8 in 8, and #7 (skin 0x5000039 weapon 0x12,
    (11.4,10.1,-16.1), hp 10) chipped 27.4 in 43.
  - run 21 (74135): the saddle via the mouth-path anchor (22.1,-2.0). The anchor nav itself
    wedged 20x3000 (60k ticks wasted); the saddle nav then worked (the legs crossed the lane
    doors and climbed: (11.42,-20.71)->(14.3,-18.57)->(19.31,-7.5)->(23.16,7.16)) and arrived
    (23.18,5.50,7.20) — but Bond was DRY (clip 0 spare 0, the wheel cycled to the fists) and
    the saddle sweep stood no chance: #7 landed 41.2 in 51 hip taps. Died at the saddle
    doorstep.
- Wired and kept (all d>=3-gated, the d<3 streams untouched — the whole-binary run below
  proves it): the lane-north kill-loot cycle (clear -> loot -> re-clear) and the si-1
  saddle-wake block (walk the east-lawn saddle before the door-8 cluster block; the aimed
  sweep, sniper-first, wait 600). The resupply-sprint shape is measured and removed.

## Where the walk stands

80288 this round vs 88424 (run 12, the -3 round's best, an uncommitted intermediate tree) and
15555-15776 (the -3 round's committed shape). The lane-north coin flip vs #6 is gone (the
kill-loot cycle re-arms the stands; the re-armed clear engaged #6 and no 40-class round landed
after the lane in run 20). The two open ends are AMMO (runs 20/21 both starve before the
saddle — the lane legs' clears and the fights burn the clip, and the servants'-phase resupply
drags Bond 25.7 u WEST to the moat west end for nothing: "resupply done (false) clip 3 spare
13" [L d3run20]) and #7 (skin 0x5000039, upper floor (11.4,10.1,-16.1)): 43-51 hip hits for
27.4-41.2 across runs 20/21, alive at hp 4-10 at both deaths. The ignore message carries the
next levers in order: (1) arm the walk to the saddle (leg loot + the west-dragging resupply
gated to difficulty == 2 only); (2) the saddle route without the (22.1,-2.0) anchor (it
wedges; the direct saddle nav from the east-lane legs' end works); (3) the #7 aimed stand
once armed (a live check for the main agent: whether #7's window opens from the saddle
(23.5,6.3,8.4) at d ~26 or from the mouth (35.6,6.4,3.4)); (4) the trio wake's first volley
(sweep from the climb's midpoint if the next run still eats a point-blank volley
mid-approach); (5) the older levers stand (the north-of-door-8 approach (36.2,5.7), the
87000-tick (35.6,8.7) door stall, the exit-lawn #16 post-kill volley).

## Verification (what ran, with pass counts)

- `walkreal_07000003_live` (the task's named binary, run WHOLE on the lever tree):
  **6 passed / 0 failed / 2 ignored** (305.66 s; the 2 ignored = the d3 real-damage walk + the
  door-8 load probe). Every non-ignored stream (d0-d2 walks + the real-damage d2) stays green
  with this round's d>=3-gated changes. The only tree change after that run is the #[ignore]
  string itself (the ledger refresh), which alters no stream's behaviour.
- `interactables_07000003_live` (the task's other named binary, run WHOLE):
  **9 passed / 0 failed** (43.20 s).
- The ignored d3 walk itself: runs 18-21 above (direct, `--ignored --nocapture`).
- The FULL suite did NOT run in this sandbox, measured reasons:
  - The background start (`nohup python3 tools/test/suite.py > suite.out 2>&1 &`) died between
    tool calls — suite.out was 0 bytes and no suite/cargo/rustc process survived the next
    call (the sandbox kills background jobs; the -3 report hit the same wall).
  - The foreground full build (`cargo test --offline --release -j2 --no-run -p` x4) was
    attempted 4x: two kills by the tool timeout, then the sandbox DISK FILLED (100%, 0 free)
    twice. Autopsy: `target/release/build/nf-view/` held 4.8 G of stale per-unit hash
    directories accumulated across the killed invocations (each cargo run creates new hash
    dirs and cargo never prunes them; `target/release/deps/` was deleted mid-fail), 5.5 G
    total on a 9.9 G disk with the ISO. `cargo clean` + `rm -rf target` freed it; the two
    named binaries then built in 4 m 22 s. No test FAILED in any run I made; the run phase of
    the suite never started. This is sandbox/toolchain state, not a code result.
- Diff scope check: this session changed ONLY
  `crates/nf-view/tests/walkreal_07000003_live.rs` (+ docs). The helpers it defines are
  file-local; no other test binary exercises the changed code, so the two named binaries plus
  the direct d3 runs cover every test that uses what was touched.

## Live checks for the main agent

- #7's lean windows: drone 0x5000039 sits at (11.4,10.1,-16.1) on the lane's upper floor and
  outlives every phase (43-51 hip hits absorbed per run). Does his box pass the eye-ray set
  from the saddle (23.5,6.3,8.4) (d ~26) or from the door-8 mouth (35.6,6.4,3.4)? If yes, the
  armed saddle stand (lever 3) kills the endgame's top drainer; if no, the mouth stand is his
  exposure spot.
- Whether the 0x23 plate fires when Bond crosses the saddle climb from the mouth-path side
  (run 20 measured it firing mid-climb at (25.30,7.37,8.79)) — and how long the trio's first
  volley takes to land after the wake (the sweep's settle+raise is ~330 ticks; if the volley
  beats it, the sweep must start from the climb's midpoint, lever 4).

## What's left

- The d3 walk itself (next levers 1-2 above; the refreshed ledger in the ignore message is
  the hand-off).
- The suite's run phase on this sandbox (the disk/background notes above; a fresh sandbox
  should start the suite build FIRST in the foreground and never kill it mid-invocation —
  every killed cargo invocation leaves ~4-20 MB of stale hash dirs under
  target/release/build/nf-view and the disk cannot afford them).

## Usage

not visible to me

## ntfy failed

All three fallbacks failed (2026-11-14 10:05-10:06 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'...', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 35 s with no output (killed, WGET-EXIT=0 from the timeout wrapper)

The queue entry `done/50-real-health-00agent-03-captors-4.md` (with this report attached,
commit e31a25b89) was pushed to `origin/queue` successfully, so the driver's queue watcher
is the delivery path for this result.
