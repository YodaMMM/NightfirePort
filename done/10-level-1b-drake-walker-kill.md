## Your task: `level-1b-drake-walker-kill`, Drake killed by the player's own fire starts the finale like the game

`crates/nf-view/tests/interactables_0700001b_live.rs` (~line 211) is #[ignore]d: when Bond kills Drake through the Walker (`w.shoot`: exposed_aim box-centre aims plus the reload/strafe inputs), the death channel 0x0e never rises and the NIS chain stays dead. The same kill through raw `step_bond` (`probe1b_blockers::probe_drake_death`) spawns him enabled, dies at t81 into AstronautDeath 0xbf and raises ch 0x0e. Find what differs between the two kill paths (the hit data, the damage route, the death state entered, or a Walker input that interrupts the death) by tracing Drake's state and the ch-0x0e writer on both [M: cite the death handler that writes 0x0e]. Fix the port (not the test) so a real-input kill raises 0x0e and the finale chain runs, then un-ignore the test. Keep the teleport approach to the arena as it is (documented, the west boundary is sealed). Run the crates you touch and the 1b tests.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/1b-drake-walker-kill`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-31T08:59Z

---
## Report (done, 2026-10-31T10:32Z)


---
## Report body (the finish.sh cat missed it — absolute path retry, 2026-10-31T10:34Z)

# level-1b-drake-walker-kill — report

Branch `s/level-1b-drake-walker-kill` (from origin/master b44ac857). Note: the task body's last
line says branch `s/1b-drake-walker-kill`; the message header and the rules both say
`s/level-1b-drake-walker-kill`, so that is the branch I created and pushed (the body line reads
as a typo).

## What the blocker actually was

The task asked: find what differs between the walker-driven Drake kill (ch 0x0e never rises,
NIS chain dead — the 2026-10-23 ignore note) and the raw `step_bond` kill
(probe1b_blockers::probe_drake_death — dies into AstronautDeath 0xbf, raises ch 0x0e).

**Nothing differs any more.** On the current master the walker kill reaches the identical death:

