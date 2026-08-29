// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The enclosure living on its own schedule.
//!
//! Split out of `organism.rs` on 2026-08-01, because that file had grown past
//! this repo's six-hundred-line ceiling and the rule is to split before adding
//! rather than after. The types live next door; this is what happens to them
//! every tick.
//!
//! Nothing here knows which organism is being played. That is not an oversight;
//! it is the wing's third law holding at the level of the simulation.

use crate::cohort;
use crate::development::PartPalette;
use crate::places::{Ground, Places, Soil, Tier, TierLine};
use crate::process::FeedingMode;
use crate::rng::Rng;
use crate::species::Lineages;

use crate::history::{Event, MealKind};
// The routing threshold lives with the played meal's rule in `world`, which is
// the point: TD5 makes it one rule rather than two that agree.
use crate::world::STARVED_UPKEEP_TICKS;

use super::{Organism, OrganismId, Stage, Tally};

mod breeding;
mod movement;
mod rates;

use movement::{
    CarrionTarget, LivingTarget, carrion_cells, choose_carrion_target, choose_living_target,
    disperse, living_cells,
};

pub(crate) use rates::*;
use rates::{CROWD_CELL, CROWD_COMFORT, UPKEEP_BASE_MG};

/// Which crowding cell a position falls in.
fn cell_of(position: [i32; 3]) -> (i32, i32) {
    (
        position[0].div_euclid(CROWD_CELL),
        position[2].div_euclid(CROWD_CELL),
    )
}

/// One bite: who reached for it, out of what, and how much they reached for.
///
/// Both sides are recorded by index and resolved in a pass of their own,
/// because the borrow that reads the pasture cannot also hold the eaten.
/// `mass_mg` is the mouthful attempted; what the prey could actually pay is
/// decided when the meal is settled.
struct Meal {
    eater: usize,
    prey: usize,
    mass_mg: u64,
    kind: MealKind,
}

/// Feeding income, routed by the body — **the same rule for every kingdom**.
///
/// TD4 gave the played meal one question: is this body inside
/// [`STARVED_UPKEEP_TICKS`] of empty? If so the meal burns, refilling the
/// budget; if not it builds. TD5 asks it of every organism instead, because
/// before this every non-played gain built biomass only and `energy_mg` was a
/// birth endowment that never refilled — so an NPC crossed every hunger
/// threshold within its first few hundred ticks and lived off its own body
/// thereafter, and a decomposer could never bank a corpse against the gap to
/// the next one.
///
/// Called after [`Organism::pay_upkeep`], so the reserve it reads is this
/// tick's, post-rent.
///
/// Returns what the body could not hold. TD6's ceilings bound both halves —
/// substance at the body plan's adult mass, reserve at the same number — so a
/// caller has to put the remainder back in the world rather than let it
/// evaporate. Callers clamp their draw to [`Organism::intake_room_mg`], so in
/// practice this returns zero; it returns it anyway because a leak here is a
/// leak nothing else would catch.
fn earn(organism: &mut Organism, mg: u64) -> u64 {
    let ceiling = organism.mass_ceiling_mg();
    if organism.budget_below(STARVED_UPKEEP_TICKS) {
        let banked = mg.min(ceiling.saturating_sub(organism.energy_mg));
        organism.energy_mg += banked;
        organism.gain_mass(mg - banked)
    } else {
        let spilled = organism.gain_mass(mg);
        let banked = spilled.min(ceiling.saturating_sub(organism.energy_mg));
        organism.energy_mg += banked;
        spilled - banked
    }
}

/// A body's remains go back to the ground where it lies: what it was still
/// carrying as reserve, released the moment it stops being able to hold it.
///
/// Substance is returned separately and slowly, by [`Stage::Carrion`] decay
/// and by whatever eats the corpse.
fn release_reserve(organism: &mut Organism, soil: &mut Soil) {
    let column = soil.column_at(organism.position);
    soil.deposit(column, organism.energy_mg);
    organism.energy_mg = 0;
}

