// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The ecology lab: run the adaptation phase for a while and watch.
//!
//! Wave 2.2 asks for a headless model of the epoch loop's turn structure. This
//! is the watching half — the tests assert the rules hold, and this shows what
//! the rules *do* over a dozen rounds in three different worlds.
//!
//! ```text
//! cargo run -p mesocosm-core --example ecology_lab
//! cargo run -p mesocosm-core --example ecology_lab -- 30 7
//! ```
//!
//! Arguments are rounds and seed. Everything is deterministic, so a seed that
//! produces something interesting can be reported and re-run exactly.

use mesocosm_core::{
    Rng,
    epoch::{AUTHORED, Lineage, Role, Standing, Trait, WorldProfile, adapt_round, fitness},
};

/// What each lineage banks per round. Flat here: the epoch half is what
/// earns this, and the lab is about the adaptation half.
const INCOME: i32 = 12;

fn main() {
    let mut args = std::env::args().skip(1);
    let rounds: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(14);
    let seed: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1);

    for world in AUTHORED {
        run(world, rounds, seed);
    }
}

fn run(world: &WorldProfile, rounds: u32, seed: u64) {
    println!("\n=== {} ===", world.name);
    println!("    {}", world.question);
    print!("    pressures:");
    for force in world.forces {
        print!(" {}={}", force.pressure.name(), force.strength);
    }
    println!("\n");

    let mut roster = founders();
    let mut rng = Rng::from_seed(seed);

    for round in 1..=rounds {
        for lineage in roster.iter_mut() {
            if !lineage.extinct {
                lineage.bank += INCOME;
            }
        }

        let record = adapt_round(world, &mut roster, &mut rng);

        let moved = record.changes().count();
        if moved > 0 || !record.extinctions.is_empty() {
            println!(
                "round {round:>2}: {moved} of {} adapted",
                record.decisions.len()
            );
            for decision in record.changes() {
                let who = roster
                    .iter()
                    .find(|l| l.id == decision.lineage)
                    .map(|l| l.name.as_str())
                    .unwrap_or("?");
                println!(
                    "          {who:<10} {:<28} +{}",
                    describe(decision.chosen),
                    decision.gain()
                );
            }
            for id in &record.extinctions {
                let who = roster.iter().find(|l| l.id == *id).map(|l| l.name.as_str());
                println!("          {} went extinct", who.unwrap_or("?"));
            }
        }
    }

    println!("\n    final standing:");
    let snapshot = roster.clone();
    let standing = Standing::new(world, &snapshot);
    let mut ranked: Vec<&Lineage> = roster.iter().collect();
    ranked.sort_by_key(|l| (l.extinct, -l.complexity()));

    for lineage in ranked {
        let state = if lineage.extinct { "extinct" } else { "alive" };
        println!(
            "      {:<10} {:<8} complexity {:>3}  fitness {:>5}  {}",
            lineage.name,
            state,
            lineage.complexity(),
            fitness(lineage, &standing),
            strongest(lineage),
        );
    }
    println!("    frontier: {}", standing.frontier());
}

/// A founding assemblage: one of each trophic role plus two competitors, so
/// crowding and predation both have something to bite on.
fn founders() -> Vec<Lineage> {
    vec![
        Lineage::new(1, "moss", Role::Producer, [1, 1, 1, 0, 0, 2, 0]),
        Lineage::new(2, "frond", Role::Producer, [2, 1, 1, 0, 1, 1, 0]),
        Lineage::new(3, "grazer", Role::Consumer, [1, 1, 0, 1, 0, 2, 1]),
        Lineage::new(4, "stalker", Role::Consumer, [2, 1, 0, 2, 1, 0, 3]),
        Lineage::new(5, "rot", Role::Decomposer, [0, 1, 1, 0, 1, 2, 0]),
    ]
}

fn describe(mutation: Option<mesocosm_core::epoch::Mutation>) -> String {
    use mesocosm_core::epoch::Mutation;
    match mutation {
        Some(Mutation::Gain { trait_ }) => format!("+{}", trait_.name()),
        Some(Mutation::Swap { from, to }) => format!("{} -> {}", from.name(), to.name()),
        None => "stood pat".into(),
    }
}

/// The two traits a lineage has leaned into hardest. What it has become.
fn strongest(lineage: &Lineage) -> String {
    let mut ranked: Vec<(Trait, i32)> =
        Trait::ALL.iter().map(|t| (*t, lineage.level(*t))).collect();
    ranked.sort_by_key(|(t, level)| (-level, *t));
    ranked
        .iter()
        .take(2)
        .filter(|(_, level)| *level > 0)
        .map(|(t, level)| format!("{} {}", t.name(), level))
        .collect::<Vec<_>>()
        .join(", ")
}