- `NF_KILL_TRACE` (added to `Walker::shoot`, env-gated — the file's NF_HEALTH_TRACE pattern)
  through the real test's own flow: kill at +130, hp 20→0, st 0xc0→**0xbf** (AstronautDeath),
  corpse same tick, **ch 0x0e ON at the kill**, mode 0x1d throughout, hit parts 4/53/4 (the
  trace lines are in the WORKLOG entry). A verbatim copy of `shoot`'s loop in a scratch probe
  confirmed the same and watched the whole chain: ch 0x0b at NIS-t600, MissionState 6 at
  NIS-t720, `to_end = 0x0730001b`. The 2026-10-23 note predates later 1b merges that repaired
  the death route; its premise is stale.

**The real failure was the test's own final assert.** The test asserted
`channels.on(0x0e)` AFTER the defeat NIS played out — but the game itself clears ch 0x0e there:

- SP #198 is global placement #198 of 0700001b: params `(0, 0x06000a20) (2, 64) (3, 0)
  (4, 0) (5, 1) (6, 14)` — **stay (param 4) = 0, mask = 1, out = 0x0e** [D, dumped from the disc
  bytes with the nf_formats placement parser]. NIS 0x06000a20 is 720 ticks, 5 streams [D].
- `SP_Update` 0x1dbda8's tail writes `switch_channels[out] = cVar10` when the trigger lands;
  `cVar10` is 1 on the case-1 start and **0 on the case-3 re-arm** [M 0x1dbda8 case 3
  `cVar10 = '\0'` + the tail write]. With stay = 0 [D], the player does NOT delete when the
  720-tick script runs out — it reaches case 2's end arm → state 3, and the next trigger
  (ch 0x0e still on) fires case 3: the re-arm restarts the script and **writes ch 0x0e = 0**
  [M]. ch 0x0e rises with the kill (AstronautDeath's ENTER → `DroneFunc_SetDeathChannel`
  0x13e660 [M 0x1747b8]), drives the NIS, and is cleared by the game's own re-arm one tick
  past the NIS's end. The port implements exactly this (`crates/nf-game/src/nis.rs` state-3
  arm + the `if self.out != 0 && trig { ch.set_script(self.out, value) }` tail).

So the end-of-chain `channels.on(0x0e)` assertion contradicted the decompile: per the rules the
TEST was wrong, and it was fixed with the addresses — **the port was not changed** (no
nf-game/nf-formats/nf-audio code touched; the only source-side edit is the additive env-gated
kill-trace in the test harness).

## The test fix (`interactables_0700001b_live::drake_defeat_by_real_fire_ends_the_mission`)

- Un-ignored; the stale note replaced by the lifecycle (above) in the doc comment.
- The ch-0x0e raise is now asserted AT the kill, right after `w.shoot` returns [D param 12 = 14;
  M 0x1747b8 → 0x13e660].
- The post-kill wait no longer breaks early on status 6: the stay-0 re-arm's clear tick must
  land inside the window (NIS 720 ticks [D] + re-arm + the SP's own replay).
- End-of-chain asserts: ch 0x0e **off** (the case-3 write [M 0x1dbda8]), ch 0x0b on [D t600],
  `ended` (MissionState 6 [M 0x182c60]), `to_end == 0x0730001b` [M 0x182c60:114-116].
- The teleport approach to the arena is untouched, as required (documented stand-in; the west
  boundary is sealed).

## How it was verified

- `interactables_0700001b_live`: **7/7 pass** (the un-ignored test run twice — deterministic).
- The full 1b set on the final tree: `fix_finale_live` 4/4, `probe1b_blockers` 5/5,
  `probe1b_drake` 1/1, `walkreal_0700001b_live` 9/9 (+1 pre-existing ignore), `probe1b_route`
  3/3, `probe1b_rover` 1/1, `interactables_0700001b_probe` 1/1,
  `interactables_0700001b_console_probe` 5/5; `probe1b_c2writers`/`probe1b_float`/
  `probe1b_nis211` all-ignored as before.
- Full suite (fast runner, foreground chunks — this sandbox kills background jobs between tool
  calls, so `tools/test/suite.py` could not survive; the resumable foreground runner from the
  earlier sessions ran the same binaries with a 240 s per-binary limit): **340/983 binaries in
  the window — 1021 passed, 6 failed, 61 ignored**. The 6 failures are the 0700000c pylon view
  pair (`fix_0c_pylon_live`, panics at its line 19 spot assert), `campaign_flow_live`
  (TIMEOUT at the 240 s limit), and `disc_golden` ×3 — all three present in the previous
  session's baseline (`/home/z/NightfirePort/suite1.out`, 06:26 UTC, same failure set on a
  master without my change). My change touches only 1b test files; none of the 340 reached
  binaries regressed. **What's left: the remaining ~640 suite binaries** (alphabetically past
  `disc_golden`) — the next message can resume `/home/z/fg_runner1b.py` (state
  `/home/z/suite1b_state.json`) which re-runs only binaries whose exe changed.

## What's left

- The suite tail above (breadth insurance only — the change is test-side and the touched crate
  slices are green).
- Optional live check (below) to upgrade the case-3 conclusion from [M]+[D] to [L].

## Live checks for the main agent

1. Fresh boot 0700001b (a save-state resume skips the boot path), fight Drake and kill him,
   then `gdbwatch` `switch_channels[14]` (ch 0x0e) through the defeat NIS: expected ON at the
   death tick, staying ON through the 720-tick NIS (0x06000a20), and **clearing one tick after
   the script's last tick** (SP#198 stay=0 → `SP_Update` 0x1dbda8 case 3's re-arm write), while
   ch 0x0b (t600) and ch 100 (t720) stay latched. This confirms the [M 0x1dbda8]+[D stay=0]
   reading that the old test assert (ch 0x0e on at the end) contradicted.

## Usage

not visible to me — this environment exposes no token, context-window or cost figures for the
session.
