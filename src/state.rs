//! The global state of the game
use crate::{
    Pos,
    data_files::parse_color_palette,
    map::{
        Map,
        fov::{Fov, LightSource},
    },
    tileset::{Tile, TileSet},
    ui::Sdl2UI,
};
use hecs::{Entity, World};
use sdl2::{pixels::Color, rect::Rect};
use std::collections::HashMap;

pub struct State<'a> {
    pub world: World,
    pub e_player: Entity,
    pub e_map: Entity,
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
        let mut world = World::new();
        let e_player = world.spawn(());
        let e_map = world.spawn(());

        Ok(State {
            world,
            e_player,
            e_map,
            ui,
            ts,
            palette,
        })
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        self.update_fov()?;

        self.ui.clear();
        self.blit_all()?;
        self.ui.render()?;

        Ok(())
    }

    pub fn tile_with_color(&self, ident: &str, color: &str) -> Tile {
        let mut tile = self.ts.tile(ident).unwrap();
        tile.color = *self.palette.get(color).unwrap();

        tile
    }

    pub fn set_map(&mut self, map: Map) {
        self.world
            .insert_one(self.e_map, map)
            .expect("e_map to be valid");
    }

    /// This will no-op rather than error if we are missing the correct player components
    /// or if the map is missing.
    pub fn update_fov(&mut self) -> anyhow::Result<()> {
        let (pos, source) = match self
            .world
            .query_one_mut::<(&Pos, &LightSource)>(self.e_player)
        {
            Ok((pos, source)) => (*pos, *source),
            Err(_) => return Ok(()),
        };

        let map = match self.world.query_one_mut::<&mut Map>(self.e_map) {
            Ok(map) => map,
            Err(_) => return Ok(()),
        };

        let fov = map.fov(pos, source);
        self.world.insert_one(self.e_map, fov)?;

        Ok(())
    }

    pub fn blit_all(&mut self) -> anyhow::Result<()> {
        self.blit_map()?;
        self.blit_tiles()?;
        self.blit_text()?;

        Ok(())
    }

    fn blit_map(&mut self) -> anyhow::Result<()> {
        let (map, fov) = if self.world.satisfies::<(&Map, &Fov)>(self.e_map)? {
            self.world
                .query_one_mut::<(&mut Map, &Fov)>(self.e_map)
                .map(|(map, fov)| (map, Some(fov)))?
        } else {
            match self.world.query_one_mut::<&mut Map>(self.e_map) {
                Ok(map) => (map, None),
                Err(_) => return Ok(()), // no map to render
            }
        };

        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;
        r.x = 0;
        r.y = 0;

        // FIXME: this needs to be stored in the light map once its written
        // let black = *self.palette.get("grey16").unwrap();
        let black = *self.palette.get("hidden").unwrap();

        for (y, line) in map.tiles.chunks(map.w).enumerate() {
            for (x, tile_idx) in line.iter().enumerate() {
                r.x = x as i32 * dxy;
                r.y = y as i32 * dxy;
                let mut tile = map.tile_defs[*tile_idx];

                if let Some(fov) = fov.as_ref() {
                    let p = Pos::new(x as i32, y as i32);
                    if fov.points.contains(&p) {
                        map.explored.insert(map.pos_idx(p));
                    }
                    fov.apply_light_level(p, &mut tile.t.color, black);
                }

                if map.explored.contains(&map.idx(x, y)) {
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