/// Advances every organism one tick.
///
/// Deterministic: organisms are visited in id order, offspring are appended in
/// the order their parents produced them, and every random value comes from
/// the seeded stream.
/// One tick of the enclosure.
///
/// `events` collects what happened to *individuals*. [`Tally`] counts, which is
/// what a host shows; the events are what a history records, and significance
/// needs to know who rather than how many.
#[allow(clippy::too_many_arguments)]
pub fn step(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
) -> Tally {
    step_inner(
        organisms, next_id, rng, events, lineages, palette, soil, None, None, None, None,
    )
}

/// Advances the enclosure with place-graph ownership enabled. The plain
/// [`step`] entry point remains useful for isolated ecology fixtures; worlds
/// use this path so dispersal and tier hysteresis are part of replayed state.
#[allow(clippy::too_many_arguments)]
pub fn step_with_places(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
    places: &Places,
    focus: Option<[i32; 3]>,
) -> Tally {
    step_inner(
        organisms,
        next_id,
        rng,
        events,
        lineages,
        palette,
        soil,
        Some(places),
        None,
        focus,
        None,
    )
}

/// Advances a world-owned ecology against its voxel truth. Graph-only
/// fixtures keep using [`step_with_places`]; a replayed world calls this path
/// so embodied bodies obey the same footing and sight rules as the player.
///
/// `held` names the one organism a hand is currently on. It is the **only**
/// thing here that knows a player exists, and it is deliberately narrow: a
/// held body still ages, still pays rent, and still dies. All it is spared is
/// being walked somewhere it did not choose to go, so the keys and the
/// instincts never fight over the same body on the same tick. `None` — nobody
/// embodied, or a hand that has been still long enough to have let go — leaves
/// every organism on its own drives, which is the terrarium's resting state.
#[allow(clippy::too_many_arguments)]
pub fn step_with_ground(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
    places: &Places,
    ground: &Ground,
    focus: Option<[i32; 3]>,
    held: Option<OrganismId>,
) -> Tally {
    step_inner(
        organisms,
        next_id,
        rng,
        events,
        lineages,
        palette,
        soil,
        Some(places),
        Some(ground),
        focus,
        held,
    )
}

