// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! TD1: the headless population instrument.
//!
//! Founds real worlds and drives them with nothing but `Intent::Idle` for
//! several starter lifespans, sampling per-kingdom alive counts, total
//! biomass, and cumulative births/deaths every `SAMPLE_INTERVAL` ticks. A
//! verdict is read off the curve by simple, stated arithmetic: no fitted
//! models, no significance tests. See `design_docs/2026-08-29_terrarium_dynamics_plan.md`, TD1.
//!
//! Two batches run: a **baseline** at today's constants (the 2026-08-29
//! playtest's single played run boiled, 61 -> 8,155; this instrument checks
//! whether that holds across seeds rather than assuming it), and a
//! **control** founded with a single organism and no producer to feed it
//! (expected to collapse). The control is what proves the verdict logic can
//! say something other than "boil" — an instrument that only ever reads one
//! way has not shown anything.
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
//! Writes `Code/testing/mesocosm/td1_population.json` (curves + verdicts per
//! seed) and prints a terminal summary. `Code/testing/<repo>/` is this
//! workspace's standing receipts convention; the path is found by walking up
//! from this crate to the `repos` ancestor rather than a fixed `../` count,
//! so it survives the crate moving depth within the repo.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use mesocosm_core::{Intent, Kingdom, World};

/// The starter mass every `World::new` founds its played critter with.
/// `lifespan_for_mass(1000mg)` is 1,000 ticks at today's constants (verified
/// against `ecology.rs`'s quarter-power tables), so this many ticks is ten
/// starter lifespans.
const TICKS: u32 = 10_000;
/// Sample cadence. 100 samples over the run is enough to see the shape of
/// the curve without writing a receipt per tick.
const SAMPLE_INTERVAL: u32 = 100;
/// Five arbitrary seeds. Nothing about them is tuned; the point is that the
/// verdict holds across an uncherrypicked handful, not one lucky draw.
const BASELINE_SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
/// Same five seeds, reused for the control so a reader can see the same
/// draws produce opposite verdicts under a different founding.
const CONTROL_SEEDS: [u64; 5] = BASELINE_SEEDS;

/// Founders beyond the played critter. 60 extra plus the always-present
/// played critter is 61 founders — the same starting count the 2026-08-29
/// playtest read as "61 -> 8,155".
const BASELINE_ORGANISM_COUNT: u32 = 60;
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

#[derive(Clone, Copy)]
struct Sample {
    tick: u32,
    alive: [u32; 3],
    total_biomass_mg: u64,
    cum_born: u64,
    cum_died: u64,
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
    Breathes,
}

impl Verdict {
    fn label(self) -> &'static str {
        match self {
            Verdict::Collapse => "collapse",
            Verdict::Boil => "boil",
            Verdict::Breathes => "breathes",
        }
    }
}

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
    for organism in world.organisms.iter().filter(|o| o.is_alive()) {
        alive[kingdom_index(organism.kingdom())] += 1;
        total_biomass_mg += organism.biomass_mg();
    }
    Sample {
        tick,
        alive,
        total_biomass_mg,
        cum_born,
        cum_died,
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

    (
        Verdict::Breathes,
        format!(
            "end population {end_total} stayed above the collapse floor ({collapse_floor}) and below or falling from the boil bound ({boil_bound})"
        ),
    )
}

fn main() {
    println!("TD1 population instrument: {TICKS} ticks, sampling every {SAMPLE_INTERVAL}");
    println!(
        "verdict arithmetic: collapse when end < start and end <= start/{COLLAPSE_FRACTION}; boil >= {BOIL_MULTIPLE}x start and still rising; else breathes\n"
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

    let baseline_all_boil = baseline.iter().all(|r| r.verdict == Verdict::Boil);
    let control_all_collapse = control.iter().all(|r| r.verdict == Verdict::Collapse);
    println!(
        "\nbaseline all boil: {baseline_all_boil}   control all collapse: {control_all_collapse}"
    );

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
        "  seed {:>3}: {:<9} start={:<6} peak={:<6} (tick {:<6}) end={:<6} (tick {:<6}) born={:<6} died={:<6} [{} ms]",
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
    println!("            {}", run.reason);
}

/// Finds `Code/testing/mesocosm/td1_population.json` by walking up from this
/// crate to the `repos` ancestor documented in `Code/CLAUDE.md`'s layout
/// section, rather than counting `../` — the crate's depth under `repos/`
/// is not this example's business to hardcode.
fn receipt_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repos_ancestor = manifest_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|name| name == "repos"))
        .expect("mesocosm-core is checked out under a `repos/` directory, per Code/CLAUDE.md");
    let workspace_root = repos_ancestor
        .parent()
        .expect("`repos/` has a parent — the `Code` workspace root");
    workspace_root.join("testing/mesocosm/td1_population.json")
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
            "{{\"tick\": {}, \"producer\": {}, \"consumer\": {}, \"decomposer\": {}, \"total_biomass_mg\": {}, \"cum_born\": {}, \"cum_died\": {}}}",
            sample.tick,
            sample.alive[0],
            sample.alive[1],
            sample.alive[2],
            sample.total_biomass_mg,
            sample.cum_born,
            sample.cum_died,
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
