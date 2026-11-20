# retune-nb-14-10 — the 14 infiltration to the delta exit 00 Agent walk, round 10

Branch `s/retune-nb-14-10`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-9`
merged in first (it was not on master; the merge was clean — the round-4..9 reports, the
ledgers and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was partially reset this round: the repo survived but on a leftover branch with a
dirty tree (2041 files of mode-only changes, discarded), and `work/`, the ISO, rustup and the
cargo cache were gone. The full setup was redone: work-slim.7z (extracted in 5.6 s), the ISO
archive verified at exactly 369823203 bytes before extraction, rustup stable 1.100, cargo
fetch, `p7zip` missing this round so `py7zr` did the two extractions. The ALSA user prefix and
the env.sh exports from the earlier rounds survived and were reused unchanged. The two
required test binaries built from scratch in 4m08s.

## What's done

**The walk is still `#[ignore]`d — still red — but the round-9 lever is answered in two
measured steps, a new law is shipped, and the death moved t36527 → t37477 (+950 ticks, the
deepest this walk has ever been): the node-1 leg ARRIVED for the first time on this stream,
the corner camp #28 is dead before his second round, and the walk now dies at the delta
approach's node 22 to a world blast on a pool the #39/#40 exchanges drained.**

- **Step 1 (v1): the round-9 answer-point scan at node 0 is MEASURED DEAD (no-op)** — the
  `delta_approach_scan` shape inserted after node 0's leg per the round-9 ignore message. The
  scan's own candidate dump answered the round-9 live-check question READ-ONLY: the candidates
  at node 0 are #28 (skin 0x5000087, 0x14, **d 1.3, eye false gun false** — the corner camp
  sits 1.3 u away BEHIND the geometry), #31 (d 11.5, blocked) and #36 (0x1a, d 15.3, blocked)
  — **#62 is NOT present/enabled at node 0** (he spawns later; his first GL hit is t36439).
  The scan consumed ZERO ticks (the pick broke before any stand) and the stream reproduced the
  round-9 v3 death TICK-EXACTLY (t36527, same spot, the same 28.0-in-11-hits #62 race) — the
  scan shape is stream-neutral when nothing is answerable, which is itself the proof the
  sisters keep byte-identity. The scan stays in the tree (free live-check data every run).
- **Step 2 (v2, NF_RF_TRACE=1): the ROOT CAUSE is the `finish_fights` DETERMINED PASS** —
  after the #43 stand broke on the unseen-60 rule (aim None from t60; his t35707 volley rolled
  out of the 240-frame window), the pass (`damage_log.rs`'s `shoot_bounded` 1800 arm) started
  on #43 (hp 6, dist 4.0) and held the machinery **752 ticks with NO visibility break and NO
  target switch** (`walker_b2::shoot_gated` only breaks on corpse/dry/cap) while #62 (0x14,
  fixed post (−30.4,1.0,22.1), hp 10 FULL) raced 28.0 in 11 hits to the death —
  `fighting=true` blocked the return-fire gate the whole stretch. The 88-tick GL race had no
  answer path AT ALL.
- **The shipped lever (round 10): the OPT-IN pass-break in `walker_b2::shoot_gated`** — keyed
  on the `switch_on_first_blood && finish_fights` pair (both set only by THIS file's d1 + d3
  wake-legs blocks; every other stream short-circuits the first condition, zero cost): break
  the pass when the target has dealt NO damage over the 240-frame window (the switch law's own
  window) while ANOTHER shooter HAS, every 10th tick. The leg's own gate re-picks (the unseen
  fallback finds the new shooter) and the broken target takes the wrapper's 240-tick cooldown.
- **The v3 run (the lever's own real run)**: the break fired twice — the #31 pass broke at
  +40 and the gate RE-PICKED #28, **killed true t35326** (the corner camp dead before his
  second round; the round-9 stream ate his −5.00 unanswered); the #43 pass broke at +0 (the
  752-tick sit is gone). The node-1 leg **arrived** (+214 ticks) with health **52.7** vs the
  old stream's 28.0 at the race point (**+24.7 banked**); the wake legs crossed nodes 2-21
  (the node-7 drop fetch ran; the node-21 delta scan killed #51 AND #41 — the shipped
  approach law carrying). The NEW death: node 22 (−13.8,1.0,−24.2) at **health 14.2** — the
  **world blast** (kind 27, (−17.6,1.5,−23.2), d 4.0 = the radius edge, 50.0 nominal → dealt
  14.20, owner −1) finished a pool the delta approach drained: #39 (0x1a, d 23.8, −3.50) +
  #40 (skin 0x50000b2, 0x1a, d 21.2→15.7, −7.00 a hit — the fights fought him TWICE: the
  first stand read killed true at his 0.4 remaining and the pass spent −21.0 to remove it,
  31.5 total for one post, with the fights DRY clip 4 spare 0 and the LOW resupply finding
  nothing) + #41 (−7.0 during the scan fight).
- The ignore message is refreshed with the round-10 ledger and the round-11 levers in order:
  (1) the delta-approach pool bank — the blast dealt exactly the 14.2 remaining, so ~+15 at
  node 22 crosses it: (a) the fights' near-dry law (clip+spare < 5 → switch_to the owned armed
  0x14 GL c15+s59, riding the LEG's resupply gate — the round-9 v4 design question's own
  answer shape), (b) the node-21 scan's fast-tap variant (`shoot_bounded_fast`, the door-face
  8-tick family) so #41 dies before his second volley, (c) the #40 tail law — a pass whose
  target reads hp ≤ 1 at d > 10 while the leg can walk costs more than it removes;
  (2) the world blast's trigger — the sisters cross the SAME ground green: name it (the
  node-21 door E-open chain or a timer); if scripted with a fixed window, the shipped walk
  crosses before it fires and lever 1 alone carries; (3) the round-9 levers 2/3 (the door-face
  bank drain, the lane-crossing wedge `walk_nav_to`) stand untried, lower priority now that
  the death is past the lane.

