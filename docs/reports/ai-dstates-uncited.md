# ai-dstates-uncited — the guard AI states the port never names

Task: for the 16 ACTION.ELF functions never mentioned in `crates/*/src` (15 `NDrone2_DSTATE_*`
+ `NDrone2_initDTYPE_SniperAlert` 0x14e580): find each one's state id in `NDrone2_StateFuncs`
0x29b300 and who enters it, then classify (a) already implemented under another name / (b)
reachable on-foot and missing / (c) never entered on the shipped on-foot path.

## Result

**`NDrone2_initDTYPE_SniperAlert` 0x14e580 — (a) already implemented; citation + new test added.**
Its `DroneTypeSettings` row 25 (mode 0x19) is REACHED at creation on the shipped on-foot path:
43 placements across 0700004a (35), 0700000c (4), 07000016 (3), 07000006 (1) carry a mode-2
behaviour block with alert level 3, so `NDrone2_GetDTYPENEW` 0x14c938:105-110 lands
`+0xC5 = 0x19` and `GetDroneTypeAttackTypeFriend` 0x14cbd8:118-121 calls the init fn [M + D
census]. The port already runs it: `type_setup`'s name-keyed dispatch (drone.rs:2052) matches
the row-25 init name and sets `stationary` (`+0x4F8 & 0x10`) — which is the function's only
observable effect: the disassembly shows the `Drone_DCVfromOBJ` 0x13a0a0 call fills a stack
DCVars that is never read, and the sole drone store is `sw v1,0x4f8(s0)` after `ori v1,v1,0x10`
[M disasm 0x14e580; 0x13a0a0 only reads the obj]. Function name and address now cited at that
spot. New test `crates/nf-game/tests/dtype_sniper_alert_live.rs` (green): 0700004a's garrison
spawns with mode 0x19, `stationary`, start 0x2a (row 25 start [L 0x29aee8+300]); the same
level's dtype-3 abseil pair (mode 0x1c) negative-checks that the mapping only fires on mode-2
blocks.

**The 15 DSTATE handlers — (c) never entered on the shipped on-foot path; proof cited as a
block in `crates/nf-game/src/drone.rs`'s state module** (after the last constants): Interogator
0x3b/0x1729f0, Alerted1stEncounter 0x57/0x173178, CombatTooClose 0x5d/0x173338, Prone
0x6b/0x166940, ProneFire 0x6c/0x166a28, HoldItRightThere 0x80/0x173570, SeenOpponent
0x9d/0x173b38, SeenExplosive 0xa1/0x173bb8, FailMission 0xb5/0x173f68, Tester1 0xb7/0x16db90,
Tester2 0xb8/0x174150, Tester3 0xb9/0x1741c8, Tester4 0xba/0x1742d8, HangUp 0xbb/0x1743d8,
WaitForever 0xbc/0x174430. Per id: table slot verified from the EE snapshots [L — byte-identical
across ee_snapshot/ee_start_05/ee_start_07000001/ee_start_0700000a], body summarized [M], and
the entry-channel census over all 3039 decomp files: `Drone_SM_SetState` 0x1758c0 literals (319
sites, hex+decimal), `+0x5A2/+0x5A4/+0x5A6` stores, the msg-carried id of DSTATE_Disabled
0x172440:45 (`*(msg+0xcd8)`; drone-side stores {0,1,0x56,0x24,0x25,0x26,0xff}), the anim-end
channel (CallAnim arg4 → +0x596 → SetEndAIState), and the DroneTypeSettings start/alt columns.
Nothing produces these ids except the closed tester cluster (Tester2/4 → 0xb7; Tester3's
end-state → 0xba) whose members are themselves producer-less (Tester3 has zero inbound edges),
and the DTYPE rows 8 (0xb7) / 11-alt (0xb5) whose modes 8 and 0xb no `+0xC5` writer can emit
(GetDTYPENEW's return domain + the writer census; all placements are NEW/param 5 = 100 [D
census]).

## Doc corrections

`docs/port-map/unreached.md`: Tester4's row said "never assigned" — wrong, Tester3's msg-0xc arm
programs end-AI-state 0xba [M 0x1741c8:26]; row corrected (still transitively unreached).
Tester1's row flagged `DroneTypeSettings[8] = 0xb7` as "[D census not run; [S]]" — the census is
now closed (mode 8 unreachable) and the row says so. ProneFire's production was understated
(CallAnim follow AND HandleImpact forwarding, both from Prone) — noted in the drone.rs block.

## Calltree

`python3 tools/calltree/calltree.py work/decomp/ACTION.ELF work/elf/ACTION.ELF . 0x1729f0
0x173178 0x173338 0x166940 0x166a28 0x173570 0x173b38 0x173bb8 0x173f68 0x16db90 0x174150
0x1741c8 0x1742d8 0x1743d8 0x174430 0x14e580` → **"TOTAL distinct never-mentioned reachable: 0"**
(every reachable callee of all 16 is named or addressed in crates/*/src).

## What ran

- Full test-binary build of the four crates on the final tree: `cargo test --offline --release
  --no-run -p nf-game -p nf-formats -p nf-audio -p nf-view` — EXIT 0, 0 errors, 789 binaries.
- Focused runner chunk (the fast runner in this sandbox: `python3 tools/test/suite.py . --only
  drone_portcheck_live,dmode_row0_near_player_live,ai_missing_states_live,dtype_sniper_alert_live,
  alert_status_live,drone_weapon_drop_live`): **19 passed, 0 failed, 0 ignored**.
- `--test dtype_sniper_alert_live` alone: 1 passed (the new test above).

**The full fast suite did not complete in this sandbox.** Background processes are killed
between tool calls here (the runner died three times — once on a missing `cargo` PATH, once at
the alsa-sys build, once mid-run after printing "build ok: 788 test binaries"), and foreground
calls cap at 10 minutes on a 3 GB / 2-core box, so the 788-binary run cannot fit. The suite
reached "build ok" (i.e. every test binary of the final tree compiles) and the drone-critical
binaries all pass; no failure of any kind was observed in anything that ran. Residual risk is
the un-run remainder of the suite, not a known failure.

## What's left

- The optional live check (would close the static census's only caveat): gdbwatch over the 16
  addresses during a full PCSX2 run — DSTATE_Disabled's msg-carried id channel is statically
  unboundable, so only a live run can prove no shipped script commands one of these ids.
- The friend-switch trio's firing (mode 2 → 0x19 without the row-25 init fn) is configured on
  16 placements (07000005's sniper-pair templates among them) but no live capture proves it
  fires; the port models it at the ATTACK-ENTER/ENEMY_MISSION arms (drone.rs), the
  PreDroneControl second-switch path is marked unmodelled — pre-existing, untouched here.

## Usage

Not visible to me. This session exposes no token, credit or context-window figures to the agent
(input/output tokens, context used, cost: none shown). The two research subagents reported their
own totals (156,205 and 230,073 tokens respectively) but the driver session's figures are not
displayed anywhere I can read.
