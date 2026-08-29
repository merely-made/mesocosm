// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! TD1's headless population instrument, carried through TD2, TD2b, TD2c,
//! TD2d, TD5, TD5b, TD6, TD7, S1, TD8 and TD9 as each round's measuring stick.
//!
//! Founds real worlds and drives them with nothing but `Intent::Idle` for
//! several starter lifespans, sampling per-kingdom alive counts, total
//! biomass, and cumulative births/deaths every `SAMPLE_INTERVAL` ticks. A
//! verdict is read off the curve by simple, stated arithmetic: no fitted
//! models, no significance tests. See `design_docs/2026-08-29_terrarium_dynamics_plan.md`, TD1.
//!
//! Two batches run: a **baseline** at today's constants, and a **control**
//! founded with a single organism and no producer to feed it (expected to
//! collapse). The control is what proves the verdict logic can say something
//! other than the baseline's answer — an instrument that only ever reads one
//! way has not shown anything.
//!
//! A count inside its band is not enough to pass. [`Verdict::Thins`] catches
//! the world that holds its numbers by becoming a producer stand; TD2b's
//! receipt called six such runs "breathes" before this verdict existed.
//!
//! Boil and collapse both exit a run early once decided (see [`run`]) — a
//! fixed 10,000-tick horizon is not practical to *run* to the end on a
//! chain that is actively boiling, since its own per-tick cost grows with
//! it. Only a run that never tips either way pays for the full horizon.
//!
//! ```text
//! cargo run -p mesocosm-core --example population_instrument --release
//! ```
//!
//! Writes `Code/testing/mesocosm/td10_chain.json` (curves + verdicts
//! per seed) and prints a terminal summary; each earlier round's receipt
//! keeps its own filename and none is overwritten. `Code/testing/<repo>/` is this
//! workspace's standing receipts convention; the path is found by walking up
//! from this crate to the `repos` ancestor rather than a fixed `../` count,
//! so it survives the crate moving depth within the repo.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use mesocosm_core::world::{ENCLOSURE, FOUNDERS};
use mesocosm_core::{Intent, Kingdom, World};

