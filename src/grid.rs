use std::ops::{Index, IndexMut};

/// A cell position within a [Grid]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn dist(&self, other: Pos) -> u32 {
        self.fdist(other).ceil() as u32
    }

    pub fn fdist(&self, other: Pos) -> f32 {
        (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f32).sqrt()
    }
}

/// A generic 2D grid container
#[derive(Default, Debug, Clone)]
pub struct Grid<T> {
    pub cells: Vec<T>,
    pub w: usize,
    pub h: usize,
}

impl<T> Grid<T> {
    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    #[inline]
    pub fn pos_idx(&self, Pos { x, y }: Pos) -> usize {
        y as usize * self.w + x as usize
    }

    pub fn cell_at(&self, pos: Pos) -> &T {
        let idx = self.idx(pos.x as usize, pos.y as usize);
        &self.cells[idx]
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<T> Grid<T>
where
    T: Clone,
{
    pub fn new(w: usize, h: usize, fill_val: T) -> Self {
        Self {
            cells: vec![fill_val; w * h],
            w,
            h,
        }
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cells[index]
    }
}

impl<T> Index<Pos> for Grid<T> {
    type Output = T;

    fn index(&self, pos: Pos) -> &Self::Output {
        &self.cells[self.pos_idx(pos)]
    }
}

impl<T> IndexMut<Pos> for Grid<T> {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let idx = self.pos_idx(pos);
        &mut self.cells[idx]
    }
}
