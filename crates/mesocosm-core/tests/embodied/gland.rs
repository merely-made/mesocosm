// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD2: one native played process, in its four states.
//!
//! The gate's whole specification is four claims, and there is one test for
//! each of them here:
//!
//! 1. **acquiring or expressing it creates a readable choice** — nothing grows
//!    a gland, so having one means a development took tissue off what the part
//!    was already doing, and converting a whole frond costs a body its living;
//! 2. **allocation locates it on a part and charges its cost** — a named part,
//!    a counted number of cells, a milligram out of the reserve and into the
//!    ground, and a standing rent from then on;
//! 3. **world conditions can make it useful or dormant** — a gland makes its
//!    toxin out of the ground under the body, and goes dry where that ground
//!    cannot supply what it holds, without losing a cell or a milligram of
//!    rent;
//! 4. **severing its dependency removes the consequence** — and the branch can
//!    still say what it used to do.

use mesocosm_core::{
    Allocate, Attachment, CellId, Intent, Organism, OrganismId, Outcome, PartId, Placement,
    Process, ProcessRef, Provenance, Refusal, Registry, Rejection, Stage, VolumeRef, World, Yaw,
};

use super::bulk_world;

/// The acquired definition, addressed the way a phenotype addresses it.
fn gland() -> ProcessRef {
    Registry::native().of_native(Process::Secrete).reference()
}

fn fixing() -> ProcessRef {
    Registry::native().of_native(Process::Fix).reference()
}

/// A plate wide enough that its gland can outgrow the ground.
///
/// `[6, 4, 1]` classifies as `Role::Plate` and lattices to 4x3x1 = twelve
/// cells, each worth 23 mg of the part's own adult mass. That matters for the
/// dormancy test and nowhere else: a gland on more than four of those cells
/// holds more than a fresh soil column does, which is what lets one fixture
/// show a charged gland and a dry one without waiting for the enclosure to
/// draw itself down.
const FROND: [i32; 3] = [6, 4, 1];

/// A world whose played critter carries one frond held up in a canopy
/// position — so it reads as a producer, and carries the only shape that
/// admits a gland.
fn fronded_world(seed: u64) -> World {
    let mut world = bulk_world(seed, 24);
    frond_on(&mut world);
    world
}

fn frond_on(world: &mut World) -> PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                // Held up and clear of everything else on the body: the DC4
                // canopy position, which is what makes this a frond rather
                // than a shell.
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root")
}

/// The played critter's only frond.
fn frond_of(world: &World) -> PartId {
    let body = world.body().expect("embodied");
    body.living()
        .map(|part| part.id)
        .find(|part| {
            mesocosm_core::classify(body.part(*part).unwrap().half_extent)
                == mesocosm_core::Role::Plate
        })
        .expect("the fixture attached one")
}

/// Asks for a frond split between fixing and a gland of `cells` cells, taken
/// off the high end of the lattice.
fn split(world: &World, part: PartId, cells: u32) -> Intent {
    let capacity = world
        .phenotype()
        .expect("embodied")
        .mosaic(part)
        .expect("a living part carries a mosaic")
        .capacity();
    let kept: Vec<CellId> = (0..capacity - cells).map(|i| CellId(i as u16)).collect();
    let taken: Vec<CellId> = (capacity - cells..capacity)
        .map(|i| CellId(i as u16))
        .collect();
    let mut sites = Vec::new();
    if !kept.is_empty() {
        sites.push(Allocate {
            process: fixing(),
            cells: kept,
        });
    }
    sites.push(Allocate {
        process: gland(),
        cells: taken,
    });
    Intent::Rearrange { part, sites }
}

// ---------------------------------------------------------------------------
// 1. A readable choice
// ---------------------------------------------------------------------------

#[test]
fn no_body_a_world_founds_has_one() {
    // The property the whole gate rests on, asserted over a founded roster
    // rather than hoped for: geometry seeds four definitions and admits five,
    // so a gland is only ever somewhere a development put it. If this fails,
    // every ecology number below has moved for a reason nobody chose.
    let world = World::new(4_242, 24);
    for organism in &world.organisms {
        assert_eq!(
            organism.phenotype.secretory_mg(),
            0,
            "{:?} was founded with a gland",
            organism.id
        );
        assert!(organism.phenotype.glands().is_empty());
    }
    assert_eq!(World::new(4_242, 24).gland(), None);
}

