//! Simple Voronoi diagram generation
//!   https://en.wikipedia.org/wiki/Voronoi_diagram
use crate::{Pos, rng::RngHandle};
use sdl2::rect::Rect;

pub fn voronoi_seeds(n_seeds: usize, w: usize, h: usize, rng: &mut RngHandle) -> Vec<Pos> {
    let mut seeds = Vec::with_capacity(n_seeds);
    let r = Rect::new(0, 0, w as u32, h as u32);
    while seeds.len() < n_seeds {
        let pos = rng.random_point(r, 1);
        if !seeds.contains(&pos) {
            seeds.push(pos);
        }
    }

    seeds
}

pub fn voronoi_regions_from_seeds(
    seeds: &[Pos],
    points: impl IntoIterator<Item = Pos>,
) -> Vec<Vec<Pos>> {
    let mut regions = vec![vec![]; seeds.len()];
    let mut dists = vec![(0, 0.0); seeds.len()];

    for p in points.into_iter() {
        for (i, seed) in seeds.iter().enumerate() {
            dists[i] = (i, seed.fdist(p));
        }
        dists.sort_by(|a, b| a.1.total_cmp(&b.1));
        regions[dists[0].0].push(p);
    }

    regions
}

pub fn voronoi_regions(
    n_seeds: usize,
    w: usize,
    h: usize,
    points: impl IntoIterator<Item = Pos>,
    rng: &mut RngHandle,
) -> Vec<Vec<Pos>> {
    let seeds = voronoi_seeds(n_seeds, w, h, rng);
    voronoi_regions_from_seeds(&seeds, points)
}
