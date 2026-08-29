// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! TD8's attribution probe: one run, three targets, one per ruling.
//!
//! The population instrument reads a *verdict*; this reads whether each ruling
//! moved the thing it was aimed at. A round that only moves the total has not
//! shown its rulings did anything, so each of the three gets its own numbers:
//!
//! 1. **Reproduction gates on adult mass** was aimed at **recruitment** —
//!    consumers and decomposers reaching a breeding size. Reported as births
//!    and deaths per kingdom, and the mean of `biomass_mg / mass_ceiling_mg`
//!    over the living, which is TD7's finding table re-measured.
//! 2. **Corpses persist longer** was aimed at **decomposer persistence** —
//!    carrion as a standing resource rather than an event. Reported as the mean
//!    standing carrion count, total scavenged milligrams, decomposers alive at
//!    the horizon, and how long a dying decomposer had gone unfed.
//! 3. **No actuator, no travel** was aimed at the **free-lunch species** — a
//!    body with no `Limb` tagma that grazed at a plant's price. Reported as the
//!    founding count of unlimbed consumers and decomposers, the `Moved` events
//!    they emitted, what they ate, and whether they were still standing at the
//!    horizon.
//!
//! ```text
//! cargo run -p mesocosm-core --example td8_attribution --release
//! ```
//!
//! Writes `Code/testing/mesocosm/td8_attribution.json` and prints the same
//! numbers. Seeds and horizon are arguments so a before/after pair is one
//! command each; the defaults are TD7's own probe window (3,000 ticks) over the
//! seeds its finding table quoted plus seed 2, whose consumer species is the
//! free-lunch draw the third ruling names.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use mesocosm_core::{Event, Intent, Kingdom, MealKind, OrganismId, Stage, World};

/// TD7's probe window, kept so the two rounds' tables compare directly.
const TICKS: u32 = 3_000;
/// Mirrors `ecology::rates::STARVATION_MG`, which is crate-private. The same
/// mirroring the population instrument does for `CROWD_CELL`, and for the same
/// reason: a receipt reads the rule, it does not get to change it.
const STARVATION_MG: u64 = 20;
/// Seeds 1 and 5 are the pair TD7's recruitment table quoted; 2 is the seed
/// whose consumer species draws an unlimbed recipe.
const SEEDS: [u64; 3] = [1, 2, 5];

const KINGDOMS: [&str; 3] = ["producers", "consumers", "decomposers"];

fn kingdom_index(kingdom: Kingdom) -> usize {
    match kingdom {
        Kingdom::Producer => 0,
        Kingdom::Consumer => 1,
        Kingdom::Decomposer => 2,
    }
}

