// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The enclosure's matter, held per voxel column.
//!
//! TD6's closed cycle (ruled 2026-08-29). Producers draw matter from the
//! column they stand on, bodies return it where they fall, and the player's
//! deposit enriches it, so **total matter is conserved**: mass cannot run away
//! because it has to be somewhere. Light stays the one open input — a producer
//! spends free energy to do the drawing, and that energy never enters this
//! ledger.
//!
//! # Not the same thing as [`Ground`](super::Ground)
//!
//! Ground holds bricks: what is solid, what can be walked on, what a
//! projection draws. This holds milligrams: what can be *eaten out of* the
//! floor. A carve changes the first and not the second, which is why the
//! matter account and the terrain account are separate stores over the same
//! coordinates.
//!
//! # Why per voxel column
//!
//! Ruled on measured evidence (`Code/testing/mesocosm/soil_granularity_probe.md`,
//! 96 configs). At the shipping enclosure a direct index into a 4 KB array is
//! the *fastest* grain for point uptake — 1.75us against 5.66us for a
//! nearest-site scan — and it is the only grain that can express a forage
//! radius at all: a coarse grain's r=3 neighbourhood already covers the whole
//! world, so roots hunting minerals through soil cannot be represented in it
//! at any price.
//!
//! # The forage radius, built
//!
//! Addressing ([`Column`], [`Soil::column_at`], [`Soil::columns_within`]) is
//! separate from transfer ([`Soil::matter_mg`], [`Soil::draw`],
//! [`Soil::deposit`]), so TD7's root forage is exactly what that separation
//! promised: a read over `columns_within` followed by the same `draw` uptake
//! always took. See [`Soil::draw_richest_within`] and [`FORAGE_RADIUS`].

use serde::{Deserialize, Serialize};

/// Fraction of a column that percolates outward each tick, as a divisor.
///
/// **Measured, and the round's own structural finding put it here.** With
/// sealed columns, a producer drains the one it stands on within tens of ticks
/// and thereafter earns exactly the rent it just paid — net zero, forever —
/// while the other thousand columns keep their matter and no root can reach
/// it. The probe read 17 producers standing on 0 mg with 340,000 mg lying in
/// the enclosure around them; the whole chain starved above them. Percolation
/// is the medium's own property (dissolved minerals move through soil, which
/// is *why* a root that searches a radius finds more), so it is not the
/// foraging behaviour this round deliberately left unbuilt, and it is what
/// makes "the enclosure gets a finite matter budget" mean one budget.
const PERCOLATION_DIVISOR: u64 = 8;

/// How far a root searches for its next milligram, in voxel columns.
///
/// Three, because that is the reach the per-voxel grain was ruled for: 49 of
/// 1,089 columns here, where every coarser grain's r=3 already covered the
/// whole world and a forage radius could not be expressed at all. (TD7)
pub const FORAGE_RADIUS: i32 = 3;

/// One voxel column of the enclosure: a direct index into [`Soil`].
///
/// Opaque, and only [`Soil`] mints one, so an index can never address a
/// column a differently-sized store does not have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Column(u32);

/// Matter in the ground, one entry per voxel column of the enclosure.
///
/// World state: serialized, hashed, and deterministic. Row-major in z then x
/// over `-extent..=extent` on both axes, so the whole store is one contiguous
/// array and a lookup is arithmetic rather than a search.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Soil {
    extent: i32,
    /// Milligrams per column, in canonical index order.
    matter_mg: Vec<u64>,
}

impl Default for Soil {
    /// A single empty column. Not `derive`d: every operation here indexes, so
    /// the degenerate store still has to have somewhere to put a milligram.
    fn default() -> Self {
        Self::seeded(0, 0)
    }
}

impl Soil {
    /// A store over an enclosure of the given extent, every column holding
    /// `per_column_mg`.
    pub fn seeded(extent: i32, per_column_mg: u64) -> Self {
        let extent = extent.max(0);
        let side = (2 * extent + 1) as usize;
        Self {
            extent,
            matter_mg: vec![per_column_mg; side * side],
        }
    }

    /// How far the store reaches from the middle, in voxels. Sized from the
    /// enclosure it was raised over, never assumed.
    pub fn extent(&self) -> i32 {
        self.extent
    }

    /// Columns to a side.
    pub fn side(&self) -> i32 {
        2 * self.extent + 1
    }

    pub fn columns(&self) -> usize {
        self.matter_mg.len()
    }

    /// Which column a position stands over.
    ///
    /// **Clamped, not refused.** Everything deposited has to land somewhere or
    /// the cycle would leak at the wall; since TD2b nothing lives outside the
    /// enclosure anyway, so the clamp is insurance rather than a behaviour.
    pub fn column_at(&self, position: [i32; 3]) -> Column {
        let x = position[0].clamp(-self.extent, self.extent) + self.extent;
        let z = position[2].clamp(-self.extent, self.extent) + self.extent;
        Column((z * self.side() + x) as u32)
    }

