# fx-muzzle-repin-4 — redo on the right base

s/fx-muzzle-repin-3 was cut from master and contains none of the muzzle code, so its green results measure nothing. Start from origin/s/fx-muzzle-repin-2, not master, then merge origin/master. First check, and quote the output in your report: `grep -c shot_flashes crates/nf-game/src/drone.rs` must be above 0 and `grep -c init_gt_shot_smoke crates/nf-game/src/gas.rs` above 0. Then finish fix_npc_doors_live, walkreal_07000011_live and walkreal_07000014_live, keeping every draw. Report each binary.

---
claimed-by: sandbox4 at 2026-11-05T16:38Z

---
## Report (blocked, 2026-11-05T17:38Z)

