//! https://www.roguebasin.com/index.php/The_Incredible_Power_of_Dijkstra_Maps
//! https://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized
use crate::grid::{Grid, Pos, astar::Candidate};
use indexmap::{IndexMap, map::Entry};
use std::collections::BinaryHeap;

pub fn dijkstra_map<T, F>(grid: &Grid<T>, targets: &[(Pos, i32)], cost_fn: F) -> Grid<i32>
where
    F: Fn(Pos) -> Option<i32>,
{
    let mut cost_map = IndexMap::new();
    let mut open_set = BinaryHeap::new();

    for (index, &(pos, cost)) in targets.iter().enumerate() {
        cost_map.insert(pos, (usize::MAX, cost)); // parent and cost
        open_set.push(Candidate {
            estimate: 0,
            cost,
            index,
        });
    }

    while let Some(Candidate {
        mut cost,
        index: parent,
        ..
    }) = open_set.pop()
    {
        let (pos, &(_, c)) = cost_map.get_index(parent).unwrap();
        if cost > c {
            continue; // already have a better path to this cell
        }

        for pos in grid.neighbouring_tiles(*pos) {
            match (cost_fn)(pos) {
                Some(c) => cost += c,
                None => continue, // blocked
            }

            let index = match cost_map.entry(pos) {
                Entry::Vacant(e) => {
                    let index = e.index();
                    e.insert((parent, cost));
                    index
                }

                Entry::Occupied(mut e) => {
                    if cost >= e.get().1 {
                        continue;
                    }
                    let index = e.index();
                    e.insert((parent, cost));
                    index
                }
            };

            open_set.push(Candidate {
                estimate: cost,
                cost,
                index,
            });
        }
    }

    let mut dmap = Grid::new(grid.w, grid.h, i32::MAX);
    for (pos, (_, cost)) in cost_map.into_iter() {
        dmap[pos] = cost;
    }

    dmap
}
