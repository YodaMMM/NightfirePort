# Gas draws and guard lines: the last re-pin

Branch from `origin/s/gas-draws-8`, merge current `origin/master`, push to `s/gas-guard-repin`. Keep every gas draw. This one branch now carries the guard radii, guard glass and gas work, so getting it green lands all three.

Read docs/reports/gas-draws-8.md section 5 for the list: 14 reds on 11 levels (01 d3, 03 x2, 05, 06 x2, 08, 0c, 0d, 11, 12, 13 x2, 14, 15), plus grenade_live `a_guard_in_aim_stand_fire_0x68_flees_his_own_landed_frag`. Master has moved since: jump landings now force two steps, landing and step probes read water, and the sighted guard ignores sound. Re-measure first, the list will differ.

For each red say whether the walk's route is tuned to the old stream or a guard behaves wrongly. Stream-tuned walks get route and timing levers; the 03 keep-off on master is an example. Do not loosen pins tagged [L], do not add ignores to pass, and do not change game code to suit a walk. A guard that behaves wrongly gets a fix with a decompile citation.

Finish with one total line from a full run, not per-binary counts alone.

---
claimed-by: sandbox1 at 2026-11-06T09:37Z

---
## Report (done, 2026-11-06T11:11Z)

