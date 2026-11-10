# Guard root motion with solid guards: the last walk reds

Continue on s/pretrans-arms-3 (525153962; read docs/reports/pretrans-arms-3.md "The walkreal sweep" and its red list): on master with solid guard bodies (0a9d39435) the per-frame +0x38 root-zero leaves 7 walk tests red in 6 binaries (01 x3, 05 door band exit Agent, 09 alarm switch, 0c whole mission + 0c east route, 11 real damage), all green on both parents - walks wedging against the changed world. Rebase onto current origin/master, repair each as a player would (or find a port bug: a guard that stops dead where the game's would keep walking), then run every walkreal_* binary one per call. Never lower damage or health, no loosened pins, no new ignores.

Push to `s/pretrans-arms-4`, report in docs/reports/pretrans-arms-4.md. No draw changes.

---
claimed-by: sandbox3 at 2026-11-10T03:09Z