#[allow(clippy::too_many_arguments)]
fn step_inner(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
    places: Option<&Places>,
    ground: Option<&Ground>,
    focus: Option<[i32; 3]>,
    held: Option<OrganismId>,
) -> Tally {
    let mut tally = Tally::default();

    // Tier ownership is hysteretic state. Forming the far summary here makes
    // cohort conservation part of the tick's receipt rather than an
    // unaccounted host cache.
    if let (Some(places), Some(focus)) = (places, focus) {
        for organism in organisms.iter_mut().filter(|o| o.is_alive()) {
            let previous = organism.tier;
            organism.tier = TierLine::default().tick(places, previous, organism.position, focus);
            match (previous, organism.tier) {
                (Tier::Far, Tier::Near) => tally.promoted += 1,
                (Tier::Near, Tier::Far) => tally.demoted += 1,
                _ => {}
            }
        }
        let far = cohort::from_organisms(organisms, places);
        let (members, biomass, _) = cohort::conserved_totals(&far);
        tally.far_cohorts = far.len() as u32;
        tally.far_members = members as u32;
        tally.far_biomass_mg = biomass;
    }

    // Crowding, counted once per tick on a coarse grid. **TD6 asked whether the
    // soil made this redundant and measured the answer: no.** The soil bounds a
    // stand's *mass* — that is the fixed point the round was for — but a closed
    // matter budget says nothing about how many bodies that mass is divided
    // into, and a stand with crowding removed answered a finite enclosure by
    // subdividing: 620 producers and still climbing at the horizon, whatever
    // the soil was seeded with. Density is the job crowding still does; it just
    // is no longer the only regulator, and it no longer needs a rent floor
    // under it, because a shaded producer now simply draws less out of a
    // column instead of being handed a minimum from nowhere.
    let mut density: std::collections::BTreeMap<(i32, i32), u32> =
        std::collections::BTreeMap::new();
    for organism in organisms.iter().filter(|o| o.is_alive()) {
        *density.entry(cell_of(organism.position)).or_default() += 1;
    }

    // Carrion positions are read before anything changes, so decomposers all
    // see the same world within a tick rather than racing each other.
    let carrion: Vec<CarrionTarget> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.stage == Stage::Carrion)
        .map(|(index, o)| CarrionTarget {
            position: o.position,
            organism_index: index,
            shape: o.walker_shape(),
        })
        .collect();
    let carrion_cells = carrion_cells(&carrion);

    // Living bodies, read before anything changes, so feeding decisions all
    // see the same enclosure within a tick rather than racing each other.
    let living: Vec<LivingTarget> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_alive())
        .map(|(index, o)| LivingTarget {
            id: o.id,
            position: o.position,
            organism_index: index,
            kingdom: o.kingdom(),
            mass_mg: o.biomass_mg(),
            signal: o.signal,
            shape: o.walker_shape(),
        })
        .collect();
    let living_cells = living_cells(&living);

    let mut meals: Vec<Meal> = Vec::new();
    let mut newborns: Vec<Organism> = Vec::new();

    // **Rent, then the bite each body reaches for.** Nothing is eaten in this
    // pass: resolving a meal moves mass out of one body and into another, and
    // this borrow can only hold one of them. Producers are the exception and
    // settle here, because their counterparty is the ground rather than a
    // second body.
    for (index, organism) in organisms.iter_mut().enumerate() {
        organism.age = organism.age.saturating_add(1);
        if !organism.is_alive() {
            continue;
        }
        let column = soil.column_at(organism.position);

        // Everything alive pays rent, every tick, **and the rent scales with
        // the body**. Budget first, then the body itself: a creature with
        // nothing left to spend eats itself. Either way the milligrams go back
        // to the ground it is standing on — a body that burns itself has to
        // put that mass somewhere, and that somewhere is the cycle. (TD6)
        let owed = organism.upkeep_mg();
        let unpaid = organism.pay_upkeep();
        soil.deposit(column, owed - unpaid);

        // Nothing draws more than it can hold: past the body plan's adult mass
        // and a full reserve, more matter would only be handed straight back.
        // (TD6 determinate growth)
        let room = organism.intake_room_mg();

        match organism.feeding_mode() {
            // Producers make biomass out of the ground they stand on, and only
            // out of that. Income used to be a number the world minted; it is
            // now a withdrawal from a finite column, which is what makes
            // runaway growth impossible rather than merely discouraged. (TD6)
            FeedingMode::Producer => {
                let crowd = density
                    .get(&cell_of(organism.position))
                    .copied()
                    .unwrap_or(1)
                    .max(1);
                let income = producer_income_for_mass(organism.biomass_mg());
                // Floored at rent, as it has been since TD2. A shaded-out
                // producer stagnates rather than starving, because otherwise
                // a whole stand of identical plants crosses the starvation
                // line on the same tick and the patch goes extinct instead of
                // thinning. The floor is not a hand-out any more: it is a
                // *request*, and the column answers it or does not. (TD6)
                let want = (income * u64::from(CROWD_COMFORT) / u64::from(crowd))
                    .clamp(UPKEEP_BASE_MG, income)
                    .min(room);
                let drawn = soil.draw(column, want);
                let spilled = earn(organism, drawn);
                soil.deposit(column, spilled);
            }
            FeedingMode::Grazer | FeedingMode::Predator => {
                let amount = feeding_rate_for_mass(organism.biomass_mg()).min(room);
                if amount > 0
                    && let Some(prey) =
                        choose_living_target(organism, &living, &living_cells, ground)
                {
                    let kind = if organism.feeding_mode() == FeedingMode::Predator {
                        MealKind::Predation
                    } else {
                        MealKind::Grazing
                    };
                    meals.push(Meal {
                        eater: index,
                        prey,
                        mass_mg: amount,
                        kind,
                    });
                }
            }
            // Decomposers only earn where something has died.
            FeedingMode::Scavenger => {
                let amount = decay_rate_for_mass(organism.biomass_mg()).min(room);
                if amount > 0
                    && let Some(source) =
                        choose_carrion_target(organism, &carrion, &carrion_cells, ground)
                {
                    meals.push(Meal {
                        eater: index,
                        prey: source,
                        mass_mg: amount,
                        kind: MealKind::Scavenging,
                    });
                }
            }
        }
    }

    // **The meals, one at a time, both bodies in hand.** What the eater is
    // credited is exactly what came out of the prey — no more, because two
    // grazers can reach the same small producer and the second one finds less
    // than it bit for. Crediting first and reconciling afterwards is what
    // conjured matter here before, and there is no reconciliation that is
    // always payable.
    for meal in &meals {
        let taken = meal.mass_mg - organisms[meal.prey].spend_mass(meal.mass_mg);
        if taken == 0 {
            continue;
        }
        let from = organisms[meal.prey].id;
        let column = soil.column_at(organisms[meal.eater].position);
        let eater = &mut organisms[meal.eater];
        let eater_id = eater.id;
        let spilled = earn(eater, taken);
        soil.deposit(column, spilled);
        events.push(Event::Fed {
            eater: eater_id,
            from,
            mass_mg: taken,
            kind: meal.kind,
        });
    }

    // **What the tick did to each body**, once every ledger has settled.
    for organism in organisms.iter_mut() {
        let column = soil.column_at(organism.position);
        match organism.stage {
            Stage::Juvenile | Stage::Mature => {
                // The hand's body is not driven. Everything above still
                // happened to it — rent, income, the meal it could reach —
                // and everything below still will; only locomotion is the
                // player's while the player is there. (TD4)
                if let Some(places) = places
                    && held != Some(organism.id)
                    && disperse(
                        organism,
                        places,
                        ground,
                        rng,
                        soil,
                        &living,
                        &living_cells,
                        &carrion,
                        &carrion_cells,
                        events,
                    )
                {
                    tally.moved += 1;
                }

                if organism.stage == Stage::Juvenile
                    && organism.age >= maturity_for_mass(organism.life_history_mass_mg())
                {
                    organism.stage = Stage::Mature;
                    events.push(Event::Matured {
                        organism: organism.id,
                    });
                    tally.matured += 1;
                }

                organism.since_offspring = organism.since_offspring.saturating_add(1);

                let starved = organism.biomass_mg() <= STARVATION_MG;
                let aged = organism.age >= lifespan_for_mass(organism.life_history_mass_mg());
                if organism.biomass_mg() == 0 {
                    // Eaten to nothing. There is no corpse to leave, so this
                    // skips carrion entirely and returns straight to the world.
                    organism.stage = Stage::Spent;
                    release_reserve(organism, soil);
                    events.push(Event::Returned {
                        organism: organism.id,
                    });
                    tally.returned += 1;
                } else if starved || aged {
                    organism.stage = Stage::Carrion;
                    release_reserve(organism, soil);
                    events.push(Event::Died {
                        organism: organism.id,
                        species: organism.species,
                    });
                    organism.since_offspring = 0;
                    tally.died += 1;
                }
            }

            Stage::Carrion => {
                // The dead return whether or not a decomposer is present, just
                // far more slowly. Locked matter is a real failure mode — and
                // since TD6 "return" is literal: the milligram goes into the
                // column the body is lying on.
                let unreturned = organism.spend_mass(1);
                soil.deposit(column, 1 - unreturned);
                if organism.biomass_mg() == 0 {
                    organism.stage = Stage::Spent;
                    events.push(Event::Returned {
                        organism: organism.id,
                    });
                    tally.returned += 1;
                }
            }

            Stage::Spent => {}
        }
    }

    breeding::breed(
        organisms,
        &mut newborns,
        next_id,
        rng,
        events,
        lineages,
        palette,
        ground,
        &mut tally,
    );

    organisms.extend(newborns);
    organisms.retain(|o| o.stage != Stage::Spent);

    // The ground settles, after everything the tick put into it. Pure
    // transport between columns: no matter is made or lost here.
    soil.percolate();
    tally
}

#[cfg(test)]
mod tests;
