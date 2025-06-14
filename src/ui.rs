use crate::{
    data_files::parse_color_palette,
    map::{Cell, Grid, Tile},
    tileset::TileSet,
};
use anyhow::anyhow;
use sdl2::{
    EventPump, Sdl, VideoSubsystem,
    event::{Event, WindowEvent},
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext},
};
use std::collections::HashMap;

pub struct Sdl2UI<'a> {
    w: u32,
    h: u32,
    pub dxy: u32,
    _ctx: Sdl,
    _video_ss: VideoSubsystem,
    canvas: Canvas<Window>,
    buf: Surface<'a>,
    tc: TextureCreator<WindowContext>,
    evts: EventPump,
    bg: Color,
    debug: bool,
    pub ts: TileSet<'a>,
    pub palette: HashMap<String, Color>,
}

impl<'a> Sdl2UI<'a> {
    pub fn init(w: u32, h: u32, dxy: u32, window_title: &str) -> anyhow::Result<Self> {
        let ctx = sdl2::init().map_err(|e| anyhow!("{e}"))?;
        let video_ss = ctx.video().map_err(|e| anyhow!("{e}"))?;

        let win = video_ss
            .window(window_title, w, h)
            .position_centered()
            .build()?;

        let mut canvas = win.into_canvas().target_texture().present_vsync().build()?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let tc = canvas.texture_creator();
        let evts = ctx.event_pump().map_err(|e| anyhow!("{e}"))?;
        let buf = Surface::new(w, h, PixelFormatEnum::ARGB8888).map_err(|e| anyhow!("{e}"))?;
        let ts = TileSet::default();
        let palette = parse_color_palette()?;
        let bg = *palette.get("black").unwrap();

        Ok(Self {
            w,
            h,
            dxy,
            _ctx: ctx,
            _video_ss: video_ss,
            canvas,
            buf,
            tc,
            evts,
            bg,
            debug: false,
            ts,
            palette,
        })
    }

    /// Toggle the background color between black and magenta to help with debugging rendering
    /// issues
    pub fn toggle_debug_bg(&mut self) {
        self.debug = !self.debug;
    }

    fn bg(&self) -> Color {
        if self.debug { Color::MAGENTA } else { self.bg }
    }

    pub fn set_bg(&mut self, color_name: &str) {
        self.bg = *self.palette.get(color_name).unwrap();
    }

    /// Poll for currently pending events.
    ///
    /// Window resize events are handled internally.
    /// Returns None if no events are pending.
    pub fn poll_event(&mut self) -> Option<Event> {
        loop {
            match self.evts.poll_event()? {
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h) | WindowEvent::Resized(w, h),
                    ..
                } => {
                    self.w = w as u32;
                    self.h = h as u32;
                    self.buf = Surface::new(w as u32, h as u32, PixelFormatEnum::ARGB8888).unwrap();
                }

                evt => return Some(evt),
            }
        }
    }

    /// Block and wait for the next event.
    ///
    /// Window resize events are handled internally.
    pub fn wait_event(&mut self) -> Event {
        loop {
            match self.evts.wait_event() {
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h) | WindowEvent::Resized(w, h),
                    ..
                } => {
                    self.w = w as u32;
                    self.h = h as u32;
                    self.buf = Surface::new(w as u32, h as u32, PixelFormatEnum::ARGB8888).unwrap();
                }

                evt => return evt,
            }
        }
    }

    pub fn clear(&mut self) {
        let bg = self.bg();
        self.canvas.set_draw_color(bg);
        self.canvas.clear();
        self.buf.fill_rect(None, bg).unwrap();
    }

    fn rect_for_coords(&self, x: u32, y: u32) -> Rect {
        Rect::new(
            (x * self.dxy) as i32,
            (y * self.dxy) as i32,
            self.dxy,
            self.dxy,
        )
    }

    pub fn blit_tile(&mut self, tile: &Tile, x: u32, y: u32) -> anyhow::Result<()> {
        let r = self.rect_for_coords(x, y);
        self.ts.blit_tile(tile, r, &mut self.buf)
    }

    pub fn blit_cell(&mut self, cell: &Cell, x: u32, y: u32) -> anyhow::Result<()> {
        let r = self.rect_for_coords(x, y);
        self.ts.blit_cell(cell, r, &mut self.buf)
    }

    pub fn blit_grid(&mut self, grid: &Grid, x: u32, y: u32) -> anyhow::Result<()> {
        self.ts.blit_grid(
            grid,
            (x * self.dxy) as i32,
            (y * self.dxy) as i32,
            self.dxy,
            &mut self.buf,
        )
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let tx = self.buf.as_texture(&self.tc)?;
        self.canvas
            .copy(&tx, None, None)
            .map_err(|e| anyhow!("unable to copy buffer to canvas: {e}"))?;
        self.canvas.present();

        Ok(())
    }
}
