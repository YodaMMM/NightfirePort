# Aim snap: finish on the corrected filter

Branch from `origin/s/aim-snap-placement`, push to `s/aim-snap-final`. Read docs/reports/aim-snap-objects.md first, but note its rocket and laser results are void: `aim_pick` read object flags, which never carry 0x8000, so every static surface was skipped and the snap never landed. `s/aim-snap-placement` reads `inst.placement.flags` instead.

Now green on that tip: `firing_along_the_walkway_breaks_no_panel`, with the ray stopping at x 6.958338, the game's own point. [L PCSX2 breakpoint at 0x1a4d90: head (16.741, -1.137, 4.62), hit (6.958, -1.137, 4.62), object 0.]

Red on that tip, each to be fixed by matching the game's ray, not by turning the snap off:
- rocket_glass08_live (exploded at z -96.706, must pass the window plane -96.75), rocket_stop_plane08_live if red. Check the cel walk through the 08 window and the glass mask.
- interactables_07000014_live and fusebox_geometry_07000014_live: the laser must cut. The game's ray hits object glists, so the wire needs to be in the ray.
- rocket_origin_live against the true snapped point.

## Live numbers for 07000008 (PCSX2, breakpoints, Bond idle at the level start, launcher 0x2d held)

The report's claim that the game's ray sees nothing at the window is wrong. Head (139.531, 50.792, -76.431), raw aim (139.924, 50.792, -332.431). The ray hits OBJECT class 0x1c at (139.562, 50.792, -96.706); the object's position is (140.719, 49.5, -96.751), r 1.812. Same result for the start weapon and the launcher. So objects of that class are in the ray.

The rocket's own flight, per tick: (139.531, -76.431) start at the HEAD, then (139.538, -80.472), (139.545, -85.472), (139.553, -90.472), (139.56, -95.472), (139.568, -100.472), y 50.792 throughout. Its hit chain lists that same object on the last step. The final blast point was not captured; the tests' z -100 is older evidence. The launcher was given by a memory write, so check whether the midpoint gate (first-person model loaded) was true in this run before changing the port's origin: the beam on 0700001b DID start at the midpoint (16.2, -1.23, 4.53).

Then check walkreal_0700000c_live, walkreal_07000014_live, walkreal_07000016_live, and walkreal_0700001b_live's consoles from spots a player would use. Strip the NF_AIMPROBE prints.

Use a PCSX2-backed number wherever one exists. Run each binary whole and quote the results.
