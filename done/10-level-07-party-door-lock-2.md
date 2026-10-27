## Your task: `level-07-party-door-lock-2`, land the 07 party-door lock tests

Start from `origin/s/level-07-party-door-lock` (merge it first; its report has the chain: door #251 starts open on ch 130, script 0x06000073 stream 8 writes ch 130 <- 0 at the control scene, the door closes and refuses E). Three things block the merge:

1. `party_door_lock_07000007_live` asserts `!channels.on(200)` as permanent game truth ("no writer" is a [D] scan result, not a proof). Keep the lock assertions that come from [M] (the door refuses E after the scene, the free door #244 still opens); turn the ch-200 "never written" claim into a logged observation, or prove it [M — every writer family of a switch channel on this level, including the loadable packs] and cite it.
2. `dump_07000007_doors_and_channel_writers` and `dump_06000073_stream8` are added `#[ignore]`d. Move them to `crates/nf-view/examples/` (probes, not tests) or remove them.
3. Run the tests un-ignored on the tree merged with origin/master; report the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-party-door-lock-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T05:19Z

---
## Report (done, 2026-10-25T07:17Z)

# level-07-party-door-lock-2 — report

Task: land the 07 party-door lock tests. Three things blocked the merge of
`s/level-07-party-door-lock`: (1) `party_door_lock_07000007_live` asserted `!channels.on(200)`
as permanent game truth ("no writer" is a [D] scan result, not a proof); (2) the two
`#[ignore]`d dump tests were tests, not probes; (3) the tests had to run un-ignored on the tree
merged with `origin/master` and the totals reported.

Start tree: `origin/master` `38f1bc93` + `origin/s/level-07-party-door-lock` merged clean
(no conflicts). Branch: `s/level-07-party-door-lock-2`.

## 1. The ch-200 "never written" claim — demoted to a logged observation

`party_door_lock_07000007_live` had two hard assertions on the channel state:

- boot: `assert!(!channels.on(200), "ch 200 is off at boot — the ch-200 doors start locked")`;
- post-scene: `assert!(!channels.on(200), "ch 200 stays off — the lock is permanent on this level [D: no writer]")`.

Both are now `eprintln!` observations (logged each run, kept out of pass/fail), each with a
comment citing the evidence chain and why it is not asserted:

- **The [M] half (writer families)** is first-hand re-verified on this branch: grepping
  `work/decomp/ACTION.ELF` finds **130 `switch_channels[...] =` write sites across 103 files**
  and **no literal `switch_channels[200]` / `[0xc8]` writer anywhere** — every write takes its
  index from a data field (`obj+0x12` Switch_Activate 0x1b02a8 / Switch_Update 0x1adea8,
  `obj+0x20` Trigger_Activate 0x1b2318, Trigger_Update 0x1b1a78, Sensor_Update 0x1af618,
  SoundTrigger_Update 0x1b2790, FuseBox_Update 0x1adfe8, Copter_Update 0x17dbb8,
  Script_EventHandler 0x1d94c0's stream args, ...) except the constant writers (0xfe MP end
  0x184268, 99 mission objectives 0x1827d8, and the fixed ids in
  docs/systems/switch-channel-writers.md — none is 200).
- **The [D] half (this level's records)** — that none of 07000007's placements/streams/packs
  resolves a writer slot to 200 — is exactly what the census/probe dump shows (the moved
  example, below: the script lib's only channel event is `ev0x0d[0x82,0x0]` = ch 130 ← 0; door
  #251's p3 = 200 is the LOCK input, not a writer). But a data resolution can never be an [M]
  proof, so per the driver's ruling the claim stays out of pass/fail.

What remains asserted in the test is only [M]-backed behaviour: door #251's fields
(ch_open 130 [D p2], ch_lock 200 [D p3], created OPEN [M 0x136100:96-99]), the close after the
scene ([M Door_Update 0x1363c8's channel arm]), the E refusal on the locked door
([M Door_Activate 0x135be0 LAB_00135cf8]), the lock field staying armed
([M 0x1363c8:106 clears it only on the unlock edge]), and the free hall door #244 still
answering the use key after the story point ([M Door_Activate — no lock field]).

## 2. The two `#[ignore]`d dumps moved to `crates/nf-view/examples/`

- `crates/nf-view/tests/census_07000007.rs` **deleted** (it held only the two dumps).
- `crates/nf-view/examples/dump_07000007_doors_and_channel_writers.rs` — the doors/triggers/SPs
  census + every script stream's channel-write events. Content unchanged; `#[test] fn dump_...`
  became `fn main`; run with `cargo run -p nf-view --release --example
  dump_07000007_doors_and_channel_writers` instead of `--ignored --nocapture`. Verified by
  running it: prints the 15 doors (DOOR #251 p2=130 p3=200 …), and the script-lib scan
  (`script 06000073 stream 8 (keys 499..498): ev0x0d[0x82,0x0]` — the level's only script
  channel write; lib size 8).
- `crates/nf-view/examples/dump_06000073_stream8.rs` — the raw stream bytes of the
  control-taking cut-scene script. Verified by running it: stream 8's byte dump shows the
  `0d 82 00` triple (event 0x0D, ch 0x82=130, value 0).
- Both examples compile under `cargo build -p nf-view --examples` (the suite's `cargo test`
  builds examples too, so a broken example would have failed the suite build — it doesn't).

## 3. The tests un-ignored on the tree merged with origin/master

Ran directly (release, `--test-threads=1`, NF_ISO on the onfoot image):

| binary | result |
|---|---|
| `party_door_lock_07000007_live` | **2 passed, 0 failed** (8.00 s) |
| `walkreal_07000007_live` | **2 passed, 0 failed** (10.63 s) |
| `interact_07000007_live` | **7 passed, 0 failed** (7.38 s) |
| `probe_07_door_lock` | **1 passed, 0 failed** (1.25 s) |
| `probe_07_route` | **1 passed, 0 failed** (3.78 s) |

**13 passed, 0 failed** across the five binaries the branch touches; the census binary is gone
from the test set (the dumps are examples now — the suite's binary count drops by one).

## 4. The fast suite on the final tree

`tools/test/suite.py` on this branch's tip — the tree merged with `origin/master`, product code
identical to it (the branch changes only nf-view tests/examples and docs). Started 05:40 UTC via
a double-fork daemon (a plain `nohup ... &` was killed between tool calls, and the first daemon
died on a missing `cargo` PATH; the working launcher exports env.sh's PATH/PKG_CONFIG_PATH/NF_ISO
before the double fork).

**Harvest at the session deadline (07:16 UTC): 610 of 791 binaries run — zero FAILED/TIMEOUT/CRASH
lines.** `tail -1 suite.out` is empty (the total line prints only at completion; the deadline hit
first — the same partial shape as the previous task's run). The branch-touched binaries the run
order reached are all green in-suite: `party_door_lock_07000007_live` (2 passed, 0 failed,
8.13 s), `probe_07_door_lock` (1/1, 3.85 s), `probe_07_route` (1/1, 3.85 s),
`interact_07000007_live` (7/7, 6.71 s). `walkreal_07000007_live` (a w-tail binary the partial run
did not reach) was verified directly on this same tree instead: 2 passed, 0 failed (§3). The
sandbox's known environmental red families (the 3 `disc_golden`, `identity_disc`, the GPU-render
trio, the net_*/mp_* OOM SIGKILLs) all live in the unreached tail — none appears in the 610.

## What's left

- Nothing blocking from the driver's three points: all three are done. The only open cosmetic
  item: the [D] no-writer resolution for ch 200 could be strengthened by a full
  writer-slot-by-writer-slot resolution table for 07000007 (every door p2/p3, every SP param 6,
  every trigger out, every pickup param 3, every script event arg) — the examples now dump all
  the raw material for it, but nobody has assembled the table.

## Live checks for the main agent

None required. Everything asserted is [D] disc bytes / [M] decompile addresses read directly
(carried over from `docs/reports/level-07-party-door-lock.md`, unchanged by this task). The
optional fresh-boot watch from that report still stands: `switch_channels+130`
(0x2a38d2+130) 1 → 0 at the control-taking scene's final tick, door #251 swinging shut and
refusing E afterwards.

## Usage

not visible to me — this session's tooling shows me no token, context-window or cost figures
(no input/output token counter, no context-used or cost display anywhere I can read).
