//! The drawn gate's camera frustum (`ai-sight-drawn-gate`): the side-plane directions the sim
//! computes must be the game's — `Vision_Calculate_View_Fustrum` 0x1e7df8 [M] builds the far-quad
//! corners at depth 200 with x = ±tan(view/2)·aspect·200, y = ±tan(view/2)·200 and crosses them into
//! the four side planes the viewer carries at +0x70 (`amemcpy … 0x40`); `Vision_InView` 0x1e89e0 [M]
//! accepts when every plane meets `n·(P − eye) >= −slack` (the port's signed-distance reading —
//! nf-view scene.rs `vision_in_view`, fix-portal6). View angle 0x3F860A92 = 60° [M
//! Camera_CreateCameras 0x1bb2b8], aspect 4/3 = 0x3FAAAAAB [M Camera_CalcViewAngles 0x1baf70:29-33;
//! L ee_snapshot.bin @ 0x2a37c4 = 0]. No ISO needed — pure geometry.

use nf_game::drone::{frustum_sides, in_view, DRONE_DRAWN_SLACK};

fn close(a: [f32; 3], b: [f32; 3], what: &str) {
    for k in 0..3 {
        assert!((a[k] - b[k]).abs() < 1e-3, "{what} axis {k}: {} vs {}", a[k], b[k]);
    }
}

/// The level view (yaw 0, pitch 0): the four inward normals sit at the half angles — bottom/top mix
/// the up axis at 30° (cos = 0.86603, sin = 0.5), left/right the right axis at atan(tan30°·4/3) =
/// 37.5715° (cos = 0.79241, sin = 0.61000).
#[test]
fn level_view_side_planes_sit_at_the_half_angles() {
    let s = frustum_sides([0.0, 0.0, 1.0]);
    close(s[0], [0.0, 0.866_025_4, 0.5], "bottom");
    close(s[1], [0.792_41, 0.0, 0.610_00], "left");
    close(s[2], [0.0, -0.866_025_4, 0.5], "top");
    close(s[3], [-0.792_41, 0.0, 0.610_00], "right");
}

/// The acceptance band at the level view, eye at the origin, slack 1.0 (the [L] guard world-sphere
/// radius stand-in): ahead in view; 35° right in (37.57° half-angle); 45° right out; 26.6° up in
/// (30° half-angle); 42° up out; behind out — the cone's near/behind bypass is gone.
#[test]
fn level_view_band_matches_the_game_angles() {
    let s = frustum_sides([0.0, 0.0, 1.0]);
    let e = [0.0; 3];
    assert!(in_view(e, [0.0, 0.0, 10.0], DRONE_DRAWN_SLACK, &s), "ahead rejected");
    assert!(in_view(e, [7.0, 0.0, 10.0], DRONE_DRAWN_SLACK, &s), "35° right rejected");
    assert!(!in_view(e, [10.0, 0.0, 10.0], DRONE_DRAWN_SLACK, &s), "45° right accepted — the cone's 60° half-angle");
    assert!(in_view(e, [0.0, 5.0, 10.0], DRONE_DRAWN_SLACK, &s), "26.6° up rejected");
    assert!(!in_view(e, [0.0, 9.0, 10.0], DRONE_DRAWN_SLACK, &s), "42° up accepted — above the 30° half-angle");
    assert!(!in_view(e, [0.0, 0.0, -10.0], DRONE_DRAWN_SLACK, &s), "behind accepted — the cone's near bypass");
}

/// The pitch tilt (the live test's mechanism): forward at 45° up (pitch 0.5 in the π/2 units the
/// game stores at BL+0x8a8 [M Camera_SetToPlayer 0x1bc048's RotMatrixX(−pitch·π/2)]) — the horizon
/// point falls below the tilted bottom plane (rejected), the point along the new forward stays in.
#[test]
fn pitched_view_tilts_the_bottom_plane() {
    let f = [0.0, std::f32::consts::FRAC_1_SQRT_2, std::f32::consts::FRAC_1_SQRT_2];
    let s = frustum_sides(f);
    assert!(!in_view([0.0; 3], [0.0, 0.0, 10.0], DRONE_DRAWN_SLACK, &s), "the horizon stays in view while looking up 45°");
    assert!(in_view([0.0; 3], [0.0, 10.0 * FRAC_1_SQRT_2, 10.0 * FRAC_1_SQRT_2], DRONE_DRAWN_SLACK, &s), "the forward point rejected");
}

use std::f32::consts::FRAC_1_SQRT_2;
