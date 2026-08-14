# Resident Views Composition Plan (2026-08-14)

**Status: founded 2026-08-14; proof gate open.** How the voxel world,
Burn/CubeCL, the tracer, the mesher, collision, and persistence compose
on one device. Ratified direction (Mark, 2026-08-14): the voxel world
is Burn-addressable state while remaining voxel-authoritative state.
Burn does not import voxels into a second world representation; every
consumer receives a resident view of the same facts.

The platform half of this ruling (lanes are program shapes, the
allocator direction, the resident-views law, the tensor-bundle plane
taxonomy) lives in mere
(`design_docs/mere_docs/technical_architecture/2026-08-13_spatial_compute_plan.md`
section 0.5). This doc is the wing application: the seam type, the
ownership rows, and the first proof.

This is the graph architecture applied to spatial state. The graph is
not inside layout, rendering, search, or ML; it stays authoritative
while each consumer gets an appropriate resident reading. `Ground` is
not inside the tracer, Burn, or parry; same law. The wing had already
grown the authoritative half before the seam was named: `Ground`
carries `revision: u64` and a dirty-brick queue whose doc comment
states the law ("projection work queue, not world authority; the
revision and brick bytes carry the authoritative change").

## The seam

```
ResidentChunk
  identity
  world position and extent
  authoritative revision (mirrors Ground revision at materialization)
  typed channel planes
  device-buffer handles
  shape, strides, element types
  valid-read epoch
  dirty regions
```

Views produced without copying: Burn tensor view, CubeCL kernel view,
tracer/material view, mesher input, collision-refresh input,
persistence/checkpoint view. A view is one arrangement of the chunk,
never a replacement for it.

**Planes are a bundle, not one tensor.** Exact planes (occupancy,
palette id, provenance, flags), fixed-point simulation planes
(temperature, moisture, nutrient, pressure; replay-exact), learned and
derived planes (latents, light estimates; projections, drift is a
shrug), temporary planes (inference output, constraint masks, work
queues). The plane taxonomy is the fact-bearing ruling: exact and
fixed-point planes belong to the record's side of the doctrine;
everything else is projection.

**Allocation flows compute-side out.** Burn/CubeCL can hand its
allocations outward but cannot cheaply adopt a foreign buffer, so
planes are allocated through the CubeCL client (conatus's resident
allocations) and leased to tracer, mesher, and render tenants.

## The transition path

```
resident chunk
    | tensor view
Burn model or CubeCL process
    | proposed channel deltas (candidate planes)
constraint and game-rule projection
    | accepted delta
authoritative commit + revision bump (Ground)
    |
tracer / mesher / collision / persistence observe the revision
```

**Burn proposes; the record disposes.** Inference stays GPU-resident.
The CPU reads accepted deltas, checkpoints, hashes, and small
reductions at durability boundaries, never whole planes per tick (the
4-byte readback pattern from the resident lane). A commitment that
reads a projected channel snapshots it into the fact at commitment
time.

## Ownership

- **Mesocosm** owns channel meanings, authoritative voxel facts
  (`Ground`), constraints, and commits.
- **Conatus** owns the shared device schedule, resident allocations,
  leases, and execution order.
- **Burn/CubeCL** owns tensor evaluation and learned or numerical
  transforms.
- **Renderling/netrender** consumes resident buffers and revisions as
  tenants.
- **Parry** derives tactile query state from committed voxel revisions
  (never from candidate planes).
- **Persistence and peers** exchange canonical chunk facts and deltas,
  never GPU allocation details.

## The first proof (gate open)

Use one existing `Ground` region; invent no second chunk model.

1. Materialize its authoritative bricks into one resident chunk bundle.
2. Construct a Burn tensor view over a material-derived plane
   (temperature or moisture).
3. Run a simple 3D diffusion or NCA pass through Burn/CubeCL.
4. Have the brick tracer visualize that field from the same resident
   allocation.
5. Commit one constrained field delta back into world history.
6. Receipts: one device; one allocation per plane (proved by buffer
   identity, the P4 epoch style); zero per-tick bulk copies; revision
   coherence (every consumer observes the same revision); replay
   (facts reproduce the committed state with no simulation present);
   unchanged-frame silence (a static world causes no recomputation,
   gated on elapsed time, never frame counts).

Generation, batched brains, fluids, and learned growth then stack on
this seam rather than each negotiating its own import path.

**Sequencing:** the place-graph engine work (G2+) is live in this tree
and owns `Ground`'s internals right now. This proof starts after that
work settles; the seam consumes `Ground`'s public revision contract
and does not reach into brick internals.

## Stop rules

- No second world representation inside any consumer; a consumer that
  wants one is asking for a view type instead.
- No CPU whole-plane reads in the per-tick path; durability reads are
  deltas, hashes, checkpoints, reductions.
- Candidate planes never feed collision, persistence, or peers; only
  committed revisions do.
- A view is promoted to a shared contract only after multiple real
  consumers prove it (the lease's promotion rule).
