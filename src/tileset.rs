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
    pub s: Surface<'a>,
    dx: u32,
    dy: u32,
    start: Pos,
    gap: u32,
    char_map: HashMap<char, Pos>,
}

impl<'a> TileSet<'a> {
    pub fn urizen() -> anyhow::Result<Self> {
        Self::new(
            "assets/urizen/urizen_onebit_tileset__v1d1.png",
            12,
            12,
            Pos::new(1, 1),
            1,
        )
    }

    pub fn new(
        path: impl AsRef<Path>,
        dx: u32,
        dy: u32,
        start: Pos,
        gap: u32,
    ) -> anyhow::Result<Self> {
        let s = Surface::from_file(path.as_ref())
            .map_err(|e| anyhow!("unable to load tileset: {e}"))?
            .convert_format(PixelFormatEnum::ARGB8888)
            .map_err(|e| anyhow!("unable to convert image format: {e}"))?;

        Ok(Self {
            s,
            dx,
            dy,
            start,
            gap,
            char_map: Default::default(),
        })
    }

    pub fn map_tile(&mut self, ch: char, row: u32, col: u32) {
        self.char_map.insert(ch, self.pos(row, col));
    }

    pub fn pos(&self, row: u32, col: u32) -> Pos {
        let mut p = self.start;
        p.x += (col * (self.dx + self.gap)) as i32;
        p.y += (row * (self.dy + self.gap)) as i32;

        p
    }

    pub fn blit_tile(
        &mut self,
        ch: char,
        color: Color,
        dest: &mut Surface,
        r: Rect,
    ) -> anyhow::Result<()> {
        match self.char_map.get(&ch) {
            Some(pos) => self.blit_pos(*pos, color, dest, r),
            None => Ok(()),
        }
    }

    pub fn blit_pos(
        &mut self,
        pos: Pos,
        color: Color,
        dest: &mut Surface,
        r: Rect,
    ) -> anyhow::Result<()> {
        let r_tile = Rect::new(pos.x, pos.y, self.dx, self.dy);
        self.s.set_color_mod(color);
        self.s
            .blit_scaled(r_tile, dest, r)
            .map_err(|e| anyhow!("unable to blit tile: {e}"))?;

        Ok(())
    }
}