#[test]
fn the_tissue_has_to_come_off_something() {
    // A seeded part arrives fully committed, so the first development that
    // wants a second process takes tissue from the first. That is the choice,
    // and it is visible as an exchange of cells rather than as an addition.
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    let before = world.phenotype().unwrap().mosaic(part).unwrap().free();
    assert_eq!(before, 0, "a seeded frond has nothing spare");

    assert!(matches!(
        world.apply(split(&world.clone(), part, 4)),
        Outcome::Rearranged { .. }
    ));
    let mosaic = world.phenotype().unwrap().mosaic(part).unwrap();
    assert_eq!(mosaic.free(), 0, "and it still has nothing spare");
    assert_eq!(mosaic.sites().len(), 2, "it does two things now");
    assert_eq!(world.gland().unwrap().cells, 4);
}

#[test]
fn converting_the_whole_frond_costs_the_body_its_living() {
    // The downside, and the reason PD1b left the canopy reading for this gate:
    // a plate held up in the light that is not allocated to fixing is not a
    // canopy, so a body that turned its whole frond into poison stops being a
    // producer and has to eat like everything else.
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    assert_eq!(
        world.controlled().unwrap().kingdom(),
        mesocosm_core::Kingdom::Producer,
        "the frond is what makes it one"
    );

    let capacity = world.phenotype().unwrap().mosaic(part).unwrap().capacity();
    let outcome = world.apply(split(&world.clone(), part, capacity));
    assert!(matches!(outcome, Outcome::Rearranged { .. }));

    assert!(!world.phenotype().unwrap().canopy());
    assert_ne!(
        world.controlled().unwrap().kingdom(),
        mesocosm_core::Kingdom::Producer,
        "it is still held up, and it is not fixing"
    );
    // The anatomy is untouched: the plate is still a plate, still in a canopy
    // position. What changed is what its tissue is doing.
    assert_eq!(world.body().unwrap().canopy_parts().count(), 1);
}

#[test]
fn a_part_still_cannot_acquire_a_capability_by_editing_a_number() {
    // The anti-Spore gate, over the acquired definition: admission is by
    // shape, so a gland cannot be put on bulk. To make a body secrete you
    // still have to give it the shape that secretes.
    let mut world = fronded_world(4_242);
    let root = world.body().unwrap().root;
    let outcome = world.apply(Intent::Rearrange {
        part: root,
        sites: vec![Allocate {
            process: gland(),
            cells: vec![CellId(0)],
        }],
    });
    assert!(
        matches!(
            outcome,
            Outcome::Rejected(Rejection::Refused(Refusal::SiteMismatch { .. }))
        ),
        "{outcome:?}"
    );
    assert_eq!(world.gland(), None, "and nothing was allocated");
}

#[test]
fn a_refusal_names_its_boundary_and_moves_nothing() {
    // The refusal contract survives the trip through an intent: PD1b made the
    // boundary the answer, and the world door carries it whole rather than
    // flattening fifteen named refusals into one word.
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    let before = world.phenotype().unwrap().clone();

    let outcome = world.apply(Intent::Rearrange {
        part,
        sites: vec![Allocate {
            process: gland(),
            // Not a connected region: an organ is a piece of tissue.
            cells: vec![CellId(0), CellId(2)],
        }],
    });
    assert!(
        matches!(
            outcome,
            Outcome::Rejected(Rejection::Refused(Refusal::Disconnected(_)))
        ),
        "{outcome:?}"
    );
    let unknown = world.apply(Intent::Rearrange {
        part,
        sites: vec![Allocate {
            process: ProcessRef {
                definition: mesocosm_core::DefinitionDigest(0xDEAD),
            },
            cells: vec![CellId(0)],
        }],
    });
    assert!(
        matches!(
            unknown,
            Outcome::Rejected(Rejection::Refused(Refusal::UnknownProcess(_)))
        ),
        "{unknown:?}"
    );
    assert_eq!(
        world.phenotype().unwrap().revision(),
        before.revision(),
        "a refused development moves no allocation"
    );
}

// ---------------------------------------------------------------------------
// 2. Located, and charged
// ---------------------------------------------------------------------------

#[test]
fn the_development_is_located_paid_for_and_on_the_record() {
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    let cell_mg = world.phenotype().unwrap().cell_mg(part);
    let matter_before = world.total_matter_mg();
    let energy_before = world.energy_mg().unwrap();

    let outcome = world.apply(split(&world.clone(), part, 4));
    let Outcome::Rearranged {
        part: on,
        cost_mg,
        revision,
    } = outcome
    else {
        panic!("{outcome:?}");
    };

    // Located: on a named part, and the reading says which.
    assert_eq!(on, part);
    assert_eq!(world.gland().unwrap().sites, vec![(part, 4)]);
    // Charged: the cells whose expression changed, priced in that part's own
    // tissue. Four cells changed hands, and nothing else on the part did.
    assert_eq!(cost_mg, 4 * cell_mg);
    assert!(energy_before - world.energy_mg().unwrap() >= cost_mg);
    assert_eq!(revision, 1, "and the rearrangement is ordered");

    // The milligram went into the ground, it did not evaporate. Ecology moves
    // matter between accounts on the same tick; the sum is what must not move.
    assert_eq!(
        world.total_matter_mg(),
        matter_before,
        "the development conserved matter"
    );
}

