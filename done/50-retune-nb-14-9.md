## Your task: `retune-nb-14-9`, 14 infiltration to the delta exit 00 Agent walk green again (round 9)

Branch from current origin/master; if origin/s/retune-nb-14-8 is not on master, merge it in first. Read docs/reports/retune-nb-14-8.md there and the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Keep the Operative sister green. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-14-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-20T11:22Z

---
## Report (done, 2026-11-20T12:40Z)

Closed by the driver: the report is on `s/retune-nb-14-9` (docs/reports/retune-nb-14-9.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-20T12:41Z)


---

# retune-nb-14-9 — the 14 infiltration to the delta exit 00 Agent walk, round 9

Branch `s/retune-nb-14-9`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-8`
merged in first (it was not on master; the merge was clean — the round-4..8 reports, the
ledgers and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was fresh this round: the full setup was redone from the task message (clone,
work-slim.7z, the ISO archive verified at 369823203 bytes before extraction, rustup stable
1.100, cargo fetch). Two environment notes for the next round: this sandbox runs as a
non-root user (no sudo, no /root), and the fast disk for many-small-file writes is the home
volume — the PolarFS mount under /tmp/my-project is ~340x slower for them (the work-slim
extract took 1.8 s locally vs >600 s there). The ALSA link needed three user-prefix fixes
(no root this time): `PKG_CONFIG_ALLOW_SYSTEM_LIBS=1`, the user `alsa.pc`'s prefix rewritten
to the prefix path, and `RUSTFLAGS="-L native=<prefix>/usr/lib/x86_64-linux-gnu"`. The fast
suite was started in the background twice in the first minutes per the task order; on this
box background processes are killed between tool calls (the same failure mode rounds 4-8
recorded — suite.out never appeared).

## What's done

**The walk is still `#[ignore]`d — still red — but the death moved t25389 → t36527 (+11000
ticks over the round-8 bank), the pocket squad is DEAD, the escape touch #456 was REACHED,
and the walk loads the wake legs with the grenade launcher ARMED (c15+s59) — the deepest
this walk has ever been, and all three round-8 levers are now measured.**

- **Lever 1 (the Kiko-corridor armour ledge approach) is MEASURED DEAD, read-only** — the
  round-8 ignore message ordered the probe first, and the probe answered without spending a
  walk: the NF_SRV_PROBE extension (env-gated, byte-identical when unset, `quick` mode
  exits before the walk) dumped `nav_route` from the east lane's end (−44.5,0.9,34.0) to the
  armour (−41.29,1.53,36.48) = **NO ROUTE**, and from the armour to the escape touch =
  **NO ROUTE** — the y 1.53 ledge sits off the nav web in both directions, so the take is a
  blind walk, which the round-8 ledger itself forbids. The shipped lane-end→touch route was
  also dumped for comparison: 3 nodes at y 0.40, z 30.7/28.9/27.8 — the pocket convergence
  the sisters cross fine. The v5/v6 deaths rode the *fetch wander's* route to this armour,
  not this shipped route.
- **Lever 3 (the dry-gun law) is MEASURED REAL and SHIPPED** as the opt-in `no_lane_fetch`
  law on `DamageLog` (crates/nf-view/tests/common/damage_log.rs): with the flag on, the
  fights' own dry/low resupply fetches stay off — a dry gun bails the fight and the leg
  walks on. Set only by this walk's round_dodge stream at the lane crossing, cleared after
  the touch cut-scene; every other walk byte-identical by construction. The v1 run fixed the
  scope: the fetch fires INSIDE the lane crossing leg (the walk_to's mid-leg return-fire
  gate → the LOW gate → the fetch at t27840, its nav walk south through the pocket
  convergence into the kind-52 world explosion at (−40.6,0.0,24.2) r 8.0 — death t28518,
  the v6 tick exactly); the v2 run (scope at the lane crossing) removed the wander, the
  pocket swing and the wire death entirely.
