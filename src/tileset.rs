use crate::data_files::parse_tile_map;
use anyhow::anyhow;
use sdl2::{
    image::LoadSurface,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    surface::Surface,
};
use std::{collections::HashMap, path::Path};

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
    tiles: HashMap<String, Pos>,
}

impl<'a> TileSet<'a> {
    pub fn urizen() -> anyhow::Result<Self> {
        parse_tile_map("assets/urizen/tile.map")
    }

    pub(crate) fn new(
        path: impl AsRef<Path>,
        dx: u16,
        dy: u16,
        start: Pos,
        gap: u16,
    ) -> anyhow::Result<Self> {
        let s = Surface::from_file(path)
            .map_err(|e| anyhow!("unable to load tileset: {e}"))?
            .convert_format(PixelFormatEnum::ARGB8888)
            .map_err(|e| anyhow!("unable to convert image format: {e}"))?;

        Ok(Self {
            s,
            dx,
            dy,
            start,
            gap,
            tiles: HashMap::new(),
        })
    }

    /// Map a row/column offset within a tilesheet into the correct pixel coordinates for blitting
    /// the tile.
    pub fn map_tile(&mut self, ident: impl Into<String>, row: u16, col: u16) -> Pos {
        let p = self.pos(row, col);
        self.tiles.insert(ident.into(), p);

        p
    }

    pub fn pos(&self, row: u16, col: u16) -> Pos {
        let mut p = self.start;
        p.x += (col * (self.dx + self.gap)) as i32;
        p.y += (row * (self.dy + self.gap)) as i32;

        p
    }

    pub fn tile(&self, ident: &str) -> Option<Pos> {
        self.tiles.get(ident).copied()
    }

    pub fn tile_name(&self, p: Pos) -> Option<&str> {
        self.tiles
            .iter()
            .find(|(_, v)| **v == p)
            .map(|(k, _)| k.as_str())
    }

    pub fn blit_tile(
        &mut self,
        pos: Pos,
        color: Color,
        dest: &mut Surface,
        r: Rect,
    ) -> anyhow::Result<()> {
        let r_tile = Rect::new(pos.x, pos.y, self.dx as u32, self.dy as u32);
        self.s.set_color_mod(color);
        self.s
            .blit_scaled(r_tile, dest, r)
            .map_err(|e| anyhow!("unable to blit tile: {e}"))?;

        Ok(())
    }
}
