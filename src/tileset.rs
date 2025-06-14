use crate::{
    data_files::{parse_ibm437_tileset, parse_tile_map},
    grid::{Cell, Grid, Tile},
};
use anyhow::anyhow;
use sdl2::{
    image::LoadSurface,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::BlendMode,
    surface::Surface,
};
use std::{collections::HashMap, ops::Index, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub struct TileSet<'a> {
    s: Surface<'a>,
    dx: u16,
    dy: u16,
    start: Pos,
    gap: u16,
    tiles: Vec<Pos>,
    idents: HashMap<String, usize>,
}

impl<'a> Default for TileSet<'a> {
    fn default() -> Self {
        Self::df_classic().unwrap()
    }
}

impl<'a> Index<usize> for TileSet<'a> {
    type Output = Pos;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tiles[index]
    }
}

impl<'a> TileSet<'a> {
    pub fn urizen() -> anyhow::Result<Self> {
        parse_tile_map("data/tilesets/urizen/tile.map")
    }

    pub fn df_classic() -> anyhow::Result<Self> {
        parse_ibm437_tileset(
            "data/tilesets/df/Curses_classic_square_12x12.png",
            12,
            Some(Color::MAGENTA),
        )
    }

    pub fn df_buddy() -> anyhow::Result<Self> {
        parse_ibm437_tileset("data/tilesets/df/Buddy.png", 10, None)
    }

    pub fn df_sb() -> anyhow::Result<Self> {
        parse_ibm437_tileset("data/tilesets/df/16x16_sb_ascii.png", 16, None)
    }

    pub fn df_nordic() -> anyhow::Result<Self> {
        parse_ibm437_tileset(
            "data/tilesets/df/DF-Nordic_v1.png",
            16,
            Some(Color::MAGENTA),
        )
    }

    pub fn df_rde() -> anyhow::Result<Self> {
        parse_ibm437_tileset("data/tilesets/df/RDE_8x8.png", 8, Some(Color::MAGENTA))
    }

    pub fn df_yayo() -> anyhow::Result<Self> {
        parse_ibm437_tileset(
            "data/tilesets/df/Yayo_tunur_1040x325.png",
            13,
            Some(Color::MAGENTA),
        )
    }

    pub fn df_kruggsmash() -> anyhow::Result<Self> {
        parse_ibm437_tileset("data/tilesets/df/kruggsmash.png", 32, Some(Color::MAGENTA))
    }

    pub(crate) fn new(
        path: impl AsRef<Path>,
        dx: u16,
        dy: u16,
        start: Pos,
        gap: u16,
        bg: Option<Color>,
    ) -> anyhow::Result<Self> {
        let mut s = Surface::from_file(path)
            .map_err(|e| anyhow!("unable to load tileset: {e}"))?
            .convert_format(PixelFormatEnum::ARGB8888)
            .map_err(|e| anyhow!("unable to convert image format: {e}"))?;

        if let Some(color) = bg {
            s.set_blend_mode(BlendMode::Blend)
                .map_err(|e| anyhow!("unable to set blend mode: {e}"))?;
            s.set_color_key(true, color)
                .map_err(|e| anyhow!("unable to set color key: {e}"))?;
        }

        Ok(Self {
            s,
            dx,
            dy,
            start,
            gap,
            tiles: Vec::new(),
            idents: HashMap::new(),
        })
    }

    /// Map a row/column offset within a tilesheet into the correct pixel coordinates for blitting
    /// the tile.
    pub fn map_tile(&mut self, ident: impl Into<String>, row: u16, col: u16) -> Tile {
        let p = self.pos(row, col);
        let idx = self.tiles.len();
        self.tiles.push(p);
        self.idents.insert(ident.into(), idx);

        Tile::new(idx)
    }

    pub fn pos(&self, row: u16, col: u16) -> Pos {
        let mut p = self.start;
        p.x += (col * (self.dx + self.gap)) as i32;
        p.y += (row * (self.dy + self.gap)) as i32;

        p
    }

    pub fn ibm437_tile(&self, row: u16, col: u16) -> Tile {
        Tile::new((row * 16 + col) as usize)
    }

    pub fn tile(&self, ident: &str) -> Option<Tile> {
        self.idents.get(ident).map(|&idx| Tile::new(idx))
    }

    pub fn tile_index(&self, ident: &str) -> Option<usize> {
        self.idents.get(ident).copied()
    }

    pub fn tile_name(&self, t: impl AsTileIndex) -> Option<&str> {
        let idx = t.as_index(self)?;
        self.idents
            .iter()
            .find(|(_, v)| **v == idx)
            .map(|(k, _)| k.as_str())
    }

    pub fn blit_tile(&mut self, tile: &Tile, r: Rect, dest: &mut Surface) -> anyhow::Result<()> {
        let pos = self[tile.idx];
        let r_tile = Rect::new(pos.x, pos.y, self.dx as u32, self.dy as u32);
        self.s.set_color_mod(tile.color);
        self.s
            .blit_scaled(r_tile, dest, r)
            .map_err(|e| anyhow!("unable to blit tile: {e}"))?;

        Ok(())
    }

    pub fn blit_cell(&mut self, cell: &Cell, r: Rect, dest: &mut Surface) -> anyhow::Result<()> {
        for tile in cell.iter() {
            self.blit_tile(tile, r, dest)?;
        }

        Ok(())
    }

    pub fn blit_grid(
        &mut self,
        grid: &Grid,
        x: i32,
        y: i32,
        dxy: u32,
        dest: &mut Surface,
    ) -> anyhow::Result<()> {
        let mut r = Rect::new(x, y, dxy, dxy);

        for line in grid.cells.chunks(grid.w) {
            for cell in line {
                self.blit_cell(cell, r, dest)?;
                r.x += dxy as i32;
            }
            r.x = x;
            r.y += dxy as i32;
        }

        Ok(())
    }
}

pub trait AsTileIndex {
    fn as_index(&self, ts: &TileSet<'_>) -> Option<usize>;
}

impl AsTileIndex for usize {
    fn as_index(&self, _ts: &TileSet<'_>) -> Option<usize> {
        Some(*self)
    }
}

impl AsTileIndex for Pos {
    fn as_index(&self, ts: &TileSet<'_>) -> Option<usize> {
        ts.tiles.iter().position(|pos| pos == self)
    }
}

impl AsTileIndex for Tile {
    fn as_index(&self, _ts: &TileSet<'_>) -> Option<usize> {
        Some(self.idx)
    }
}
