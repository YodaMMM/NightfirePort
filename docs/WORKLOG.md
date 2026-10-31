
## level-1b-drake-walker-kill (started 09:06 UTC 2026-10-31)

Task: make Drake's defeat-by-real-fire raise ch 0x0e and run the finale chain —
`crates/nf-view/tests/interactables_0700001b_live.rs::drake_defeat_by_real_fire_ends_the_mission`
(~line 211) is #[ignore]d: the walker-driven kill (w.shoot: exposed_aim box-centre aims + the
reload/strafe inputs) leaves death channel 0x0e down and the NIS chain dead, while the SAME
kill through raw step_bond (probe1b_blockers::probe_drake_death) spawns him enabled, dies at
t81 into AstronautDeath 0xbf and raises ch 0x0e. Find what differs (hit data, damage route,
death state, or a Walker input interrupting the death) by tracing Drake's state and the
ch-0x0e writer on both paths [M: cite the death handler that writes 0x0e]. Fix the port (not
the test), un-ignore the test, keep the teleport approach as documented. Branch
`s/level-1b-drake-walker-kill`, from origin/master b44ac857.

Progress log:
- 09:06 branch created from origin/master b44ac857 (sandbox reused from the level-14 session:
  repo, work/, ISO, nf-queue, cargo all present). Branch pushed with this entry.
- 09:28 THE BLOCKER RESOLVED — the ignore note was stale; the test's own ch-0x0e end assert was
  the contradiction. Trace (NF_KILL_TRACE added to Walker::shoot, env-gated, the file's
  NF_HEALTH_TRACE pattern): the walker kill enters AstronautDeath 0xbf at +130 with ch 0x0e ON
  (corpse same tick), the defeat NIS plays out, ch 0x0b lands at the NIS's t600, MissionState 6
  at the t720, to_end 0x0730001b — the chain is ALIVE on current master (a later 1b merge fixed
  the death route; the 2026-10-23 note predates it). What never survived was the test's
  end-of-chain `channels.on(0x0e)`: SP#198's stay is 0 [D placement #198 param 4, dumped from
  the disc], so when the 720-tick NIS [D length] runs out the player reaches SP_Update's case
  3, whose re-arm writes the OUT channel 0 [M 0x1dbda8 case 3 cVar10='\0' + the tail
  `switch_channels[out] = cVar10`] — the game itself clears ch 0x0e after the NIS. Test fixed
  per the rule (never weaken a decompile-backed assert, but this one overreached its citation):
  the raise is now asserted AT the kill [M 0x1747b8/0x13e660], the clear asserted at the chain's
  end, the wait loop runs the full window (no early break) so the re-arm tick lands inside it.
  Un-ignored; interactables_0700001b_live 7/7, fix_finale_live 4/4, probe1b_blockers 5/5,
  probe1b_drake 1/1, walkreal 9/9 (+1 ignored), probe1b_route 3/3, probe1b_rover 1/1,
  interactables probes 1/1 + 5/5. Port code untouched — the fix is test-side only.
- 10:32 FINISH: report written to docs/reports/level-1b-drake-walker-kill.md (the stale-note
  finding, the [D] SP#198 dump, the [M] case-3 re-arm chain, the test fix, the partial suite
  totals 340/983 with the 3 pre-existing failing binaries, the live check, what's left). The
  suite ran as foreground chunks — this sandbox kills background jobs between tool calls, so
  tools/test/suite.py (nohup, then setsid) died twice with an empty suite.out; the resumable
  fg_runner1b.py (state /home/z/suite1b_state.json) carries the partial for the next message.
