# aim-ease-wire-accuracy-pins — real-input pins for two landed behaviours

Two changes on master have no direct real-input test:
1. Aim assist: while the aim button is held, the assist offsets ease toward the manual offsets at 0.5 per tick and the Check_AutoAim scan does not run [M Player_AutoAim 0x1a7c98:50-59]. Pin on real input: lock the assist on a guard hip-firing, raise aim, show the offsets ease and the next round goes where the view points; sister: with the button up the scan still runs.
2. Wire accuracy: the move quad zeroes on climb ticks only [M Player_Update 0x193600:129-134], so a wire rider keeps the last walk lanes and guards read him as MOVING [M DroneWeap_DoOpponentTargetting 0x179a48:34-41 -> DroneWeap_DoBulletAccuracy 0x179740:71-73]. Pin on real input on 0d's wire: the rider reads moving, a stationary Bond reads still.

No behaviour change is expected; if a pin shows the port differs from the decompile, fix it with the address. Every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox3 at 2026-11-04T21:54Z

---
## Report (done, 2026-11-04T23:24Z)