/// Everything one seeded run says about the three targets.
struct Reading {
    seed: u64,
    // Ruling 1: recruitment.
    born: [u64; 3],
    died: [u64; 3],
    /// Deaths split by which of the ecology's two exits took the body: it was
    /// at or under `STARVATION_MG` when it died, or it had simply run out of
    /// lifespan. A kingdom dying of old age is a different failure from a
    /// kingdom dying hungry, and the difference is not in the verdict.
    died_starved: [u64; 3],
    died_aged: [u64; 3],
    alive_end: [u64; 3],
    /// Mean of `biomass_mg * 100 / mass_ceiling_mg` over the living, per
    /// kingdom, sampled at the horizon. TD7 measured 0.32 / 0.23 / 0.00 as
    /// fractions; this is the same number in percent.
    adult_pct_end: [u64; 3],
    /// The same reading averaged over every sample, so a kingdom that is
    /// extinct at the horizon still reports how it lived.
    adult_pct_mean: [u64; 3],
    /// Living bodies that pass `can_reproduce` at the horizon.
    breeding_end: [u64; 3],
    /// Meals taken, and body-ticks lived, per kingdom. The ratio is the **prey
    /// hit rate** TD2c named as the number a grazer's viability turns on: at
    /// `GRAZES_BASE_MG` 3 a fed tick nets roughly double its rent, so a grazer
    /// needs to eat on something like a third to a half of its ticks. This is
    /// what says whether a starving consumer is short of food or short of
    /// reach.
    fed_events: [u64; 3],
    /// Milligrams those meals actually delivered. `fed_mg / fed_events` is the
    /// **mouthful**, which is the other half of the viability question: a body
    /// that reaches food on most of its ticks and starves anyway is being paid
    /// too little per bite, not failing to find one.
    fed_mg: [u64; 3],
    alive_ticks: [u64; 3],
    // Ruling 2: decomposer persistence.
    /// Mean standing carrion count across the samples. TD7 read 12-15.
    carrion_mean: u64,
    carrion_end: u64,
    scavenged_mg: u64,
    /// Ticks since its last meal, averaged over decomposers at death.
    decomposer_fast_at_death: u64,
    decomposer_deaths: u64,
    // Ruling 3: the free-lunch species.
    /// Consumers and decomposers founded with `actuator_span() == 0`.
    unlimbed_founders: u64,
    /// Of those, how many were still alive at the horizon.
    unlimbed_alive_end: u64,
    /// `Moved` events emitted by an unlimbed consumer or decomposer. Zero is
    /// what "no actuator, no travel" means in receipt terms.
    unlimbed_moves: u64,
    /// Every `Moved` event, by kingdom. A producer is unlimbed by construction,
    /// so this is where a ruling about actuators shows its whole reach.
    moves: [u64; 3],
    /// What those bodies ate, and what every other consumer/decomposer ate, so
    /// the free lunch can be seen being withdrawn rather than merely counted.
    unlimbed_fed_mg: u64,
    limbed_fed_mg: u64,
    /// Total living count at the horizon, so the probe's own window can be
    /// checked against the instrument's.
    alive_total_end: u64,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let ticks: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(TICKS);
    let seeds: Vec<u64> = {
        let rest: Vec<u64> = args.filter_map(|a| a.parse().ok()).collect();
        if rest.is_empty() {
            SEEDS.to_vec()
        } else {
            rest
        }
    };

    println!("TD8 attribution probe: {ticks} idle ticks, seeds {seeds:?}\n");
    let census = founding_census();
    println!("  founding census, seeds 1-10 (the free-lunch draw, before anything moves)");
    for (seed, unlimbed, consumers_decomposers) in &census {
        println!(
            "    seed {seed:>2}: {unlimbed} of {consumers_decomposers} consumers+decomposers have no actuator"
        );
    }
    println!();

    let readings: Vec<Reading> = seeds.iter().map(|&seed| run(seed, ticks)).collect();
    for reading in &readings {
        report(reading);
    }

    let path = receipt_path();
    fs::create_dir_all(path.parent().expect("receipt path has a parent"))
        .expect("Code/testing/mesocosm already exists; create_dir_all is insurance");
    fs::write(&path, render_json(ticks, &readings, &census)).expect("writing the receipt");
    println!("wrote {}", path.display());
}

/// The free-lunch draw at genesis across the instrument's own ten seeds:
/// consumers and decomposers whose recipe grew no contractile part. Cheap — a
/// `World::new` per seed and no ticks — so it can widen past the seeds the run
/// half of the probe can afford. (TD7 measured 22 of 160 and 20 of 50 at the
/// ±16 founding; S1's pyramid names a species by its tier, so the draw is now
/// all-or-nothing per species.)
fn founding_census() -> Vec<(u64, u64, u64)> {
    (1..=10)
        .map(|seed| {
            let world = World::new(seed, mesocosm_core::world::FOUNDERS);
            let fauna: Vec<_> = world
                .living()
                .filter(|o| o.kingdom() != Kingdom::Producer)
                .collect();
            let unlimbed = fauna.iter().filter(|o| o.actuator_span() == 0).count() as u64;
            (seed, unlimbed, fauna.len() as u64)
        })
        .collect()
}

