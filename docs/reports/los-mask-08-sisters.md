Task: los-mask-08-sisters — 07000008 walkable on Agent and 00 Agent with the game's line-of-sight masks.
Branch s/los-mask-08-sisters = origin/s/los-mask-08-walks (aa9326d30) + merge origin/master (784ba2d99). Continued from the walks report's "What's left" items 1-2 (the Agent sister, then the 00 Agent sister) and the two sisters' ignore messages.

## What's done

**The merge is in and the 08 battery is green on it: the whole `walkreal_07000008_live` binary passes 5 / 0 / 1 (the 1 is the 00 Agent sister's ignore).** The conflict resolution was the session's first half:

- `crates/nf-game/src/world.rs`: master's side taken whole. The walks branch's own commits never touched it (verified: `git log 03b86b40c..origin/s/los-mask-08-walks -- crates/nf-game/src/world.rs` is empty) — its `g.pass[k]`-unscoped conditions were the older master state the branch carried, and current master's `(!dyn_set && g.pass[k])` + the `skip_panes`/`pane_flags` refinement supersedes them (the pane comment cites 0x1eae80:76-82 / 0x1ebaa8:93-99 for why a pane's baked pass bytes are not its glist word).
- `crates/nf-game/tests/ai_live.rs`: the walks branch's side kept — it deliberately un-pinned the sniper's second-hit PART (a masked-vision-chain consequence, asserted tick + damage only); master still carried the old hard pin `(130, 32)`.
- `crates/nf-view/tests/walkreal_07000001_live.rs`: master's side kept — the same ch-0xF ambush logic refactored into `run_slab_ambush` (126 lines, same comments, same hurt-of/route-order/walled partition).
- `crates/nf-view/tests/walkreal_07000008_live.rs`: master's d2 hall-mouth pre-engage kept AND the walks branch's station-deck pre-engage, walkway-door unstuck walk and deck-pair pre-engage kept (all fight_pkg/d2+ gated, so the topped-up Operative timeline stays byte-exact — its test passed unchanged); master's missile dodge (the 2-30 u line dodge with the 24-tick hold, d2+/real-damage gated) taken over the walks branch's earlier terminal strafe dodge (the same fight, one dodge system, master's is the later refinement with the d3 ledger; `exit_ticks` stays declared for the log line); master's green state for the d2 Agent test kept (ignore stays removed).
- `crates/nf-view/tests/walkreal_07000011_live.rs`: master's newer ignore ledger taken (text only; no route change).

**The Agent (d2) sister is GREEN on the merged tree — verified, not assumed.** `walk_the_whole_mission_on_real_input_real_damage_agent` passes in the whole-binary run above; master's west-doorway-slit Hind fight + missile dodge survive the merged tree with the walks branch's extra stands layered in (the station-deck stand no-ops behind the hall-mouth stand; the deck-pair stand found no visible guard at tick 2475 and walked on, exactly as the walks ledger predicted). There was no ignore left to remove — master had already un-ignored this test; this session's work was proving it green on the merged tree.

**The 00 Agent (d3) sister stays ignored, with a sharpened ledger and two new next levers.** The merged tree moved the death much earlier than the old solid-body ledger (tick 32622 at the Hind): the walk now dies at tick 9592 at (153.72, 43.10, −120.47) — the walkway/fusebox — 25 hits for 100.0, 10 return-fire fights:

- The east-hall guards' upstream bleed: #11 28.0 in 7 hits, #3 24.0 in 7, #2 20.0 in 6, #14 12.0 in 3 (placements (152.5-161.5, 46.0, −87.7..−101.5)), then the secure-room guard #4 (weapon 0xa) 8.0 in 2 at d 5.2 finishes [L the NF_DMG_TRACE runs on this branch].
- Health is already 8.0 by tick 2475 (the bleed completes during the walkway legs) and still 8.0 at ticks 9152/9345 — the descent stands arrive too late.
- The stands found NO visible guard at every tried spot this session: the station hall mouth (tick 373 — pre-wake: the ch-0x2a plate (139.1, 50.2, −93.6) [D] IS the hall arrival, the wake lands the same tick and the stand walked on), the deck pair (tick 2475), the lower landing (tick 9152 — the walkway's own band blocks the reverse ray from that spot; tried and dropped), the fusebox (tick 9345 — though the guards' own rays reached that spot at d 23-34, the masked sight query answers blocked).
- NEXT LEVERS (in the ignore message): (1) the stands' `|dy| < 4.0` filter inside `fight_guards_within`'s `close()` excludes the east-deck guards from the mouth — Bond y 49.5-50.2 vs the placements' y 46.0 = dy 3.5-4.2 — a d3-only variant with dy 5.0 (or a mouth stand spot at y 47-48) lets the mouth fight them the tick they wake; (2) the wake-tick race — re-run the mouth stand after a bounded wait so it fires post-wake (the first run fired pre-wake and the walkway-head leg then ate the bursts, 4 hits at frames 1330-1683); (3) the Hind's dodge-window widening (30→60 u, the wall-band step) stays open — never reached, the walk dies upstream.

One descent stand shipped in code: the d3-gated `fight_guards_within(&mut w, "the east deck from the fusebox", 4200, true, 34.0)` after the fusebox doorway fight (it found nothing this run — kept because it is the fusebox fight site the hits name, and the next session's dy fix turns the same spot into the kill). No src files changed; everything is in `crates/nf-view/tests/walkreal_07000008_live.rs`.

## How it was verified

- Whole binary `walkreal_07000008_live` (`--test-threads=1`, NF_ISO on the slim on-foot ISO): **5 passed / 0 failed / 1 ignored** in 116.8 s — the topped-up whole-mission walk (byte-exact, passed unchanged), the Operative real-damage walk, the Agent real-damage walk, both ally tests; the 00 Agent sister is the ignore.
- Whole binary `interactables_07000008_live`: **15 passed / 0 failed / 0 ignored** in 15.4 s.
- Runs used real input from the level start, no teleports, no damage/health/ammo writes, no top-ups; every tactic cites its run evidence ([L]) or addresses already pinned in the file.
- Not run this window (the sandbox's 9.9 G disk filled three times — the linker died with bus errors until stale pre-merge binaries, browser caches and rust-docs were cleared; the remaining window went to the named binaries): the other walkreal binaries, `nf-game --lib`, the full fast suite. The merge's test-file resolutions outside 08 (01's helper refactor, 11's ledger text) are master's own proven code carried verbatim. The fast suite should be re-run on this branch by the next session before merge.

## What's left

1. The 00 Agent sister: the two new levers above (the d3-only dy-5.0 mouth variant, or the post-wake re-stand), then the Hind's 30→60 u dodge window if the fight still leaks.
2. The fast suite (`tools/test/suite.py`) on this final tree, plus `nf-game --lib` and the other walkreal binaries.
3. The report's live checks for the main agent: whether the east-hall guards are visible from the station hall's mouth in the real game (the masked sight query answers blocked at the fusebox while their bullets reach it — gdbwatch on one #11 sight ray from (139.1, 49.5, −93.0) would settle whether the port's query over-blocks).

## ntfy failed

All three fallbacks failed at the network level (2026-11-11 ~06:03 UTC):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "...urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'los-mask-08-sisters: done — ...', headers={'Title':'NightfirePort'}), timeout=20)"` →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -O- --post-data "..." -T 15 https://ntfy.sh/nightfire-yoda-8k3x` →
   `Connecting to ntfy.sh (ntfy.sh)|159.203.148.75|:443... failed: Connection timed out.` /
   `Connecting to ntfy.sh (ntfy.sh)|2604:a880:800:14:0:1:73c0:2000|:443... failed: Network is unreachable.`

## Usage

not visible to me — this session's driver shows no token, cost or context-window figures to the worker (the IM gateway metadata carries only session/chat ids; no usage counters appeared in any tool result).
