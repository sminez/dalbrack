//! The global state of the game
use crate::{
    Pos,
    data_files::parse_color_palette,
    map::Map,
    tileset::{Tile, TileSet},
    ui::Sdl2UI,
};
use hecs::World;
use sdl2::{pixels::Color, rect::Rect};
use std::collections::HashMap;

pub struct State<'a> {
    pub world: World,
    pub ui: Sdl2UI<'a>,
    pub ts: TileSet<'a>,
    pub palette: HashMap<String, Color>,
}

impl<'a> State<'a> {
    pub fn init(w: u32, h: u32, dxy: u32, window_title: &str) -> anyhow::Result<Self> {
        let ts = TileSet::default();
        let palette = parse_color_palette()?;
        let bg = *palette.get("black").unwrap();
        let ui = Sdl2UI::init(w, h, dxy, window_title, bg)?;

        Ok(State {
            world: World::new(),
            ui,
            ts,
            palette,
        })
    }

    pub fn tile_with_color(&self, ident: &str, color: &str) -> Tile {
        let mut tile = self.ts.tile(ident).unwrap();
        tile.color = *self.palette.get(color).unwrap();

        tile
    }

    pub fn blit_all(&mut self) -> anyhow::Result<()> {
        self.blit_map()?;
        self.blit_tiles()?;
        self.blit_text()?;

        Ok(())
    }

    fn blit_map(&mut self) -> anyhow::Result<()> {
        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;

        for (_entity, map) in self.world.query_mut::<&Map>() {
            r.x = 0;
            r.y = 0;

            for (y, line) in map.tiles.chunks(map.w).enumerate() {
                for (x, tile_idx) in line.iter().enumerate() {
                    if !map.explored.contains(&(y * map.w + x)) {
                        continue;
                    }

                    r.x = x as i32 * dxy;
                    r.y = y as i32 * dxy;
                    let tile = &map.tile_defs[*tile_idx];
                    self.ts.blit_tile(&tile.t, r, &mut self.ui.buf)?;
                }
            }
        }

        Ok(())
    }

    fn blit_tiles(&mut self) -> anyhow::Result<()> {
        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;

        for (_entity, (pos, tile)) in self.world.query_mut::<(&Pos, &Tile)>() {
            r.x = pos.x * dxy;
            r.y = pos.y * dxy;
            self.ts.blit_tile(tile, r, &mut self.ui.buf)?;
        }

        Ok(())
    }

    fn blit_text(&mut self) -> anyhow::Result<()> {
        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;

        let mut buf = [0; 4];
        for (_entity, (pos, s)) in self.world.query_mut::<(&Pos, &String)>() {
            r.x = pos.x * dxy;
            r.y = pos.y * dxy;

            for ch in s.chars() {
                let ident = ch.encode_utf8(&mut buf);
                let tile = self.ts.tile(ident).unwrap();
                self.ts.blit_tile(&tile, r, &mut self.ui.buf)?;
                r.x += dxy;
            }
        }

        Ok(())
    }
}