fn run(seed: u64, ticks: u32) -> Reading {
    let mut world = World::new(seed, mesocosm_core::world::FOUNDERS);

    // The founding free-lunch draw, read before anything moves: a consumer or
    // decomposer whose recipe grew no contractile part at all.
    let unlimbed: Vec<OrganismId> = world
        .living()
        .filter(|o| o.kingdom() != Kingdom::Producer && o.actuator_span() == 0)
        .map(|o| o.id)
        .collect();
    let unlimbed_founders = unlimbed.len() as u64;
    // Genesis pushes a `Born` for every founder. Those are the world arriving,
    // not the world recruiting, so they are dropped before the count starts.
    world.drain_events();

    let mut born = [0u64; 3];
    let mut died = [0u64; 3];
    let mut died_starved = [0u64; 3];
    let mut died_aged = [0u64; 3];
    let mut scavenged_mg = 0u64;
    let mut carrion_total = 0u64;
    let mut carrion_samples = 0u64;
    let mut adult_pct_total = [0u64; 3];
    let mut adult_pct_samples = [0u64; 3];
    let mut unlimbed_moves = 0u64;
    let mut moves = [0u64; 3];
    let mut fed_events = [0u64; 3];
    let mut fed_mg = [0u64; 3];
    let mut alive_ticks = [0u64; 3];
    let mut unlimbed_fed_mg = 0u64;
    let mut limbed_fed_mg = 0u64;
    let mut decomposer_fast_total = 0u64;
    let mut decomposer_deaths = 0u64;

    // Kingdom by id, so an event about a body that has since been retained out
    // of the world can still be attributed. Filled at genesis and at birth.
    let mut kingdom_of: BTreeMap<u32, usize> = world
        .living()
        .map(|o| (o.id.0, kingdom_index(o.kingdom())))
        .collect();
    let unlimbed_ids: std::collections::BTreeSet<u32> = unlimbed.iter().map(|id| id.0).collect();
    let mut last_meal: BTreeMap<u32, u32> = BTreeMap::new();

    for tick in 1..=ticks {
        world.apply(Intent::Idle);
        // Drained rather than read: `pending` accumulates until someone takes
        // it, so reading it every tick would count every event again.
        let events = world.drain_events();
        for event in &events {
            match event {
                Event::Born { organism, .. } => {
                    // A newborn's kingdom is its parent's species' — read off
                    // the body itself, which is in the world by now.
                    if let Some(o) = world.organisms.iter().find(|o| o.id == *organism) {
                        let k = kingdom_index(o.kingdom());
                        kingdom_of.insert(organism.0, k);
                        born[k] += 1;
                    }
                }
                Event::Died { organism, .. } => {
                    let k = kingdom_of.get(&organism.0).copied().unwrap_or(0);
                    died[k] += 1;
                    // The corpse is still in the roster this tick, carrying the
                    // mass it died holding — the same reading the ecology's own
                    // starvation clause makes.
                    let hungry = world
                        .organisms
                        .iter()
                        .find(|o| o.id == *organism)
                        .is_some_and(|o| o.biomass_mg() <= STARVATION_MG);
                    if hungry {
                        died_starved[k] += 1;
                    } else {
                        died_aged[k] += 1;
                    }
                    if k == 2 {
                        decomposer_deaths += 1;
                        let last = last_meal.get(&organism.0).copied().unwrap_or(0);
                        decomposer_fast_total += u64::from(tick - last);
                    }
                }
                Event::Fed {
                    eater,
                    mass_mg,
                    kind,
                    ..
                } => {
                    last_meal.insert(eater.0, tick);
                    let eater_kingdom = kingdom_of.get(&eater.0).copied().unwrap_or(0);
                    fed_events[eater_kingdom] += 1;
                    fed_mg[eater_kingdom] += mass_mg;
                    if *kind == MealKind::Scavenging {
                        scavenged_mg += mass_mg;
                    }
                    if kingdom_of.get(&eater.0).copied().unwrap_or(0) != 0 {
                        if unlimbed_ids.contains(&eater.0) {
                            unlimbed_fed_mg += mass_mg;
                        } else {
                            limbed_fed_mg += mass_mg;
                        }
                    }
                }
                Event::Moved { organism, .. } => {
                    if unlimbed_ids.contains(&organism.0) {
                        unlimbed_moves += 1;
                    }
                    moves[kingdom_of.get(&organism.0).copied().unwrap_or(0)] += 1;
                }
                _ => {}
            }
        }

        for organism in world.living() {
            alive_ticks[kingdom_index(organism.kingdom())] += 1;
        }

        // Sampled every hundred ticks, the instrument's own cadence.
        if tick.is_multiple_of(100) {
            carrion_total += world
                .organisms
                .iter()
                .filter(|o| o.stage == Stage::Carrion)
                .count() as u64;
            carrion_samples += 1;
            for (k, (total, samples)) in adult_pct_total
                .iter_mut()
                .zip(adult_pct_samples.iter_mut())
                .enumerate()
            {
                let (sum, count) = adult_share(&world, k);
                if let Some(mean) = sum.checked_div(count) {
                    *total += mean;
                    *samples += 1;
                }
            }
        }
    }

    let mut alive_end = [0u64; 3];
    let mut breeding_end = [0u64; 3];
    let mut adult_pct_end = [0u64; 3];
    for organism in world.living() {
        let k = kingdom_index(organism.kingdom());
        alive_end[k] += 1;
        if organism.can_reproduce() {
            breeding_end[k] += 1;
        }
    }
    for (k, slot) in adult_pct_end.iter_mut().enumerate() {
        let (sum, count) = adult_share(&world, k);
        *slot = sum.checked_div(count).unwrap_or(0);
    }

    Reading {
        seed,
        born,
        died,
        died_starved,
        died_aged,
        alive_end,
        adult_pct_end,
        adult_pct_mean: std::array::from_fn(|k| {
            adult_pct_total[k]
                .checked_div(adult_pct_samples[k])
                .unwrap_or(0)
        }),
        breeding_end,
        fed_events,
        fed_mg,
        alive_ticks,
        carrion_mean: carrion_total.checked_div(carrion_samples).unwrap_or(0),
        carrion_end: world
            .organisms
            .iter()
            .filter(|o| o.stage == Stage::Carrion)
            .count() as u64,
        scavenged_mg,
        decomposer_fast_at_death: decomposer_fast_total
            .checked_div(decomposer_deaths)
            .unwrap_or(0),
        decomposer_deaths,
        unlimbed_founders,
        unlimbed_alive_end: world
            .living()
            .filter(|o| unlimbed_ids.contains(&o.id.0))
            .count() as u64,
        unlimbed_moves,
        moves,
        unlimbed_fed_mg,
        limbed_fed_mg,
        alive_total_end: alive_end.iter().sum(),
    }
}

