## Your task: `lock-relock-channel`, a decrypted lock re-locks when its channel drops

`Lock::update_burnt_open` in crates/nf-game/src/lock.rs ports `Lock_Update` 0x1ae3d8 case 2 (`Lock_ReLock` 0x1b0750 when a kind-0 lock's channel drops), but nothing calls it, so `re_lock` never runs in play and a decrypted lock stays open for good. Confirm that against the per-tick lock pass, then call it from that pass with the real switch-channel value, in the game's order. Read 0x1b0750 in full: `re_lock` returns a key id to unequip, and the caller has to act on that as the game does. Also check that the lock's model, keypad state and door channel go back to the locked look and behaviour, and replace the `[S mapping]` comment for `burned` with what the decompile shows. Census which placements have a kind-0 lock with a non-zero out channel [D], and which script or switch can drop that channel. Pin through a real level from the census: Bond decrypts the lock, the channel drops, the lock is locked again and can be decrypted again. A kind-1 (laser-burn) lock must stay open. If no placement can reach the arm, say so in the report and keep a unit pin. Run the lock and decryptor binaries (grep tests for `Lock`), interactables_07000014_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/lock-relock-channel`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-06T23:47Z
