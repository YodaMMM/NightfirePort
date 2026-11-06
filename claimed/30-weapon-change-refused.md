# The refused weapon change

`docs/systems/sound-coverage.md` lists `Player_WeaponChange` 0x1a6f80:128: sound 0x1DB when the change is refused on `level_tag+0x62 == 1`, MISSING because the tag byte is unparsed. Branch from current `origin/master`, push to `s/weapon-change-refused`. Find what that byte is and which levels set it, parse it, and port the refusal: does the weapon stay, and does the sound play? Test on a real level that sets it, proved red on the old code. Do not guess at the byte's meaning; cite its reader and the disc value.

---
claimed-by: sandbox4 at 2026-11-06T20:49Z
