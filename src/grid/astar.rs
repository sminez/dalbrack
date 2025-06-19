use crate::grid::{Grid, Pos};
use indexmap::{IndexMap, map::Entry};
use std::{
    cmp::{Ordering, max, min},
    collections::BinaryHeap,
    iter::from_fn,
};

pub trait WeightedGrid {
    type Cell;

    fn grid(&self) -> &Grid<Self::Cell>;
    fn cost(&self, pos: Pos) -> Option<i32>;

    /// Compute a path from `a` to `b` in terms of a sequence of [Pos] steps.
    ///
    /// If no path can be found then the resulting vec will be empty.
    fn a_star(&self, a: Pos, b: Pos) -> Vec<Pos> {
        let mut came_from = IndexMap::new();
        came_from.insert(a, (usize::MAX, 0));

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

            for pos in self.grid().neighbouring_tiles(*pos) {
                match self.cost(pos) {
                    Some(c) => cost += c,
                    None => continue, // can't path through
                }
                let h = heuristic(pos, b);

                let index = match came_from.entry(pos) {
                    Entry::Vacant(e) => {
                        let index = e.index();
                        e.insert((parent, cost));
                        index
                    }

                    Entry::Occupied(mut e) => {
                        if e.get().1 <= cost {
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
}

// Approximating octile distance using 10 and 14 for the vertical/horizontal and diagonal costs and
// then dividing through by 10 to scale the result to match the tile costs.
//
// Here we compute the number of steps you take if you can’t take a diagonal, then subtract the
// steps you save by using the diagonal. There are min(dx, dy) diagonal steps, and each one costs
// D2 but saves you 2⨉D non-diagonal steps.
//
//   http://theory.stanford.edu/~amitp/GameProgramming/Heuristics.html#diagonal-distance
fn heuristic(p: Pos, to: Pos) -> i32 {
    let (dx, dy) = ((p.x - to.x).abs(), (p.y - to.y).abs());

    // 10 = d1, 4 = (d2-d1)
    (10 * max(dx, dy) + 4 * min(dx, dy)) / 10
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

struct Candidate {
    estimate: i32,
    cost: i32,
    index: usize,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.estimate.eq(&other.estimate) && self.cost.eq(&other.cost)
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.estimate.cmp(&other.estimate) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            o => o,
        }
    }
}