## How it was verified

- **walkreal_07000014_live (whole binary, final tree, --test-threads=1)** [L 2026-11-20]:
  **9 passed / 0 failed / 1 ignored, 174.47 s** — same counts as round 9. The ignored one is
  the 00agent sister (its refreshed ledger in the ignore message). Includes the d1 real-damage
  walk — which sets `switch_on_first_blood` + `finish_fights` itself and so runs the new
  pass-break law — **green**: the d1 stream carries the law.
- **interactables_07000014_live (whole binary)** [L 2026-11-20]: **10 passed / 0 failed,
  12.89 s** — matches rounds 5-9's counts.
- **The walker_b2 spot-checks** (the helper changed, so other levels' walker_b2 binaries were
  sampled per the merge lessons): **walkreal_07000009_live 9 passed / 0 failed (16.38 s)** and
  **walkreal_0700000b_live 8 passed / 0 failed (63.16 s)** — the flags short-circuit there and
  the streams stay byte-identical. There are 80 `walkreal_*_live` test files; the full sweep
  does not fit the 2-hour window on this box (rounds 4-9 recorded the same constraint) — the
  two 07000014 binaries plus these two spot-checks are this round's evidence.
- Every lever was judged on its own real run: v1 (the node-0 scan — the no-op verdict), v2
  (the same tree + NF_RF_TRACE, a read-only diagnostic env — the mechanism), v3 (the pass-break
  — the +950 verdict). Each with NF_DMG_TRACE ledgers captured (v1.log, v2.log, v3.log,
  whole_binary.log, interact.log, spotcheck.log — untracked scratch).
- **The fast suite (`tools/test/suite.py`)**: started in the background in the first minutes
  as ordered (`NF_ISO=… nohup python3 tools/test/suite.py > suite.out 2>&1 &`); it was killed
  between tool calls before suite.out wrote a line (the same background-kills failure mode
  rounds 4-9 recorded — `ps` confirmed 0 processes). NO total line this round, same as rounds
  4-9; the two required binaries' whole runs above are the evidence.
- Changed files: `crates/nf-view/tests/walkreal_07000014_live.rs` (the node-0 scan insertion +
  the refreshed ignore message) and `crates/nf-view/tests/common/walker_b2.rs` (the opt-in
  pass-break). walker_b2.rs is a shared test helper: the change is gated on the
  `switch_on_first_blood && finish_fights` pair — both flags set only inside this test file
  (the d1 wake legs at line ~2096, the d3 body at lines ~1492/1950) and the code really checks
  the flags; the whole-binary run proves every other stream in this binary is green, and the
  two spot-check binaries prove the other levels untouched. No file under `crates/*/src/`
  changed; the other walks' byte-identity is by construction (flags off → the first condition
  short-circuits) plus the spot-check runs.
- Facts tagged [L] come from this window's runs on the real ISO with NF_DMG_TRACE /
  NF_RF_TRACE; code citations are [M] with the decomp addresses carried in the test comments
  (Player_HandlePain 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88, 113-115; the
  GameState+0x28 → Drones::difficulty assert at load); no test was weakened; the game bytes,
  the token and the URLs stayed out of the commits.

## Still red / what's left (the round-11 levers, in the ignore message too)

1. **The delta-approach pool bank** — the world blast dealt exactly the 14.2 remaining;
   ~+15 health at node 22 crosses it. The measured waste to bank: the #40 double-fight (−31.5
   for his last 0.4 hp with the fights dry), the #41 scan fight (−7.0), #39's opener (−3.50).
2. **The world blast's trigger** (kind 27 at (−17.6,1.5,−23.2)) — scripted window or route
   content; decides whether lever 1 alone carries.
3. **The round-9 levers 2/3** (the door-face bank drain, the lane-crossing wedge
   walk_nav_to) — untried, lower priority now that the death is past the lane.

## Live checks for the main agent

- The world explosion at (−17.6,1.5,−23.2) r 4.0 kind 27 owner −1: which object/SEQ fires it
  and on what trigger (the node-21 door E-open chain, a timer, or a hurt volume on the
  approach)? If scripted with a fixed window, the shipped walk crosses BEFORE it fires and the
  fix is purely the fight-tax bank (lever 1). A gdbwatch pass over that address family during
  a fresh-boot run to node 21-22 names it.
- #62's spawn/enable point: the round-9 question is ANSWERED — he is not present at node 0
  [L the v1 candidate dump]. New: from where does he spawn/enable (the node-1 door channel?),
  and is his muzzle line to the node-1 point (−34.18,1.13,21.37) open or an arc-over-geometry
  lob (his 11 hits all landed from the same fixed post with the walk machinery reading no
  fightable line)?

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost
figures for the session; nothing to copy.

## ntfy failed

All three fallbacks failed, in order, at ~17:00 UTC 2026-11-20 (the same failure mode rounds
5-9 recorded — the DNS resolves, the TLS connect to ntfy.sh hangs):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-14-10: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20003 milliseconds` (exit 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'retune-nb-14-10: …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → hung silently, killed by the 25 s wrapper with no output (exit 124).
3. `wget -q -O- --post-data "retune-nb-14-10: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → hung with no output; the 25 s wrapper kill hit before wget's own timeout produced
   anything (exit 124).