/// `lifespan_for_mass(1000mg)` — the starter mass every `World::new` founds
/// its played critter with — is 3,000 ticks after the 2026-08-29 retune
/// (verified against `ecology.rs`'s quarter-power tables), so this many ticks
/// is three and a third starter lifespans. It was ten before the retune
/// stretched the tempo; the horizon stayed put so the two receipts compare.
const TICKS: u32 = 10_000;
/// Sample cadence. 100 samples over the run is enough to see the shape of
/// the curve without writing a receipt per tick.
const SAMPLE_INTERVAL: u32 = 100;
/// Ten arbitrary seeds. Nothing about them is tuned; the point is that the
/// verdict holds across an uncherrypicked handful, not one lucky draw. Widened
/// from five on 2026-08-29 for TD2, because a retune judged on five draws is
/// judged on one bad founder composition either way.
const BASELINE_SEEDS: [u64; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// The first five of those seeds, reused for the control so a reader can see
/// the same draws produce opposite verdicts under a different founding.
const CONTROL_SEEDS: [u64; 5] = [1, 2, 3, 4, 5];

/// Founders beyond the played critter: the world's own area-scaled cohort,
/// read from `world::FOUNDERS` rather than typed here, so the instrument
/// measures the terrarium that ships instead of a fixture that used to.
/// (2026-08-29 S1; it was a literal 60 through TD7.)
const BASELINE_ORGANISM_COUNT: u32 = FOUNDERS;
/// Zero extra: the played critter (a Consumer) founds alone, with no
/// producer in the world to ever feed it. It can find no prey, so its
/// income is permanently zero and upkeep starves it out. This is the
/// broken control, not a baseline variant.
const CONTROL_ORGANISM_COUNT: u32 = 0;

/// A run reads **boil** when the end population is at least this many times
/// the start population and is still rising on the last sample. The
/// playtest finding this instrument documents (61 -> 8,155, ~134x) clears
/// this bound by more than an order of magnitude, so 10x is a conservative
/// floor for "still exploding" rather than a hair-trigger.
const BOIL_MULTIPLE: u64 = 10;
/// A run reads **collapse** when the end population has both fallen below
/// the start population and is at or below this fraction of it (integer
/// division, no forced floor — see the comment on the check itself for why
/// a one-founder start needs true zero rather than a rounded-up floor).
/// 1/50th is "near enough to zero that stragglers don't matter," not a
/// demand for literal extinction.
const COLLAPSE_FRACTION: u64 = 50;

/// Edge of the occupancy bucket this receipt reports.
///
/// It used to mirror `ecology.rs`'s `CROWD_CELL`, the grid the producer income
/// rule divided by. **TD6 retired crowding** — the soil does that job now, per
/// voxel column and on mass rather than on head count — so this is a plain
/// density statistic kept at 8 so the `max_cell` column still compares against
/// every earlier round's receipt.
const CROWD_CELL: i32 = 8;

#[derive(Clone, Copy)]
struct Sample {
    tick: u32,
    alive: [u32; 3],
    total_biomass_mg: u64,
    cum_born: u64,
    cum_died: u64,
    /// Occupancy of the fullest crowding cell. This is the number the
    /// producer income rule divides by, so it says directly whether
    /// self-thinning is even being asked to do anything.
    max_cell: u32,
    /// Furthest occupied position from the enclosure's centre, on either
    /// horizontal axis. Anything past `world::ENCLOSURE` is a body standing
    /// where the ground the enclosure grew does not reach.
    span: i32,
    /// Count of living organisms strictly past `ENCLOSURE` on either
    /// horizontal axis: the escapee proof for TD2b's wall. Zero across a run
    /// is what "the wall holds" means in receipt terms. (2026-08-29 TD2b)
    outside: u32,
    /// Matter held in the ground. (2026-08-29 TD6)
    soil_mg: u64,
    /// Soil plus every body's substance and reserve — the conserved total.
    /// Flat across a whole run is the closed cycle's own receipt.
    total_matter_mg: u64,
}

impl Sample {
    fn total_alive(&self) -> u64 {
        self.alive.iter().map(|&count| u64::from(count)).sum()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    Collapse,
    Boil,
    /// The count held its band, but a kingdom the world was founded with is
    /// gone at the horizon. A pure producer stand is not a terrarium
    /// breathing, and before this verdict existed it read as one — TD2b's
    /// receipt called six such runs "breathes". (2026-08-29 TD2c)
    Thins,
    Breathes,
}

impl Verdict {
    fn label(self) -> &'static str {
        match self {
            Verdict::Collapse => "collapse",
            Verdict::Boil => "boil",
            Verdict::Thins => "thins",
            Verdict::Breathes => "breathes",
        }
    }
}

const KINGDOM_NAMES: [&str; 3] = ["producers", "consumers", "decomposers"];

struct RunResult {
    seed: u64,
    founders: u32,
    samples: Vec<Sample>,
    verdict: Verdict,
    reason: String,
    /// The tick the verdict was read at. Equal to `TICKS` for a run that had
    /// to watch the full horizon (breathes); earlier for a run that decided
    /// itself before that, per the early-exit note on [`run`].
    decided_tick: u32,
    elapsed_ms: u128,
}

fn kingdom_index(kingdom: Kingdom) -> usize {
    match kingdom {
        Kingdom::Producer => 0,
        Kingdom::Consumer => 1,
        Kingdom::Decomposer => 2,
    }
}

fn sample_of(world: &World, tick: u32, cum_born: u64, cum_died: u64) -> Sample {
    let mut alive = [0u32; 3];
    let mut total_biomass_mg = 0u64;
    let mut density: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    let mut span = 0i32;
    let mut outside = 0u32;
    for organism in world.organisms.iter().filter(|o| o.is_alive()) {
        alive[kingdom_index(organism.kingdom())] += 1;
        total_biomass_mg += organism.biomass_mg();
        *density
            .entry((
                organism.position[0].div_euclid(CROWD_CELL),
                organism.position[2].div_euclid(CROWD_CELL),
            ))
            .or_default() += 1;
        span = span
            .max(organism.position[0].abs())
            .max(organism.position[2].abs());
        if organism.position[0].abs() > ENCLOSURE || organism.position[2].abs() > ENCLOSURE {
            outside += 1;
        }
    }
    Sample {
        tick,
        alive,
        total_biomass_mg,
        cum_born,
        cum_died,
        max_cell: density.into_values().max().unwrap_or(0),
        span,
        outside,
        soil_mg: world.soil().total_mg(),
        total_matter_mg: world.total_matter_mg(),
    }
}

/// Runs one seeded world for up to `TICKS` idle ticks, sampling every
/// `SAMPLE_INTERVAL`, and reads a verdict off the resulting curve.
///
/// **Boil and collapse both exit early**, the moment [`verdict_for`] can
/// already read one off the samples taken so far. Neither verdict changes
/// its mind later: a boiling chain's per-tick cost only grows as its
/// population does (this is what made a fixed 10,000-tick horizon
/// impractical to *run* at today's constants — the instrument itself boiled
/// alongside the world), and a collapsed chain has no organisms left to
/// recover with. **Breathes is the one verdict that cannot be read early**:
/// no sample can prove a population stays bounded for ticks it has not
/// reached yet, so that path always watches the full horizon.
fn run(seed: u64, organism_count: u32) -> RunResult {
    let started = Instant::now();
    let mut world = World::new(seed, organism_count);
    let founders = world.organisms.len() as u32;

    let mut cum_born = 0u64;
    let mut cum_died = 0u64;
    let mut samples = vec![sample_of(&world, 0, 0, 0)];
    let mut decision: Option<(Verdict, String, u32)> = None;

    for tick in 1..=TICKS {
        world.apply(Intent::Idle);
        let tally = world.last_tally();
        cum_born += u64::from(tally.born);
        cum_died += u64::from(tally.died);
        if !tick.is_multiple_of(SAMPLE_INTERVAL) {
            continue;
        }
        let sample = sample_of(&world, tick, cum_born, cum_died);
        eprintln!(
            "    seed {seed}: tick {tick:>6} alive={:<7} elapsed_ms={}",
            sample.total_alive(),
            started.elapsed().as_millis()
        );
        samples.push(sample);

        let (verdict, reason) = verdict_for(&samples);
        if matches!(verdict, Verdict::Boil | Verdict::Collapse) {
            decision = Some((verdict, reason, tick));
            break;
        }
    }

    let (verdict, reason, decided_tick) = match decision {
        Some(decided) => decided,
        None => {
            // Ran the full horizon without an early boil or collapse.
            // Always land the final tick, even if TICKS isn't a multiple of
            // the sample interval (it is, today, but the receipt shouldn't
            // silently drop the end state if that ever changes).
            if !TICKS.is_multiple_of(SAMPLE_INTERVAL) {
                samples.push(sample_of(&world, TICKS, cum_born, cum_died));
            }
            let (verdict, reason) = verdict_for(&samples);
            (verdict, reason, TICKS)
        }
    };

    RunResult {
        seed,
        founders,
        samples,
        verdict,
        reason,
        decided_tick,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

/// Reads collapse / boil / breathes off a sample curve by the arithmetic
/// documented on [`BOIL_MULTIPLE`] and [`COLLAPSE_FRACTION`]. Collapse is
/// checked first: a population that is both near-zero and still nominally
/// "10x start" (start of 1, end of 0) must not be misread as neither.
fn verdict_for(samples: &[Sample]) -> (Verdict, String) {
    let start = samples.first().expect("at least the tick-0 sample");
    let end = samples.last().expect("at least the tick-0 sample");
    let start_total = start.total_alive();
    let end_total = end.total_alive();

    // No forced minimum of 1 here: a one-founder start (the collapse
    // control) has a floor of 1/50 = 0, which is deliberate. A floor
    // clamped up to the start count itself would read "collapse" from the
    // very first sample regardless of what actually happened, since the
    // population never has to move to satisfy `end <= start`. Requiring
    // `end_total < start_total` on top of the floor is what turns this back
    // into "it fell," not "it started small."
    let collapse_floor = start_total / COLLAPSE_FRACTION;
    if end_total <= collapse_floor && end_total < start_total {
        return (
            Verdict::Collapse,
            format!(
                "end population {end_total} <= collapse floor {collapse_floor} (start {start_total} / {COLLAPSE_FRACTION}) and below start {start_total}"
            ),
        );
    }

    let previous_total = samples
        .get(samples.len().saturating_sub(2))
        .map(Sample::total_alive)
        .unwrap_or(end_total);
    let boil_bound = start_total.saturating_mul(BOIL_MULTIPLE);
    if end_total >= boil_bound && end_total > previous_total {
        return (
            Verdict::Boil,
            format!(
                "end population {end_total} >= {BOIL_MULTIPLE}x start {start_total} ({boil_bound}) and still rising (previous sample {previous_total})"
            ),
        );
    }

    // A count inside its band is necessary but not sufficient. A world that
    // founded three kingdoms and ends holding one is a producer stand, not a
    // food web, so it must not be able to spend the word "breathes". Only
    // kingdoms actually present at tick 0 are required to survive: the
    // collapse control founds a lone consumer and would otherwise be failed
    // for missing producers it never had.
    let lost: Vec<&str> = (0..3)
        .filter(|&k| start.alive[k] > 0 && end.alive[k] == 0)
        .map(|k| KINGDOM_NAMES[k])
        .collect();
    if !lost.is_empty() {
        return (
            Verdict::Thins,
            format!(
                "end population {end_total} held its band, but the world founded {}/{}/{} P/C/D and ends {}/{}/{}: {} died out",
                start.alive[0],
                start.alive[1],
                start.alive[2],
                end.alive[0],
                end.alive[1],
                end.alive[2],
                lost.join(" and "),
            ),
        );
    }

    (
        Verdict::Breathes,
        format!(
            "end population {end_total} stayed above the collapse floor ({collapse_floor}) and below or falling from the boil bound ({boil_bound}), all {} founded kingdoms still alive",
            start.alive.iter().filter(|&&n| n > 0).count(),
        ),
    )
}

fn main() {
    println!("TD1 population instrument: {TICKS} ticks, sampling every {SAMPLE_INTERVAL}");
    println!(
        "verdict arithmetic: collapse when end < start and end <= start/{COLLAPSE_FRACTION}; boil >= {BOIL_MULTIPLE}x start and still rising; thins when the band held but a founded kingdom died out; else breathes\n"
    );

    println!("== baseline (current constants, {BASELINE_ORGANISM_COUNT} extra founders) ==");
    let baseline: Vec<RunResult> = BASELINE_SEEDS
        .iter()
        .map(|&seed| run(seed, BASELINE_ORGANISM_COUNT))
        .inspect(report)
        .collect();

    println!("\n== control (single founder, no producer to feed it) ==");
    let control: Vec<RunResult> = CONTROL_SEEDS
        .iter()
        .map(|&seed| run(seed, CONTROL_ORGANISM_COUNT))
        .inspect(report)
        .collect();

    let tally =
        |runs: &[RunResult], verdict: Verdict| runs.iter().filter(|r| r.verdict == verdict).count();
    let control_all_collapse = control.iter().all(|r| r.verdict == Verdict::Collapse);
    println!(
        "\nbaseline: {} breathes, {} thins, {} boil, {} collapse (of {})   control all collapse: {control_all_collapse}",
        tally(&baseline, Verdict::Breathes),
        tally(&baseline, Verdict::Thins),
        tally(&baseline, Verdict::Boil),
        tally(&baseline, Verdict::Collapse),
        baseline.len(),
    );
    // TD2b's wall proof: any nonzero count here is a body standing where
    // `Ground::grow` never laid terrain.
    let max_outside = baseline
        .iter()
        .flat_map(|r| r.samples.iter())
        .map(|s| s.outside)
        .max()
        .unwrap_or(0);
    println!("max escapees observed across baseline (any sample, any seed): {max_outside}");

    let path = receipt_path();
    let json = render_json(&baseline, &control);
    fs::create_dir_all(path.parent().expect("receipt path has a parent")).expect(
        "Code/testing/mesocosm already exists in this workspace; create_dir_all is just insurance",
    );
    fs::write(&path, json).expect("writing the receipt");
    println!("\nwrote {}", path.display());
}

fn report(run: &RunResult) {
    let start = run.samples.first().expect("tick-0 sample");
    let end = run.samples.last().expect("final sample");
    let peak = run
        .samples
        .iter()
        .max_by_key(|s| s.total_alive())
        .expect("at least one sample");
    println!(
        "  seed {:>3}: {:<9} start={:<5} peak={:<5} (tick {:<5}) end={:<5} (tick {:<5}) born={:<5} died={:<5} [{} ms]",
        run.seed,
        run.verdict.label(),
        start.total_alive(),
        peak.total_alive(),
        peak.tick,
        end.total_alive(),
        run.decided_tick,
        end.cum_born,
        end.cum_died,
        run.elapsed_ms,
    );
    // Kingdoms at both ends, because a total that "breathes" while one
    // kingdom is flat at zero is not the trophic balance TD2 is after.
    println!(
        "            kingdoms P/C/D start {}/{}/{} -> end {}/{}/{}; fullest cell {} -> {}; span {} -> {}; outside {} -> {}; end biomass {} mg",
        start.alive[0],
        start.alive[1],
        start.alive[2],
        end.alive[0],
        end.alive[1],
        end.alive[2],
        start.max_cell,
        end.max_cell,
        start.span,
        end.span,
        start.outside,
        end.outside,
        end.total_biomass_mg,
    );
    // The closed cycle's own receipt: soil against bodies, and a total that
    // must not move. (2026-08-29 TD6)
    println!(
        "            matter: soil {} -> {} mg; total {} -> {} mg{}",
        start.soil_mg,
        end.soil_mg,
        start.total_matter_mg,
        end.total_matter_mg,
        if start.total_matter_mg == end.total_matter_mg {
            " (conserved)"
        } else {
            " (NOT CONSERVED)"
        },
    );
    println!("            {}", run.reason);
}

/// Finds `Code/testing/mesocosm/td10_chain.json` by walking up from
/// this crate to the `repos` ancestor documented in `Code/CLAUDE.md`'s layout
/// section, rather than counting `../` — the crate's depth under `repos/`
/// is not this example's business to hardcode. Each round's receipt keeps its
/// own filename and none is overwritten: `td1_population.json`,
/// `td2_retune.json`, `td2b_walls.json`, `td2c_persistence.json`,
/// `td2d_scavengers.json`, `td5_economy.json`, `td5b_midlife.json`,
/// `td6_matter.json`, `td7_priced.json`, `s1_wide_instrument.json`, and now
/// this. (2026-08-29 TD8)
fn receipt_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repos_ancestor = manifest_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|name| name == "repos"))
        .expect("mesocosm-core is checked out under a `repos/` directory, per Code/CLAUDE.md");
    let workspace_root = repos_ancestor
        .parent()
        .expect("`repos/` has a parent — the `Code` workspace root");
    workspace_root.join("testing/mesocosm/td10_chain.json")
}

