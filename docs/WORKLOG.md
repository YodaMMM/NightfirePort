
## lock-relock-channel

Branch `s/lock-relock-channel` from `origin/master` (22d5836b7). Task: a decrypted
lock re-locks when its channel drops. `Lock::update_burnt_open` in
crates/nf-game/src/lock.rs ports `Lock_Update` 0x1ae3d8 case 2 (`Lock_ReLock`
0x1b0750, kind-0 lock whose channel drops) but nothing calls it, so `re_lock`
never runs in play. Confirm against the per-tick lock pass, call it from there
with the real switch-channel value in the game's order, act on the returned
key-to-unequip id as the game does, check model/keypad/door channel return to
locked look and behaviour, replace the `[S mapping]` comment for `burned` with
the decompile, census kind-0 placements with a non-zero out channel [D], pin a
real level from the census (decrypt → drop → re-lock → decrypt again), keep a
kind-1 laser-burn lock open. Entry started — work follows.

Work (same entry): the case-2 tick wired into the per-tick pass (drone.rs) with the real
channel read and the Player_UnEquipKey clear; the refill + re-lock check in the game's
order inside update_burnt_open; decrypt/keypad unlocks land in the state-2 fold
(burned) with the completion memset's scan_solved clear; the hold loop split to the
game's state-1-only window; census probes (placement/writer/all-map/script sweeps): the
arm is unreachable on shipped data — every kind-0 lock's out channel feeds a door's
ch_lock and nothing writes it low; unit pins + one decompile-contradicting test fixed.