/// Sum of `biomass_mg * 100 / mass_ceiling_mg` over one kingdom's living, and
/// how many bodies that was. The ratio TD6 made meaningful and TD7 found sitting
/// at a fifth.
fn adult_share(world: &World, kingdom: usize) -> (u64, u64) {
    let mut sum = 0u64;
    let mut count = 0u64;
    for organism in world.living() {
        if kingdom_index(organism.kingdom()) != kingdom {
            continue;
        }
        sum += organism.biomass_mg() * 100 / organism.mass_ceiling_mg().max(1);
        count += 1;
    }
    (sum, count)
}

fn report(r: &Reading) {
    println!("  seed {}", r.seed);
    println!("    1. recruitment (the reproduction gate's target)");
    for (k, name) in KINGDOMS.iter().enumerate() {
        println!(
            "       {name:<12} born {:<6} died {:<6} (starved {:<5} aged {:<5}) alive_end {:<6} breeding_end {:<5} adult% end {:<4} mean {}",
            r.born[k],
            r.died[k],
            r.died_starved[k],
            r.died_aged[k],
            r.alive_end[k],
            r.breeding_end[k],
            r.adult_pct_end[k],
            r.adult_pct_mean[k],
        );
    }
    println!(
        "       prey hit rate (meals per body-tick lived) C {}% D {}%; mouthful C {} mg D {} mg",
        r.fed_events[1] * 100 / r.alive_ticks[1].max(1),
        r.fed_events[2] * 100 / r.alive_ticks[2].max(1),
        r.fed_mg[1] / r.fed_events[1].max(1),
        r.fed_mg[2] / r.fed_events[2].max(1),
    );
    println!("    2. decomposer persistence (the carrion ruling's target)");
    println!(
        "       standing carrion mean {} end {}; scavenged {} mg; decomposer deaths {} after a mean {} ticks unfed",
        r.carrion_mean,
        r.carrion_end,
        r.scavenged_mg,
        r.decomposer_deaths,
        r.decomposer_fast_at_death,
    );
    println!("    3. the free-lunch species (the dispersal floor's target)");
    println!(
        "       unlimbed consumers/decomposers founded {}, alive at horizon {}, Moved events {}, ate {} mg (limbed peers ate {} mg)",
        r.unlimbed_founders,
        r.unlimbed_alive_end,
        r.unlimbed_moves,
        r.unlimbed_fed_mg,
        r.limbed_fed_mg,
    );
    println!(
        "       every Moved event by kingdom P/C/D: {}/{}/{}",
        r.moves[0], r.moves[1], r.moves[2],
    );
    println!("       total alive at horizon {}\n", r.alive_total_end);
}

