# Guard root motion: finish the PreTransCallback arms

Branch s/pretrans-arms-2 (616d4b0e1; read docs/reports/pretrans-arms-2.md) ports the per-frame +0x38 root-zero and its setters; 25 of 27 walkreal binaries pass. Rebase it onto current origin/master (solid guard bodies landed: 0a9d39435), then fix the remaining reds: walkreal_07000002 00 Agent real health (Bond dies to one 9.8 hit at the geisha-2 doorway) and walkreal_07000003 Agent real damage (a guard cluster engages earlier), as a player would; then every walkreal_* binary one per call. Never lower damage or health, no loosened pins, no new ignores.

Push to `s/pretrans-arms-3`, report in docs/reports/pretrans-arms-3.md. No draw changes.

---
claimed-by: sandbox4 at 2026-11-09T21:15Z