    /// What one column holds.
    pub fn matter_mg(&self, column: Column) -> u64 {
        self.matter_mg
            .get(column.0 as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Takes up to `want_mg` out of one column, returning what was actually
    /// there to take. A column that is spent gives nothing, which is the whole
    /// point: a producer's income is limited by the ground under it.
    pub fn draw(&mut self, column: Column, want_mg: u64) -> u64 {
        let Some(held) = self.matter_mg.get_mut(column.0 as usize) else {
            return 0;
        };
        let drawn = want_mg.min(*held);
        *held -= drawn;
        drawn
    }

    /// Returns matter to one column. Decay, rent, and the player's deposit all
    /// land here.
    pub fn deposit(&mut self, column: Column, mg: u64) {
        if let Some(held) = self.matter_mg.get_mut(column.0 as usize) {
            *held = held.saturating_add(mg);
        }
    }

    /// Every milligram the ground is holding. One half of the conservation
    /// ledger; living bodies, carrion, and budgets are the other.
    pub fn total_mg(&self) -> u64 {
        self.matter_mg.iter().sum()
    }

    /// One tick of percolation: every column sheds a share of what it holds
    /// into the columns beside it.
    ///
    /// **Transport, not regrowth.** Nothing is created — a column's loss is
    /// exactly its neighbours' gain, in integers — so conservation is
    /// unaffected. What it buys is that the enclosure's matter budget is one
    /// budget rather than 1,089 sealed jars.
    ///
    /// See [`PERCOLATION_DIVISOR`] for why the round needed it.
    pub fn percolate(&mut self) {
        let side = self.side();
        let columns = self.matter_mg.len();
        let mut delta = vec![0i64; columns];
        for index in 0..columns {
            let (x, z) = (index as i32 % side, index as i32 / side);
            let neighbours: Vec<usize> = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                .iter()
                .map(|(dx, dz)| (x + dx, z + dz))
                .filter(|(nx, nz)| (0..side).contains(nx) && (0..side).contains(nz))
                .map(|(nx, nz)| (nz * side + nx) as usize)
                .collect();
            if neighbours.is_empty() {
                continue;
            }
            // Split so nothing is lost to integer truncation: the remainder
            // goes to the first neighbours in a fixed order rather than being
            // rounded away, which at these column sizes was the difference
            // between a flow and no flow at all.
            let out = self.matter_mg[index] / PERCOLATION_DIVISOR;
            if out == 0 {
                continue;
            }
            let share = out / neighbours.len() as u64;
            let extra = out % neighbours.len() as u64;
            delta[index] -= out as i64;
            for (rank, neighbour) in neighbours.into_iter().enumerate() {
                delta[neighbour] += (share + u64::from((rank as u64) < extra)) as i64;
            }
        }
        for (held, change) in self.matter_mg.iter_mut().zip(delta) {
            *held = held.saturating_add_signed(change);
        }
    }

    /// Takes up to `want_mg` out of the richest column within `radius`.
    ///
    /// **The reach is wide; the draw is not.** A root reads its whole
    /// neighbourhood and then takes the ordinary income out of the best column
    /// it found — at the speed of growth, on low-rent metabolism, never the
    /// radius' worth of columns at once. Ties go to the lowest column index, so
    /// a stand on flat ground forages the same way every replay.
    pub fn draw_richest_within(&mut self, column: Column, radius: i32, want_mg: u64) -> u64 {
        let richest = self
            .columns_within(column, radius)
            .max_by_key(|found| (self.matter_mg(*found), std::cmp::Reverse(found.0)))
            .unwrap_or(column);
        self.draw(richest, want_mg)
    }

    /// The columns within `radius` of one, in canonical order.
    ///
    /// The reach a root searches: [`Soil::draw_richest_within`] is this read
    /// followed by the same [`Soil::draw`] a point uptake always took, which
    /// is why the addressing was kept separate from the transfer.
    pub fn columns_within(&self, column: Column, radius: i32) -> impl Iterator<Item = Column> + '_ {
        let side = self.side();
        let (cx, cz) = (column.0 as i32 % side, column.0 as i32 / side);
        let radius = radius.max(0);
        ((cz - radius).max(0)..=(cz + radius).min(side - 1)).flat_map(move |z| {
            ((cx - radius).max(0)..=(cx + radius).min(side - 1))
                .map(move |x| Column((z * side + x) as u32))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The shipping enclosure: `world::ENCLOSURE` is 16, so a 33x33 grid. Sized
    // from the constant here the same way the world sizes it.
    const ENCLOSURE: i32 = 16;

    #[test]
    fn the_store_covers_the_enclosure_one_column_per_voxel() {
        let soil = Soil::seeded(ENCLOSURE, 100);
        assert_eq!(soil.side(), 33);
        assert_eq!(soil.columns(), 33 * 33);
        assert_eq!(soil.total_mg(), 33 * 33 * 100);

        // Distinct columns for distinct voxels, and the same one twice for
        // the same voxel: this is the grain the ruling bought.
        let mut seen = std::collections::BTreeSet::new();
        for z in -ENCLOSURE..=ENCLOSURE {
            for x in -ENCLOSURE..=ENCLOSURE {
                assert!(seen.insert(soil.column_at([x, 7, z])), "({x},{z}) collided");
            }
        }
        assert_eq!(seen.len(), soil.columns());
    }

    #[test]
    fn height_is_not_part_of_a_column() {
        let soil = Soil::seeded(ENCLOSURE, 1);
        assert_eq!(soil.column_at([3, -4, -7]), soil.column_at([3, 22, -7]));
    }

    #[test]
    fn a_position_past_the_wall_clamps_rather_than_dropping_matter() {
        // Insurance against a leak at the edge: whatever is deposited has to
        // land in a column that exists, or the cycle stops conserving.
        let mut soil = Soil::seeded(ENCLOSURE, 0);
        let outside = soil.column_at([900, 0, -900]);
        soil.deposit(outside, 40);
        assert_eq!(soil.total_mg(), 40);
        assert_eq!(outside, soil.column_at([ENCLOSURE, 0, -ENCLOSURE]));
    }

    #[test]
    fn a_draw_never_takes_more_than_the_column_holds() {
        let mut soil = Soil::seeded(2, 30);
        let column = soil.column_at([0, 0, 0]);
        assert_eq!(soil.draw(column, 12), 12);
        assert_eq!(soil.matter_mg(column), 18);
        assert_eq!(
            soil.draw(column, 1_000),
            18,
            "a spent column gives what it has"
        );
        assert_eq!(soil.draw(column, 1_000), 0);
    }

    #[test]
    fn drawing_and_depositing_move_matter_without_making_it() {
        let mut soil = Soil::seeded(4, 50);
        let before = soil.total_mg();
        let from = soil.column_at([-3, 0, 2]);
        let to = soil.column_at([1, 0, -4]);
        let moved = soil.draw(from, 40);
        soil.deposit(to, moved);
        assert_eq!(soil.total_mg(), before);
    }

    #[test]
    fn a_radius_reads_a_real_neighbourhood_at_the_shipping_size() {
        // The measured reason for this grain: r=3 is 49 of 1,089 columns
        // here, where the coarse grains' r=3 already covered the whole world.
        let soil = Soil::seeded(ENCLOSURE, 0);
        let middle = soil.column_at([0, 0, 0]);
        assert_eq!(soil.columns_within(middle, 3).count(), 49);
        assert_eq!(soil.columns_within(middle, 0).count(), 1);
        // Clipped at the wall rather than wrapping onto the far side.
        let corner = soil.column_at([-ENCLOSURE, 0, -ENCLOSURE]);
        assert_eq!(soil.columns_within(corner, 3).count(), 16);
    }

    #[test]
    fn a_root_reaches_the_richest_column_it_can_search_and_takes_only_its_income() {
        // The TD7 shape: wide reach, ordinary draw. A spent column no longer
        // means a spent producer, because the neighbourhood is what it eats
        // out of — but one tick still only buys one tick's income.
        let mut soil = Soil::seeded(ENCLOSURE, 0);
        let standing = soil.column_at([0, 0, 0]);
        let rich = soil.column_at([2, 0, -3]);
        soil.deposit(rich, 500);

        assert_eq!(soil.draw_richest_within(standing, FORAGE_RADIUS, 20), 20);
        assert_eq!(soil.matter_mg(rich), 480, "only the income came out");
        assert_eq!(soil.total_mg(), 480, "and nothing was made or lost");

        // Out of reach is out of reach: the radius is a real bound, not a
        // world-wide search wearing one.
        let far = soil.column_at([12, 0, 12]);
        soil.deposit(far, 900);
        assert_eq!(
            soil.draw_richest_within(standing, FORAGE_RADIUS, 1_000),
            480
        );
        assert_eq!(soil.matter_mg(far), 900);
    }

    #[test]
    fn a_forage_read_is_deterministic_when_every_column_is_equal() {
        // Ties go to the lowest column index, so a stand on flat ground draws
        // the same way in every replay of the same world.
        let mut a = Soil::seeded(ENCLOSURE, 100);
        let mut b = a.clone();
        let middle = a.column_at([0, 0, 0]);
        a.draw_richest_within(middle, FORAGE_RADIUS, 7);
        b.draw_richest_within(middle, FORAGE_RADIUS, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn a_store_round_trips() {
        let mut soil = Soil::seeded(ENCLOSURE, 7);
        soil.deposit(soil.column_at([5, 0, -5]), 99);
        let bytes = crate::snapshot::encode(&soil).unwrap();
        assert_eq!(crate::snapshot::decode::<Soil>(&bytes).unwrap(), soil);
    }
}
