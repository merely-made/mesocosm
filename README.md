# Mesocosm

A first-person game of lineages.

You are a critter in an enclosed mid-scale ecosystem. You grow only by
incorporating other organisms — every part you carry used to be somebody.
You die, and your line continues or it does not. Your aim is to increase your
kind's share of the world's biomass, which no predator manages alone:
producers make the biomass, consumers shape the producers, and winning is
niche construction rather than a power curve.

One verb: **metabolize**. World into self, self into world. Morph yourself,
morph the world — and the world keeps what your generations did to it.

A mesocosm is ecology's mid-scale enclosed experimental ecosystem, larger
than a microcosm and smaller than the world. Playable forms may range from a
one-trait cell in proto-conditions to a compound, multicellular, colonial, or
distributed organism. Each generational run is one experiment in the same
enclosure.

Each epoch is embodied play followed by world examination and a turn-based
adaptation round. Metabolically complex lineages commit first; simpler,
faster-generating lines adapt later in response. The player may return to
other unlocked lineages, while every lineage left in the world continues to
change.

## Status

**Early.** The simulation, the body pipeline, and a windowed host run; the
game does not exist yet. Wave 1 of the execution plan is complete apart from
the cross-repo projection.

```sh
cargo run -p mesocosm-genet    # WASD move, E/Space eat, arrows orbit, Esc quit
cargo test --workspace
```

| Crate | What it is |
| ----- | ---------- |
| `mesocosm-core` | The simulation. Seeded, integer-only, a pure function of ordered intents. Owns all game state. |
| `mesocosm-runtime` | Host-neutral fixed-step driving, intent queue, and replay. |
| `mesocosm-mesh` | Body document to geometry: per-part greedy voxel meshing and rigid placement. |
| `mesocosm-render` | wgpu body renderer, headless-first so visibility is testable. |
| `mesocosm-genet` | The windowed host. Owns the loop and the device, never a rule. |

- [Project description](design_docs/PROJECT_DESCRIPTION.md)
- [Execution waves](design_docs/2026-07-31_execution_waves_plan.md) — what is built and what is next
- [Founding plan](design_docs/2026-07-30_mesocosm_founding_plan.md) — design and phases M0–M5
- [Games wing founding record](design_docs/2026-07-30_games_wing_founding.md) — shared architecture across Mesocosm, Paredros, and Isometry

## License

Mesocosm uses an explicit three-part license boundary:

- game code and repository documentation: MPL-2.0
- separately identified reusable library crates: MIT OR Apache-2.0
- original game assets: CC BY-SA 4.0

See [LICENSES.md](LICENSES.md) for scope and attribution rules. The published
`0.0.1` name reservation remains available under its original MIT OR
Apache-2.0 terms; this split begins with `0.0.2`.