#[test]
fn carrying_a_gland_costs_rent_every_tick() {
    // The standing cost, and the reason a gland is a decision rather than a
    // free upgrade. Same body, same mass, same ceiling; the only difference is
    // what its tissue is doing.
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    let plain = world.controlled().unwrap().upkeep_mg();

    world.apply(split(&world.clone(), part, 4));
    let armed = world.controlled().unwrap().upkeep_mg();
    assert!(
        armed > plain,
        "rent {armed} should exceed the {plain} the same body paid without a gland"
    );
    assert_eq!(world.gland().unwrap().rent_mg, armed - plain);
}

// ---------------------------------------------------------------------------
// 3. Useful, and dormant
// ---------------------------------------------------------------------------

/// A neighbour the played critter can reach, with a gland of `cells` cells on
/// a frond of its own.
fn armed_neighbour(world: &mut World, cells: u32) -> OrganismId {
    let here = world.position().unwrap();
    let at = [here[0] + 1, here[1], here[2]];
    let id = OrganismId(9_200);
    let mut prey = Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            id,
            world.controlled().unwrap().species,
            mesocosm_core::Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            at,
            1_200,
        )
    };
    let root = prey.body().root;
    let part = prey
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches");
    if cells > 0 {
        let capacity = prey.phenotype.mosaic(part).unwrap().capacity();
        let proposal = mesocosm_core::AllocationProposal {
            expect: prey.phenotype.digest(),
            source: mesocosm_core::Arrangement::Direct,
            parts: vec![part],
            sites: vec![mesocosm_core::ProposedSite {
                part,
                process: gland(),
                cells: (capacity - cells..capacity)
                    .map(|i| CellId(i as u16))
                    .collect(),
            }],
        };
        prey.phenotype
            .develop(&proposal)
            .expect("a plate admits a gland");
    }
    world.organisms.push(prey);
    id
}

#[test]
fn a_charged_gland_costs_whatever_eats_the_body_that_carries_it() {
    // Useful: the same meal, twice, differing only in whether the prey had
    // turned some of a frond into poison. What the eater loses is exactly what
    // the gland held.
    let mut plain = fronded_world(4_242);
    let mut armed = plain.clone();
    let prey = armed_neighbour(&mut plain, 0);
    assert_eq!(armed_neighbour(&mut armed, 4), prey);
    let potency = armed
        .organisms
        .iter()
        .find(|o| o.id == prey)
        .unwrap()
        .phenotype
        .secretory_mg();
    assert!(potency > 0);

    let intent = Intent::Metabolize {
        organism: prey,
        placement: Placement::Planned,
    };
    plain.apply(intent.clone());
    armed.apply(intent);

    assert_eq!(
        plain.energy_mg().unwrap() - armed.energy_mg().unwrap(),
        potency,
        "the bite cost exactly what the gland held"
    );
    // And it went into the ground under the prey rather than nowhere.
    assert_eq!(plain.total_matter_mg(), armed.total_matter_mg());
}

/// Builds a gland of `cells` cells and then walks one column off the ground it
/// was built on.
///
/// **The step matters, and it is a finding rather than a workaround.** The
/// development's price is paid into the column under the body, and building a
/// gland from scratch costs exactly what the gland comes to hold — so a fresh
/// gland is always charged where it was made, by its own spoil. Dormancy is
/// therefore something a body walks into, which is the right shape for it:
/// carrying a big gland means keeping to rich ground.
fn armed_and_moved_on(world: &mut World, cells: u32) -> PartId {
    let part = frond_of(world);
    let snapshot = world.clone();
    world.apply(split(&snapshot, part, cells));
    world.apply(Intent::Move { delta: [2, 0, 0] });
    part
}