fn render_json(baseline: &[RunResult], control: &[RunResult]) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"ticks\": {TICKS},\n"));
    out.push_str(&format!("  \"sample_interval\": {SAMPLE_INTERVAL},\n"));
    out.push_str(&format!("  \"boil_multiple\": {BOIL_MULTIPLE},\n"));
    out.push_str(&format!("  \"collapse_fraction\": {COLLAPSE_FRACTION},\n"));
    out.push_str("  \"baseline\": ");
    render_batch(&mut out, baseline, 2);
    out.push_str(",\n  \"control\": ");
    render_batch(&mut out, control, 2);
    out.push('\n');
    out.push_str("}\n");
    out
}

fn render_batch(out: &mut String, runs: &[RunResult], indent: usize) {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    out.push_str("[\n");
    for (index, run) in runs.iter().enumerate() {
        out.push_str(&inner);
        render_run(out, run, indent + 2);
        if index + 1 < runs.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&pad);
    out.push(']');
}

fn render_run(out: &mut String, run: &RunResult, indent: usize) {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    out.push_str("{\n");
    out.push_str(&format!("{inner}\"seed\": {},\n", run.seed));
    out.push_str(&format!("{inner}\"founders\": {},\n", run.founders));
    out.push_str(&format!(
        "{inner}\"verdict\": \"{}\",\n",
        run.verdict.label()
    ));
    out.push_str(&format!(
        "{inner}\"reason\": \"{}\",\n",
        json_escape(&run.reason)
    ));
    out.push_str(&format!("{inner}\"decided_tick\": {},\n", run.decided_tick));
    out.push_str(&format!("{inner}\"elapsed_ms\": {},\n", run.elapsed_ms));
    out.push_str(&format!("{inner}\"samples\": [\n"));
    let sample_indent = " ".repeat(indent + 4);
    for (index, sample) in run.samples.iter().enumerate() {
        out.push_str(&sample_indent);
        out.push_str(&format!(
            "{{\"tick\": {}, \"producer\": {}, \"consumer\": {}, \"decomposer\": {}, \"total_biomass_mg\": {}, \"cum_born\": {}, \"cum_died\": {}, \"max_cell\": {}, \"span\": {}, \"outside\": {}, \"soil_mg\": {}, \"total_matter_mg\": {}}}",
            sample.tick,
            sample.alive[0],
            sample.alive[1],
            sample.alive[2],
            sample.total_biomass_mg,
            sample.cum_born,
            sample.cum_died,
            sample.max_cell,
            sample.span,
            sample.outside,
            sample.soil_mg,
            sample.total_matter_mg,
        ));
        if index + 1 < run.samples.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&inner);
    out.push_str("]\n");
    out.push_str(&pad);
    out.push('}');
}

/// Minimal JSON string escaping. Every string this instrument writes is
/// built from `format!` over its own labels and numbers, so the only
/// character that plausibly needs escaping is the `"` inside a `reason`
/// message; this covers that plus the standard control cases rather than
/// assuming it never happens.
fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
