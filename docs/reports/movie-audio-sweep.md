# movie-audio-sweep

Headless sweep of every movie on the disc (`crates/nf-view/tests/movie_audio_sweep_live.rs`, silent).
The audio layer's own sample counts are read, not the speakers.

## Fixes

1. A skipped briefing kept playing its sound. GameFlow case 6 calls `psiStopBackgroundMovie` on the
pop, for both skip and end [M 0x1c98c8:150]. In the port, `mission_movie_glue`'s decode-ahead arm
runs while the map load is still pending (until phase 2), and it kept the skipped briefing's player,
with its decoder and audio stream, alive. With an empty slot it reopened the same code. The glue now
stops a non-looping player once state 6 has popped back to the map load (`movie_code` set, state
ST_MAPLOAD) and does not start another one. `briefing_plays_live::a_skipped_briefing_stops_its_player_at_the_pop`
fails on the old glue (the player is still up on the first redraw after the skip) and passes on
the new one. The lib harness skip test (`the_briefing_skips_on_the_games_own_keys...`) now also
asserts the player is gone after the real-key skip. It failed before the fix.
   With the briefing stopped, one more pre-existing step showed up: in the frame between phase 2
   and the front end closing, the page arm restarted the mission list's looping backdrop
   (07350048), opening and demuxing a whole movie only to stop it a frame later. The arm now
   starts no backdrop when phase 2 has just resolved to a real level.

2. Movie audio on a surround or mono output device. The device writer put the left channel on every
even channel and the right on every odd one. On a 5.1 device the centre speaker played the left
channel and the LFE played the right. On a mono device the right channel was dropped. The writer is
now `PcmCursor`, the body of the cpal callback, made pure so it can be tested. It sends the pair to
the front L/R and mixes both channels for a mono device, the same mapping the game mixer's music
path uses (nf-audio). `the_movie_pair_lands_on_the_front_speakers` fails on the old mapping (5.1
output `[L,R,L,R,L,R]`) and passes on the new one. Stereo output is unchanged.

## Per-movie table

All tracks are PCM (kind 1), 48000 Hz, stereo, with 512-byte blocks [D]. Drift = video length minus
audio length. "gap" is the longest run of exact digital silence inside the track. "head"/"tail" is
the silence at the start/end.

| movie | samples | audio s | pictures | video s | drift ms | gap ms | head ms | tail ms |
|---|---|---|---|---|---|---|---|---|
| 07100001 | 2048512 | 42.677 | 1280 | 42.667 | -10.7 | 0.0 | 0.0 | 2.4 |
| 07100005 | 4440064 | 92.501 | 2776 | 92.533 | 32.0 | 0.1 | 0.0 | 1.3 |
| 07100009 | 4102400 | 85.467 | 2564 | 85.467 | 0.0 | 0.0 | 0.0 | 0.0 |
| 0710000C | 3819264 | 79.568 | 2387 | 79.567 | -1.3 | 0.0 | 0.0 | 1.3 |
| 07100011 | 4227328 | 88.069 | 2643 | 88.100 | 30.7 | 0.0 | 0.0 | 2.7 |
| 07100014 | 2052864 | 42.768 | 1283 | 42.767 | -1.3 | 0.2 | 0.0 | 1.3 |
| 07100018 | 1452288 | 30.256 | 907 | 30.233 | -22.7 | 0.1 | 0.0 | 4.4 |
| 07300004 | 2384128 | 49.669 | 1490 | 49.667 | -2.7 | 0.1 | 0.0 | 2.7 |
| 07300008 | 1520128 | 31.669 | 950 | 31.667 | -2.7 | 0.0 | 0.0 | 2.7 |
| 0730000B | 1420800 | 29.600 | 888 | 29.600 | 0.0 | 0.0 | 0.0 | 0.0 |
| 0730000D | 2724864 | 56.768 | 1700 | 56.667 | -101.3 | 2.1 | 0.0 | 2.0 |
| 07300013 | 2040064 | 42.501 | 1275 | 42.500 | -1.3 | 0.4 | 0.0 | 1.3 |
| 07300017 | 1485056 | 30.939 | 926 | 30.867 | -72.0 | 0.1 | 0.0 | 4.8 |
| 0730001B | 3384064 | 70.501 | 2115 | 70.500 | -1.3 | 0.1 | 0.0 | 1.3 |
| 07330048 | no audio | - | 1800 | 60.000 | - | - | - | - |
| 07350048 | no audio | - | 1800 | 60.000 | - | - | - | - |
| 07360048 | not on the disc | | | | | | | |
| 07370048 | 152320 | 3.173 | 96 | 3.200 | 26.7 | 0.0 | 200.0 | 3.6 |
| 07380048 | 526080 | 10.960 | 329 | 10.967 | 6.7 | 6.9 | 1401.2 | 1889.2 |
| 07390048 | 3685376 | 76.779 | 2304 | 76.800 | 21.3 | 0.1 | 0.0 | 1.9 |
| 073A0048 | 3238656 | 67.472 | 2023 | 67.433 | -38.7 | 20.6 | 166.8 | 501.2 |
| 073B0048 | 2814208 | 58.629 | 1756 | 58.533 | -96.0 | 0.0 | 0.1 | 33.7 |
| 073F0048 | 6769920 | 141.040 | 4230 | 141.000 | -40.0 | 3.8 | 0.0 | 3079.4 |

Checked for each movie: the demuxed body equals the `SSbd` size, whole L/R blocks, and decoded
samples = `samples()` = body/4 = the golden table. The last block decodes. The state-6 hold
(audio length in 60 Hz ticks) covers the whole track.

Device side (`PcmCursor`, 480/441-frame buffers): at 48 kHz, n-1 frames go out and are
sample-exact against the decode. At 44.1 kHz the frame count matches the expected length to the
frame (length error under 0.02 ms). After the end the output is silence, with no wrap back to the
start.

## Findings that are not port bugs

- No movie's video runs more than one picture past its audio. 0730000D (101 ms), 073B0048 (96 ms)
  and 07300017 (72 ms) carry audio up to 3 pictures past the last picture. This is disc data [D].
  The hold is the audio length, so the last picture stays up for that time.
- The long head/tail silences (07380048, 073A0048, 073F0048) are zero bytes in the disc's PCM body.
  The decode is byte-exact.

## Tests run (release, `--features nf-view/movies,nf-formats/movies`, silent)

movie_audio_sweep_live 3/3, briefing_plays_live 19/19, menu_movies 9/9, movie_playback_flow 10/10,
mission_outros_live 3/3, campaign_flow_live 5/5, load_scene_equality 1/1, movie_audio_layout_live,
movie_gop_stream_live, movie_hold_rate, movie_over_scene_live, movie_pacing_live,
movie_stream_bounded_live, movie_trigger_live all pass. nf-formats pss_movies_live 4/4 and
pss_vag_golden 4/4 pass. Lib `flow_audit::` + `movie::` 27/27 and the harness skip test pass.
pss_movies_g008: 4 of 5 pass; its every-picture full decode ran past the 10-minute command cap
(nf-formats is unchanged here). The lib `movie::lag_tests::the_briefing_decodes_well_ahead_of_real_time`
measured 38.8 fps against its 50 fps floor on a loaded machine. The decoder is untouched.

## Unproven

- The audio clock is the position written to the device buffer, so pictures lead the heard sound
  by the device's output latency (about 10-30 ms on shared WASAPI). This was not measured.
- Stopping a movie cuts its audio without a fade, as the game's `changeInputVolume(0)` does [M
  0x1073d8]. Whether the PS2 output clicks there was not checked live.
- Output formats other than f32 still run silent (unchanged).
