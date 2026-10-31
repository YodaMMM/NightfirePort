
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