/// Beside every other round's receipt, found the same way the population
/// instrument finds its own.
fn receipt_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repos_ancestor = manifest_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|name| name == "repos"))
        .expect("mesocosm-core is checked out under a `repos/` directory, per Code/CLAUDE.md");
    let workspace_root = repos_ancestor
        .parent()
        .expect("`repos/` has a parent — the `Code` workspace root");
    workspace_root.join("testing/mesocosm/td8_attribution.json")
}

fn render_json(ticks: u32, readings: &[Reading], census: &[(u64, u64, u64)]) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"ticks\": {ticks},\n"));
    out.push_str("  \"founding_census\": [\n");
    for (index, (seed, unlimbed, fauna)) in census.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"seed\": {seed}, \"unlimbed\": {unlimbed}, \"consumers_decomposers\": {fauna}}}"
        ));
        if index + 1 < census.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");
    out.push_str("  \"runs\": [\n");
    for (index, r) in readings.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"seed\": {},\n", r.seed));
        out.push_str(&format!("      \"born\": {:?},\n", r.born));
        out.push_str(&format!("      \"died\": {:?},\n", r.died));
        out.push_str(&format!("      \"died_starved\": {:?},\n", r.died_starved));
        out.push_str(&format!("      \"died_aged\": {:?},\n", r.died_aged));
        out.push_str(&format!("      \"alive_end\": {:?},\n", r.alive_end));
        out.push_str(&format!("      \"breeding_end\": {:?},\n", r.breeding_end));
        out.push_str(&format!("      \"fed_events\": {:?},\n", r.fed_events));
        out.push_str(&format!("      \"alive_ticks\": {:?},\n", r.alive_ticks));
        out.push_str(&format!(
            "      \"adult_pct_end\": {:?},\n",
            r.adult_pct_end
        ));
        out.push_str(&format!(
            "      \"adult_pct_mean\": {:?},\n",
            r.adult_pct_mean
        ));
        out.push_str(&format!("      \"carrion_mean\": {},\n", r.carrion_mean));
        out.push_str(&format!("      \"carrion_end\": {},\n", r.carrion_end));
        out.push_str(&format!("      \"scavenged_mg\": {},\n", r.scavenged_mg));
        out.push_str(&format!(
            "      \"decomposer_deaths\": {},\n",
            r.decomposer_deaths
        ));
        out.push_str(&format!(
            "      \"decomposer_fast_at_death\": {},\n",
            r.decomposer_fast_at_death
        ));
        out.push_str(&format!(
            "      \"unlimbed_founders\": {},\n",
            r.unlimbed_founders
        ));
        out.push_str(&format!(
            "      \"unlimbed_alive_end\": {},\n",
            r.unlimbed_alive_end
        ));
        out.push_str(&format!(
            "      \"unlimbed_moves\": {},\n",
            r.unlimbed_moves
        ));
        out.push_str(&format!(
            "      \"unlimbed_fed_mg\": {},\n",
            r.unlimbed_fed_mg
        ));
        out.push_str(&format!("      \"limbed_fed_mg\": {},\n", r.limbed_fed_mg));
        out.push_str(&format!(
            "      \"alive_total_end\": {}\n",
            r.alive_total_end
        ));
        out.push_str("    }");
        if index + 1 < readings.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}