#[test]
fn a_gland_bigger_than_the_ground_is_dry_and_still_costs_its_rent() {
    // Dormant, and the whole of the rule: a gland makes its toxin out of the
    // column under the body, so it works only where that ground could replace
    // what the gland holds. Nothing about the allocation changes — the plan
    // forbids a changing environment from rewriting the mosaic — so the tissue
    // is still there, still committed, and still charged for.
    let mut world = fronded_world(4_242);
    armed_and_moved_on(&mut world, 5);

    let reading = world.gland().expect("it has one");
    assert!(
        reading.potency_mg > reading.ground_mg,
        "the fixture wants a gland the fresh column cannot supply: \
         {} mg against {} mg",
        reading.potency_mg,
        reading.ground_mg
    );
    assert!(!reading.charged, "so it is dry");
    assert_eq!(
        world.controlled().unwrap().bite_mg(reading.ground_mg),
        world.controlled().unwrap().venom_mg,
        "and a bite costs only what the line was born with"
    );
    assert!(reading.rent_mg > 0, "which it is paying for regardless");
    assert_eq!(reading.cells, 5, "and has lost no tissue");
}

#[test]
fn enriching_the_ground_charges_the_gland_the_body_already_had() {
    // The same claim from the other side, through a verb a player already has:
    // depositing into the column under you is what turns a dry gland on. The
    // allocation does not move, the revision does not move, and the body's
    // capability changes — which is exactly what "world conditions can make it
    // useful or dormant" has to mean if it is not to be a second biology.
    let mut world = fronded_world(4_242);
    armed_and_moved_on(&mut world, 5);
    let dry = world.gland().expect("it has one");
    assert!(!dry.charged);
    let revision = world.phenotype().unwrap().revision();

    let owed = dry.potency_mg - dry.ground_mg;
    world.apply(Intent::Deposit { mass_mg: owed + 8 });

    let charged = world.gland().expect("still has one");
    assert!(
        charged.charged,
        "{} mg of ground against {} mg of gland",
        charged.ground_mg, charged.potency_mg
    );
    assert_eq!(charged.cells, dry.cells, "no tissue moved");
    assert_eq!(charged.potency_mg, dry.potency_mg);
    assert_eq!(
        world.phenotype().unwrap().revision(),
        revision,
        "and no development happened"
    );
}

// ---------------------------------------------------------------------------
// 4. Severed, and gone
// ---------------------------------------------------------------------------

#[test]
fn severing_the_frond_takes_the_bite_and_the_rent_with_it() {
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    // A control that never had one, severed the same way: rent falls when a
    // part is lost whatever that part was doing, so the claim here is that
    // what is *left* is the same, not that nothing changed.
    let mut control = world.clone();
    world.apply(split(&world.clone(), part, 4));
    assert!(world.gland().unwrap().charged);

    for at in [&mut world, &mut control] {
        let me = at.controlled_id().unwrap();
        let organism = at.organisms.iter_mut().find(|o| o.id == me).unwrap();
        organism.phenotype.sever(part);
    }

    let after = world.gland().expect("the loss is still readable");
    assert!(after.sites.is_empty(), "nothing expresses it any more");
    assert_eq!(after.cells, 0);
    assert_eq!(after.potency_mg, 0);
    assert_eq!(after.rent_mg, 0, "and it stopped costing rent");
    assert_eq!(
        world.controlled().unwrap().upkeep_mg(),
        control.controlled().unwrap().upkeep_mg(),
        "a body that lost its gland pays what a body that never had one pays"
    );
    assert_eq!(
        world.controlled().unwrap().bite_mg(u64::MAX),
        world.controlled().unwrap().venom_mg,
        "the bite went with the branch"
    );

    // But the injury is still explainable: PD1b keeps a severed part's mosaic
    // addressable precisely so a player can be told what that branch used to
    // do, and PD2 is the first thing with something worth saying.
    assert_eq!(after.lost, vec![part]);
    let explanation = world
        .phenotype()
        .unwrap()
        .explain(part)
        .expect("the part is still addressable");
    assert!(!explanation.living);
    assert!(
        explanation
            .sites
            .iter()
            .any(|site| site.named.map(|id| id.name) == Some("secrete")),
        "the lost branch can still name what it did: {explanation:?}"
    );
}

#[test]
fn a_severed_frond_cannot_be_rearranged() {
    // The other half of severing: a branch that is gone is not somewhere a
    // development can put anything.
    let mut world = fronded_world(4_242);
    let part = frond_of(&world);
    let me = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .phenotype
        .sever(part);

    let outcome = world.apply(Intent::Rearrange {
        part,
        sites: vec![Allocate {
            process: gland(),
            cells: vec![CellId(0)],
        }],
    });
    assert!(
        matches!(
            outcome,
            Outcome::Rejected(Rejection::Refused(Refusal::SeveredPart(_)))
        ),
        "{outcome:?}"
    );
}
