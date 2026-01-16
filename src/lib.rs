//! # Conway’s Game of Life (WASM)
//!
//! This crate implements a toroidal (wrap-around) version of Conway’s Game of Life
//! and exposes it to JavaScript via [`wasm_bindgen`].
//!
//! ## Data model
//! - [`Position`] represents a single cell coordinate (`x`, `y`).
//! - [`World`] stores the grid dimensions and the set of currently alive cells.
//!
//! **Coordinate convention:** positions are effectively treated as **1-based** in
//! the wrapping logic. Any position can be passed in, but neighbors are wrapped
//! into the inclusive ranges `1..=width` and `1..=height`.
//!
//! ## Rules
//! For each generation:
//! - A live cell survives if it has 2 or 3 live neighbors.
//! - An empty cell becomes alive (birth) if it has exactly 3 live neighbors.
//! - All other cells die or remain empty.
//!
//! Neighbor counting uses the 8 surrounding cells (Moore neighborhood), with
//! wrap-around at the edges (toroidal topology).
//!
//! ## Public API
//! - Construct a world with [`World::new`] or with the provided patterns
//!   ([`glider_pattern`], [`pulsar_pattern`]).
//! - Advance the simulation in-place using [`World::tick`].
//! - Retrieve current live cells with [`World::alive_positions`].
//!
//! ## Performance note
//! Alive cells are stored in a `Vec<Position>`, and membership checks use
//! `Vec::contains`, which is `O(n)`. For large worlds or dense populations,
//! consider switching to a `HashSet`-based representation if performance becomes
//! an issue.
//!
//! ## Example
//! ```no_run
//! use game_of_life_wasm::{glider_pattern};
//!
//! let mut world = glider_pattern(20, 20);
//! world.tick(); // advance one generation
//! let alive = world.alive_positions();
//! ```
//!
//! [`wasm_bindgen`]: https://docs.rs/wasm-bindgen
extern crate wasm_bindgen;

use std::collections::HashSet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// Simple coordinate for an individual cell.
pub struct Position {
    pub x: i64,
    pub y: i64,
}

#[wasm_bindgen]
impl Position {
    #[wasm_bindgen(constructor)]
    // JS-friendly constructor for a coordinate.
    pub fn new(x: i64, y: i64) -> Position {
        Position { x, y }
    }
}

#[wasm_bindgen]
// Toroidal world state and its live cell positions.
pub struct World {
    width: i64,
    height: i64,
    alive: Vec<Position>,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    // Create a world with given dimensions and initial live cells.
    pub fn new(width: u32, height: u32, alive: Vec<Position>) -> World {
        World {
            width: width.into(),
            height: height.into(),
            alive: alive,
        }
    }

    // Membership check for a live cell.
    fn is_alive(&self, position: Position) -> bool {
        self.alive.contains(&position)
    }

    // Convenience for "not alive".
    fn is_empty(&self, position: Position) -> bool {
        !self.is_alive(position)
    }

    // Wrap a coordinate into the 1-based toroidal bounds.
    fn wrap(&self, position: Position) -> Position {
        let x = (position.x - 1).rem_euclid(self.width) + 1;
        let y = (position.y - 1).rem_euclid(self.height) + 1;

        Position::new(x, y)
    }

    // Return the 8 Moore-neighborhood positions with wrapping.
    fn neighbors(&self, position: Position) -> Vec<Position> {
        let x = position.x;
        let y = position.y;

        let neighboring_cells = [
            Position::new(x - 1, y - 1), // Top-left
            Position::new(x, y - 1),     // Top
            Position::new(x + 1, y - 1), // Top-right
            Position::new(x - 1, y),     // Left
            Position::new(x + 1, y),     // Right
            Position::new(x - 1, y + 1), // Bottom-left
            Position::new(x, y + 1),     // Bottom
            Position::new(x + 1, y + 1), // Bottom-right
        ];

        // Pre-allocate to avoid growth when pushing wrapped neighbors.
        let mut results = Vec::with_capacity(neighboring_cells.len());

        for pos in neighboring_cells {
            results.push(self.wrap(pos));
        }

        results
    }

    // Count live neighbors around a position.
    fn live_neighbors(&self, position: Position) -> usize {
        self.neighbors(position)
            .into_iter()
            .filter(|pos| self.is_alive(*pos))
            .count()
    }

