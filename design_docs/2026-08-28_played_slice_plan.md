# Played Slice Plan (2026-08-28)

**Status: in progress (2026-08-28); PS0 and PS1 scoped, ruled by Mark.**
The first slice of Mesocosm a hand actually plays: the live epoch as the
ruled terrarium section with direct control of your organism. Scope ruled
2026-08-28: PS0 + PS1; succession and the epoch boundary are PS2, extracted
below as future work. Renderer ruled: the brick-traced side-on section.

The rulings this plan executes, cited not restated: the terrarium section
and direct control (`2026-08-18_vessel_briefs_and_presentation.md` §2), the
Rain-World pull-back with camera-is-not-the-person, control as a recorded
intent over `World.controlled` (`2026-07-31_phenotype_plan.md`, P1 and the
2026-08-01 correction), Route A's textless HUD lane
(`2026-08-02_views_founding_plan.md`), and metabolize as the single direct
verb (founding plan). The slice depends on no open gate: PD1b/PD2 upgrade
feeding's authority, not playability, and E1's interim feeding is landed.

**D5 (burn vs incorporate) is deliberately not in the slice.** Its doc
still names it the recommended first playable proof, but Mark rejected the
hotkey-pair interface (2026-08-28: "framing it as a hotkey is a bit odd…
not a workable ui"). D5 waits on an interaction design consistent with
"experienced directly, never through a menu" — likely diegetic (how or
where the meal happens), designed with Mark before implementation.

## PS0 — revive the hand

The playable host already exists: `mesocosm-genet` drives a real `World`
through `Runtime::advance` with WASD/E/Space/Q mapped to recorded intents
and the minimap HUD composited textlessly. It has been dead behind one
call-site type mismatch (`hud.rs` handing `&Vec<PaintCmd>` where netrender
now takes `&[PaintCmd]`).

**Done when:** `mesocosm-genet` builds and runs against current netrender;
a keyboard moves the controlled organism through the live world at fixed
timestep; clippy is clean; and the fix is the call site, not a fork of the
HUD lane.

**Receipt, 2026-08-28.** Landed with zero source changes — the diagnosis
was wrong in an instructive way. The "netrender API drift" was two cargo
source identities for one crate: sprigging (via genet's workspace) pins
the netrender family by rev `6f1a4fe7`, mesocosm rode `branch = "main"`
whose cached fetch was stale at an ancestor commit, and even after
updating, cargo keys git sources by URL *plus reference*, so rev and
branch never unify. Mesocosm's workspace now pins the same rev string
genet uses; the types deduplicated and the host compiled untouched. The
full workspace's tests and `-D warnings` clippy are green with genet
included for the first time since the drift began, and the host launches
and runs its loop. Keyboard play is Mark's half of the receipt.

## PS1 — the ruled look, played

Reunite the live `World` with the terrarium-quality renderer. Today they
have never met: the genet host plays a real world through the rasterized
`mesocosm-render` view with an orbit camera, while the brick tracer — the
ruled side-on orthographic section — has only ever been fed fixture poses.

The work: the genet host's main view becomes the lens brick tracer. The
tracer's `BrickMap` binds the live `Ground` (full build at genesis,
`refresh` from drained dirty bricks as play carves and deposits, revision
advancing with the world). The camera is an orthographic slab section
following the controlled organism — the pulled-back framing G2 ratified —
with the section's few-voxel depth; camera motion is presentation only and
never enters the trace. Bodies ride the lens's existing body-projection
path (the V2 `BodyLensProjection` lineage and the menagerie precedent
decide the exact mechanism; the implementer follows the landed pattern
rather than inventing one). The minimap HUD composites over the section
exactly as it does today. Arrow keys become a small presentation-only
section pan, replacing orbit.

Receipt discipline, per the house pattern: the host writes its intent
trace and `Receipt` (seed, organisms, steps, state hash) on exit; a
`--replay <trace>` mode drives the same headed frames from a recorded
trace with no keyboard and must land the identical state hash, and doubles
as the self-driving receipt (capture written, hash asserted) so the slice
is judged without a human in the loop after the first recording.

**Done when:** a played session in the brick-traced side-on section — move,
metabolize, deposit, carve — writes a trace on exit that replays headless
to the identical state hash and headed to a non-trivial capture; the
section visibly follows the controlled organism with the ruled framing;
the HUD minimap rides Route A unchanged; and clippy stays clean across the
workspace.

## PS2 — extracted, not scheduled

The loop's two unwired seams, deliberately out of v1: handling
`World::control_lost()` (death → witnessing → `Intent::TakeControl` of an
eligible critter — the documented disembodiment seam) and the
earned-reproduction epoch end driving `Runtime::end_epoch` toward the
adaptation phase. Both have landed core machinery and zero host wiring.
The trait-board review screen stays with Views route B.

## Findings

- **2026-08-28:** the long-flagged "mesocosm-genet does not compile
  against current genet" was neither genet drift nor an API break: it was
  a duplicate-source split (rev-pin vs stale branch fetch of the same
  netrender commit line). The ninth resolution trap; fixed by pinning the
  family to the rev string genet pins.
- **2026-08-28:** two rendering pipelines have never met: genet's host
  (real world, rasterized) and the lens tracer (ruled look, fixture-fed).
  PS1 is their join.
- **2026-08-28:** nothing calls the epoch boundary or `control_lost()`
  from any host; the adaptation phase has only ever run headlessly against
  the bare lineage roster.

## Progress

- **2026-08-28 (later):** PS0 landed: the host builds, tests and clippy
  are green workspace-wide, and it launches and runs. Netrender-family
  pins aligned to `6f1a4fe7`; netrender's registry-published stack noted
  as a possible future retirement of the vello git patch.
- **2026-08-28:** founded; scope, renderer, and the D5 deferral ruled by
  Mark the same day.