- **Lever 2 landed as the Kiko-corridor fast taps REMOVED** (judged on the v2 trace): four
  300-tick stands left #20 hp 10 UNCHANGED every time (the round-5 zero-of-18-taps
  measurement re-confirmed — they never killed anything), "out of ammo at +164" burned the
  0x1 from 15+61 to 0+19 (the dry-gun cause west of the door), and the stands sat Bond in
  the squad's 0x14 GL line that drained the 35-bank to 0 before the lane. Removing them
  banked BOTH the clips and the armour budget for the pocket approach.
- **The v3 run is the FINAL TREE**: the pocket pass carried — the fights answered with the
  preserved clips and killed #29, #22, #31, #61 AND #30 (all killed true, the round-8
  ledger's main shooters); the escape touch #456 was reached (arrived (−34.40,1.07,26.83));
  the wake-legs loadout line reads in-hand 0x14 clip 15 spare 59. The new death is INSIDE
  the wake legs at their node 1 (−34.18,1.13,21.37): #62 (skin 0x5000090, 0x14, hp 10 FULL,
  d 3.9) dealt 28.0 in 11 hits unanswered while the fight machinery exchanged with #43/#31
  (the determined pass on #31 cost −21.20; #43's stand burned ~700 ticks without a kill).
  The node-1 ambush is the landing-5 note's own answer point ("#43 answered before the round
  only from node 1") that the shipped code covers only at node 2/21.
- The ignore message is refreshed with the round-9 ledger and the next levers in order:
  (1) the wake-legs node-1 ambush law — an answer point before node 1's leg, or the loadout
  law (the owned 0x11 reads c2 dmg 60.0 in the loadout dump — two 60-dmg rounds kill every
  hp-10 post the GL taps four times for; marked [S] unverified, the 0x11's id and round
  flight are next round's first check); (2) the door-face bank DRAIN (still untried: the
  mute legs + scans drained the 35-bank to 0 by the lane with health 87.5 CONSTANT — a
  shorter stand shape, e.g. the west-face scan's failed #32 300-tick fight the corridor-east
  leg's own return fire killed anyway); (3) the lane-crossing wedge (−47.0,32.4) d 3.0 —
  the blind walk_to burns the 6000-tick cap every run since the fetch removal;
  walk_nav_to the same target (the web routes it — the probe's own fetch walks proved it).

## How it was verified

- **walkreal_07000014_live (whole binary, the final tree, --test-threads=1)** [L 2026-11-20]:
  **9 passed / 0 failed / 1 ignored, 173.50 s** — the ignored one is the 00agent sister
  (its refreshed ledger in the ignore message). Includes the un-ignored Operative d2 walk,
  the plain/tops-up walk, the d1 real-damage walk, the vent-drop, grill, E-cascade and
  tripbomb tests — the sisters stay green with the `no_lane_fetch` law and the taps removal
  on the tree (both laws are d3-gated; the whole-binary run proves the gates hold).
- **interactables_07000014_live (whole binary)** [L 2026-11-20]: **10 passed / 0 failed,
  15.27 s** — matches rounds 5-8's counts.
- Every lever was judged on its own real run (v1 the fetch scope, v2 the no-fetch stream,
  v3 the taps removal), each with NF_DMG_TRACE ledgers captured (probe1.out, v1.log, v2.log,
  v3.log, whole_binary.log, interact.log — untracked scratch). The probe verdict was taken
  read-only before any leg was spent (the round-8 ignore message's own discipline).
- **The fast suite (`tools/test/suite.py`)**: started in the background twice in the first
  minutes as ordered; both starts were killed before suite.out appeared (the same
  background-kills-between-tool-calls failure mode rounds 4-8 recorded). The foreground
  four-crate build exceeds the window on this 2-core box; the two required binaries' whole
  runs above are the evidence. NO total line this round, same as rounds 4-8.
- Only `crates/nf-view/tests/walkreal_07000014_live.rs` and
  `crates/nf-view/tests/common/damage_log.rs` changed. damage_log.rs is a shared test
  helper: the change is the opt-in `no_lane_fetch` field + two `&& !self.dmg.no_lane_fetch`
  gates on the fights' own resupply calls — off by default, set only by this test's
  round_dodge stream, and the whole-binary run above proves every other stream in this
  binary is green. No file under `crates/*/src/` changed.
- Facts tagged [L] come from this window's runs on the real ISO with NF_DMG_TRACE /
  NF_SRV_PROBE; code citations are [M] with the decomp addresses carried in the test
  comments (Player_HandlePain 0x1902c8:84-104; DroneWeap_DoBulletAccuracy 0x179740:113-115);
  the 0x11 loadout item is marked [S] unverified in the ignore message; no test was
  weakened; the game bytes, the token and the URLs stayed out of the commits.

## Still red / what's left (the round-10 levers, in the ignore message too)

1. **The wake-legs node-1 ambush law** — the walk dies to #62's unanswered GL line at the
   wake legs' node 1; an answer point before the leg (the landing-5 note's own law), or the
   loadout law (the 0x11's two 60-dmg rounds) — the loadout dump is in the trace.
2. **The door-face bank drain** — the mute legs + scans still drain the 35-bank to 0 by the
   lane (health 87.5 constant); a shorter stand shape banks it for the pocket + wake legs.
3. **The lane-crossing wedge** — the blind walk_to burns the 6000-tick cap every run since
   the fetch removal; walk_nav_to threads it.

## Live checks for the main agent

- The wake-legs node-1 area: is #62's post (−30.4,1.0,22.1) in gun-ray sight from node 0
  (−30.84,1.03,24.32)? If yes, a scan stop there (the delta_approach_scan shape, 300 cap)
  answers him before his first round; the landing-5 note measured #43's first round only
  answered from node 1 — the same shape.
- The 0x11 weapon id: confirm from the disc tables it is the two-round 60-dmg gun the
  loadout dump reads (c2 dmg 60.0), and that its round flight is not the slow-dart family
  (the fights' stand law fires it like a gun).

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost
figures for the session; nothing to copy.

---

# retune-nb-14-9 — the 14 infiltration to the delta exit 00 Agent walk, round 9

Branch `s/retune-nb-14-9`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-8`
merged in first (it was not on master; the merge was clean — the round-4..8 reports, the
ledgers and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was fresh this round: the full setup was redone from the task message (clone,
work-slim.7z, the ISO archive verified at 369823203 bytes before extraction, rustup stable
1.100, cargo fetch). Two environment notes for the next round: this sandbox runs as a
non-root user (no sudo, no /root), and the fast disk for many-small-file writes is the home
volume — the PolarFS mount under /tmp/my-project is ~340x slower for them (the work-slim
extract took 1.8 s locally vs >600 s there). The ALSA link needed three user-prefix fixes
(no root this time): `PKG_CONFIG_ALLOW_SYSTEM_LIBS=1`, the user `alsa.pc`'s prefix rewritten
to the prefix path, and `RUSTFLAGS="-L native=<prefix>/usr/lib/x86_64-linux-gnu"`. The fast
suite was started in the background twice in the first minutes per the task order; on this
box background processes are killed between tool calls (the same failure mode rounds 4-8
recorded — suite.out never appeared).

## What's done

**The walk is still `#[ignore]`d — still red — but the death moved t25389 → t36527 (+11000
ticks over the round-8 bank), the pocket squad is DEAD, the escape touch #456 was REACHED,
and the walk loads the wake legs with the grenade launcher ARMED (c15+s59) — the deepest
this walk has ever been, and all three round-8 levers are now measured.**

- **Lever 1 (the Kiko-corridor armour ledge approach) is MEASURED DEAD, read-only** — the
  round-8 ignore message ordered the probe first, and the probe answered without spending a
  walk: the NF_SRV_PROBE extension (env-gated, byte-identical when unset, `quick` mode
  exits before the walk) dumped `nav_route` from the east lane's end (−44.5,0.9,34.0) to the
  armour (−41.29,1.53,36.48) = **NO ROUTE**, and from the armour to the escape touch =
  **NO ROUTE** — the y 1.53 ledge sits off the nav web in both directions, so the take is a
  blind walk, which the round-8 ledger itself forbids. The shipped lane-end→touch route was
  also dumped for comparison: 3 nodes at y 0.40, z 30.7/28.9/27.8 — the pocket convergence
  the sisters cross fine. The v5/v6 deaths rode the *fetch wander's* route to this armour,
  not this shipped route.
- **Lever 3 (the dry-gun law) is MEASURED REAL and SHIPPED** as the opt-in `no_lane_fetch`
  law on `DamageLog` (crates/nf-view/tests/common/damage_log.rs): with the flag on, the
  fights' own dry/low resupply fetches stay off — a dry gun bails the fight and the leg
  walks on. Set only by this walk's round_dodge stream at the lane crossing, cleared after
  the touch cut-scene; every other walk byte-identical by construction. The v1 run fixed the
  scope: the fetch fires INSIDE the lane crossing leg (the walk_to's mid-leg return-fire
  gate → the LOW gate → the fetch at t27840, its nav walk south through the pocket
  convergence into the kind-52 world explosion at (−40.6,0.0,24.2) r 8.0 — death t28518,
  the v6 tick exactly); the v2 run (scope at the lane crossing) removed the wander, the
  pocket swing and the wire death entirely.
- **Lever 2 landed as the Kiko-corridor fast taps REMOVED** (judged on the v2 trace): four
  300-tick stands left #20 hp 10 UNCHANGED every time (the round-5 zero-of-18-taps
  measurement re-confirmed — they never killed anything), "out of ammo at +164" burned the
  0x1 from 15+61 to 0+19 (the dry-gun cause west of the door), and the stands sat Bond in
  the squad's 0x14 GL line that drained the 35-bank to 0 before the lane. Removing them
  banked BOTH the clips and the armour budget for the pocket approach.
- **The v3 run is the FINAL TREE**: the pocket pass carried — the fights answered with the
  preserved clips and killed #29, #22, #31, #61 AND #30 (all killed true, the round-8
  ledger's main shooters); the escape touch #456 was reached (arrived (−34.40,1.07,26.83));
  the wake-legs loadout line reads in-hand 0x14 clip 15 spare 59. The new death is INSIDE
  the wake legs at their node 1 (−34.18,1.13,21.37): #62 (skin 0x5000090, 0x14, hp 10 FULL,
  d 3.9) dealt 28.0 in 11 hits unanswered while the fight machinery exchanged with #43/#31
  (the determined pass on #31 cost −21.20; #43's stand burned ~700 ticks without a kill).
  The node-1 ambush is the landing-5 note's own answer point ("#43 answered before the round
  only from node 1") that the shipped code covers only at node 2/21.
- **A fourth run (v4) measured the round-10 loadout-law variant NET-NEGATIVE and removed it**:
  switch_to the owned 0x11 crossbow (c2 dmg 60.0 in the loadout dump [M player.rs:2696
  "crossbow 0x11"]) before the wake legs — the 90-tick real-input switch stand never reached a
  fight; it re-rolled the deep stream (wake start health 41.7 vs the v3 stream's 57.7; the wake
  route recomputed; a node-0 resupply fetched Bond west into #30's line, death t36285 vs the v3
  tree's t36527). The switch stand IS the change — the tree keeps the v3 shape, and the ignore
  message records the design question (a switch that rides INSIDE a leg's own walk).
- The ignore message is refreshed with the round-9 ledger and the next levers in order:
  (1) the wake-legs node-1 ambush law — an answer point before node 1's leg, or the loadout
  law (the owned 0x11 reads c2 dmg 60.0 in the loadout dump — two 60-dmg rounds kill every
  hp-10 post the GL taps four times for; marked [S] unverified, the 0x11's id and round
  flight are next round's first check); (2) the door-face bank DRAIN (still untried: the
  mute legs + scans drained the 35-bank to 0 by the lane with health 87.5 CONSTANT — a
  shorter stand shape, e.g. the west-face scan's failed #32 300-tick fight the corridor-east
  leg's own return fire killed anyway); (3) the lane-crossing wedge (−47.0,32.4) d 3.0 —
  the blind walk_to burns the 6000-tick cap every run since the fetch removal;
  walk_nav_to the same target (the web routes it — the probe's own fetch walks proved it).

## How it was verified

- **walkreal_07000014_live (whole binary, the final tree, --test-threads=1)** [L 2026-11-20]:
  **9 passed / 0 failed / 1 ignored, 172.73 s** (the v3 tree ran 173.50 s with the same
  counts before the v4 revert; the final tree's only diffs from it are comments) — the ignored one is the 00agent sister
  (its refreshed ledger in the ignore message). Includes the un-ignored Operative d2 walk,
  the plain/tops-up walk, the d1 real-damage walk, the vent-drop, grill, E-cascade and
  tripbomb tests — the sisters stay green with the `no_lane_fetch` law and the taps removal
  on the tree (both laws are d3-gated; the whole-binary run proves the gates hold).
- **interactables_07000014_live (whole binary)** [L 2026-11-20]: **10 passed / 0 failed,
  15.27 s** — matches rounds 5-8's counts.
- Every lever was judged on its own real run (v1 the fetch scope, v2 the no-fetch stream,
  v3 the taps removal), each with NF_DMG_TRACE ledgers captured (probe1.out, v1.log, v2.log,
  v3.log, whole_binary.log, interact.log — untracked scratch). The probe verdict was taken
  read-only before any leg was spent (the round-8 ignore message's own discipline).
- **The fast suite (`tools/test/suite.py`)**: started in the background twice in the first
  minutes as ordered; both starts were killed before suite.out appeared (the same
  background-kills-between-tool-calls failure mode rounds 4-8 recorded). The foreground
  four-crate build exceeds the window on this 2-core box; the two required binaries' whole
  runs above are the evidence. NO total line this round, same as rounds 4-8.
- Only `crates/nf-view/tests/walkreal_07000014_live.rs` and
  `crates/nf-view/tests/common/damage_log.rs` changed. damage_log.rs is a shared test
  helper: the change is the opt-in `no_lane_fetch` field + two `&& !self.dmg.no_lane_fetch`
  gates on the fights' own resupply calls — off by default, set only by this test's
  round_dodge stream, and the whole-binary run above proves every other stream in this
  binary is green. No file under `crates/*/src/` changed.
- Facts tagged [L] come from this window's runs on the real ISO with NF_DMG_TRACE /
  NF_SRV_PROBE; code citations are [M] with the decomp addresses carried in the test
  comments (Player_HandlePain 0x1902c8:84-104; DroneWeap_DoBulletAccuracy 0x179740:113-115);
  the 0x11 loadout item is marked [S] unverified in the ignore message; no test was
  weakened; the game bytes, the token and the URLs stayed out of the commits.

## Still red / what's left (the round-10 levers, in the ignore message too)

1. **The wake-legs node-1 ambush law** — the walk dies to #62's unanswered GL line at the
   wake legs' node 1; an answer point before the leg (the landing-5 note's own law). The
   loadout-law variant (the 0x11 crossbow switch) is MEASURED NET-NEGATIVE (v4) and removed
   — a standing switch re-rolls the deep stream; the switch must ride inside a leg's walk.
2. **The door-face bank drain** — the mute legs + scans still drain the 35-bank to 0 by the
   lane (health 87.5 constant); a shorter stand shape banks it for the pocket + wake legs.
3. **The lane-crossing wedge** — the blind walk_to burns the 6000-tick cap every run since
   the fetch removal; walk_nav_to threads it.

## Live checks for the main agent

- The wake-legs node-1 area: is #62's post (−30.4,1.0,22.1) in gun-ray sight from node 0
  (−30.84,1.03,24.32)? If yes, a scan stop there (the delta_approach_scan shape, 300 cap)
  answers him before his first round; the landing-5 note measured #43's first round only
  answered from node 1 — the same shape.
- The 0x11 weapon id: confirm from the disc tables it is the two-round 60-dmg gun the
  loadout dump reads (c2 dmg 60.0), and that its round flight is not the slow-dart family
  (the fights' stand law fires it like a gun).

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost
figures for the session; nothing to copy.

## ntfy failed

All three fallbacks failed, in order, at ~12:42 UTC 2026-11-20 (the same failure mode rounds
5-8 recorded — the DNS resolves, the TLS connect to ntfy.sh hangs):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-14-9: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'retune-nb-14-9: …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → hung silently, killed by the 25 s wrapper with no output (exit 124).
3. `wget -q -O /tmp/w/ntfy_out.txt --post-data "retune-nb-14-9: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → hung with no response written; the 25 s wrapper kill hit before wget's own timeout
   produced output (exit 124). /tmp/w/ntfy_out.txt existed but was empty (0 bytes).
