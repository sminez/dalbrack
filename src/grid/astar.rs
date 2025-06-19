// https://en.wikipedia.org/wiki/A*_search_algorithm
// https://www.redblobgames.com/pathfinding/
use crate::grid::{Grid, Pos};
use indexmap::{IndexMap, map::Entry};
use std::{
    cmp::{Ordering, max, min},
    collections::BinaryHeap,
    iter::from_fn,
};

/// Compute a path from `a` to `b` in terms of a sequence of [Pos] steps.
///
/// If no path can be found then the resulting vec will be empty.
pub fn a_star<T, F>(a: Pos, b: Pos, grid: &Grid<T>, cost_fn: F) -> Vec<Pos>
where
    F: Fn(Pos) -> Option<i32>,
{
    let mut came_from = IndexMap::new();
    came_from.insert(a, (usize::MAX, 0)); // parent and cost

    let mut open_set = BinaryHeap::new();
    open_set.push(Candidate {
        estimate: 0,
        cost: 0,
        index: 0,
    });

    while let Some(Candidate {
        mut cost,
        index: parent,
        ..
    }) = open_set.pop()
    {
        let (pos, &(_, c)) = came_from.get_index(parent).unwrap();
        if *pos == b {
            return build_path(parent, &came_from);
        } else if cost > c {
            continue; // already have a better path to this cell
        }

        for pos in grid.neighbouring_tiles(*pos) {
            match (cost_fn)(pos) {
                Some(c) => cost += c * 10, // to match the 10/14 weights in heuristic
                None => continue,          // can't path through
            }
            let h = heuristic(pos, b);

            let index = match came_from.entry(pos) {
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
                estimate: cost + h,
                cost,
                index,
            });
        }
    }

    Vec::new()
}

// Approximating octile distance using 10 and 14 for the vertical/horizontal and diagonal costs.
//
// Here we compute the number of steps you take if you can’t take a diagonal, then subtract the
// steps you save by using the diagonal. There are min(dx, dy) diagonal steps, and each one costs
// D2 but saves you 2⨉D non-diagonal steps.
//
//   http://theory.stanford.edu/~amitp/GameProgramming/Heuristics.html#diagonal-distance
fn heuristic(p: Pos, to: Pos) -> i32 {
    let (dx, dy) = ((p.x - to.x).abs(), (p.y - to.y).abs());

    // 10 = d1, 4 = (d2-d1)
    10 * max(dx, dy) + 4 * min(dx, dy)
}

fn build_path(start: usize, came_from: &IndexMap<Pos, (usize, i32)>) -> Vec<Pos> {
    let mut parent = start;
    let mut path: Vec<Pos> = from_fn(|| {
        came_from.get_index(parent).map(|(pos, (i, _))| {
            parent = *i;
            *pos
        })
    })
    .collect();
    path.reverse();

    path
}

#[derive(Debug, PartialEq, Eq)]
struct Candidate {
    estimate: i32,
    cost: i32,
    index: usize,
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // prefer smallest estimated cost where possible, breaking ties with the highest true cost
        // as that can indicate that we are already closer to the target
        match other.estimate.cmp(&self.estimate) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            o => o,
        }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