    // Compute survivors among the current live cells.
    fn survivors(&self) -> Vec<Position> {
        self.alive
            .iter()
            .filter_map(|pos| {
                let count = self.live_neighbors(*pos);
                if count == 2 || count == 3 {
                    Some(pos)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }

    // Compute new births among empty neighbor cells.
    fn births(&self) -> Vec<Position> {
        let mut potential_births: HashSet<Position> = HashSet::new();

        for pos in &self.alive {
            for neighbor in self.neighbors(*pos) {
                if self.is_empty(neighbor) {
                    potential_births.insert(neighbor);
                }
            }
        }

        potential_births
            .into_iter()
            .filter(|pos| self.live_neighbors(*pos) == 3)
            .collect()
    }

    // Build the next world state without mutating the current one.
    pub fn next_generation(&self) -> World {
        let mut new_alive = self.survivors();
        new_alive.extend(self.births());

        World {
            width: self.width,
            height: self.height,
            alive: new_alive,
        }
    }

    // Expose current live cells to JS.
    pub fn alive_positions(&self) -> Vec<Position> {
        self.alive.clone()
    }

    // Advance the world in place by one generation.
    pub fn tick(&mut self) {
        let next = self.next_generation();

        self.alive = next.alive;
    }
}

#[wasm_bindgen]
// Create a small world seeded with a glider.
pub fn glider_pattern(width: u32, height: u32) -> World {
    if width < 5 || height < 5 {
        panic!("World must be at least 5x5 to fit a glider pattern.");
    }

    // Hard-coded glider coordinates within the 5x5 minimum.
    let glider = vec![
        Position::new(4, 2),
        Position::new(2, 3),
        Position::new(4, 3),
        Position::new(3, 4),
        Position::new(4, 4),
    ];

    World::new(width, height, glider)
}

#[wasm_bindgen]
// Create a world seeded with a pulsar oscillator.
pub fn pulsar_pattern(width: u32, height: u32) -> World {
    if width < 15 || height < 15 {
        panic!("World must be at least 15x15 to fit a pulsar pattern.");
    }

    // Hard-coded pulsar coordinates within the 15x15 minimum.
    let pulsar = vec![
        Position::new(7, 4),
        Position::new(8, 4),
        Position::new(9, 4),
        Position::new(13, 4),
        Position::new(14, 4),
        Position::new(15, 4),
        Position::new(5, 6),
        Position::new(10, 6),
        Position::new(12, 6),
        Position::new(17, 6),
        Position::new(5, 7),
        Position::new(10, 7),
        Position::new(12, 7),
        Position::new(17, 7),
        Position::new(5, 8),
        Position::new(10, 8),
        Position::new(12, 8),
        Position::new(17, 8),
        Position::new(7, 10),
        Position::new(8, 10),
        Position::new(9, 10),
        Position::new(13, 10),
        Position::new(14, 10),
        Position::new(15, 10),
    ];

    World::new(width, height, pulsar)
}

#[cfg(test)]
// Unit tests for core world logic and patterns.
mod tests {
    use super::*;

    #[test]
    // Verify construction stores dimensions and initial live cells.
    fn test_world_creation() {
        let world = World::new(10, 10, vec![Position::new(2, 3)]);

        assert_eq!(world.width, 10);
        assert_eq!(world.height, 10);
        assert_eq!(world.alive.len(), 1);
    }

    #[test]
    // Check alive/empty predicates.
    fn test_alive_and_empty() {
        let world = World::new(5, 5, vec![Position::new(1, 1)]);

        assert!(world.is_alive(Position::new(1, 1)));
        assert!(world.is_empty(Position::new(2, 2)));
    }

    #[test]
    // Ensure wrapping for out-of-bounds positions.
    fn test_wrap() {
        let world = World::new(5, 5, vec![]);

        assert_eq!(world.wrap(Position::new(0, 3)), Position::new(5, 3));
        assert_eq!(world.wrap(Position::new(6, 3)), Position::new(1, 3));
        assert_eq!(world.wrap(Position::new(3, 0)), Position::new(3, 5));
        assert_eq!(world.wrap(Position::new(3, 6)), Position::new(3, 1));
    }

    #[test]
    // Count neighbors with wrapping involved.
    fn test_live_neighbors() {
        let world = World::new(
            5,
            5,
            vec![
                Position::new(1, 1),
                Position::new(1, 2),
                Position::new(2, 1),
            ],
        );

        assert_eq!(world.live_neighbors(Position::new(1, 1)), 2);
        assert_eq!(world.live_neighbors(Position::new(2, 2)), 3);
        assert_eq!(world.live_neighbors(Position::new(0, 0)), 1);
    }

    #[test]
    // Keep only cells with 2 or 3 neighbors.
    fn test_survivors() {
        let world = World::new(
            5,
            5,
            vec![
                Position::new(1, 1),
                Position::new(1, 2),
                Position::new(2, 1),
                Position::new(3, 3),
            ],
        );

        let survivors = world.survivors();
        assert!(survivors.contains(&Position::new(1, 1)));
        assert!(survivors.contains(&Position::new(1, 2)));
        assert!(survivors.contains(&Position::new(2, 1)));
        assert!(!survivors.contains(&Position::new(3, 3)));
    }

    #[test]
    // Births occur at empty cells with exactly three neighbors.
    fn test_births() {
        let world = World::new(
            5,
            5,
            vec![
                Position::new(1, 1),
                Position::new(1, 2),
                Position::new(2, 1),
            ],
        );

        let births = world.births();
        assert!(births.contains(&Position::new(2, 2)));
        assert!(!births.contains(&Position::new(3, 3)));
    }

    #[test]
    // Next generation should include survivors and valid births.
    fn test_next_generation() {
        let world = World::new(
            5,
            5,
            vec![
                Position::new(1, 1),
                Position::new(1, 2),
                Position::new(2, 1),
            ],
        );

        let next_world = world.next_generation();

        assert!(next_world.is_alive(Position::new(1, 1)));
        assert!(next_world.is_alive(Position::new(1, 2)));
        assert!(next_world.is_alive(Position::new(2, 1)));
        assert!(next_world.is_alive(Position::new(2, 2)));
    }
}
