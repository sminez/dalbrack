use crate::data_files::{parse_ibm437_tileset, parse_tile_map};
use anyhow::anyhow;
use sdl2::{
    image::LoadSurface,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::BlendMode,
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
        parse_tile_map("assets/tiles/urizen/tile.map")
    }

    pub fn df_classic() -> anyhow::Result<Self> {
        parse_ibm437_tileset(
            "assets/tiles/df/Curses_classic_square_12x12.png",
            12,
            Some(Color::MAGENTA),
        )
    }

    pub fn df_buddy() -> anyhow::Result<Self> {
        parse_ibm437_tileset("assets/tiles/df/Buddy.png", 10, None)
    }

    pub fn df_sb() -> anyhow::Result<Self> {
        parse_ibm437_tileset("assets/tiles/df/16x16_sb_ascii.png", 16, None)
    }

    pub fn df_nordic() -> anyhow::Result<Self> {
        parse_ibm437_tileset("assets/tiles/df/DF-Nordic_v1.png", 16, Some(Color::MAGENTA))
    }

    pub fn df_rde() -> anyhow::Result<Self> {
        parse_ibm437_tileset("assets/tiles/df/RDE_8x8.png", 8, Some(Color::MAGENTA))
    }

    pub fn df_yayo() -> anyhow::Result<Self> {
        parse_ibm437_tileset(
            "assets/tiles/df/Yayo_tunur_1040x325.png",
            13,
            Some(Color::MAGENTA),
        )
    }

    pub fn df_acorn() -> anyhow::Result<Self> {
        parse_ibm437_tileset(
            "assets/tiles/df/Acorntileset8x8.png",
            8,
            Some(Color::MAGENTA),
        )
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
