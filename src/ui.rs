use crate::tileset::{Pos, TileSet};
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

pub struct Sdl2UI<'a> {
    w: u32,
    h: u32,
    _ctx: Sdl,
    _video_ss: VideoSubsystem,
    canvas: Canvas<Window>,
    buf: Surface<'a>,
    tc: TextureCreator<WindowContext>,
    evts: EventPump,
    bg: Color,
}

impl<'a> Sdl2UI<'a> {
    pub fn init(w: u32, h: u32, window_title: &str) -> anyhow::Result<Self> {
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

        Ok(Self {
            w,
            h,
            _ctx: ctx,
            _video_ss: video_ss,
            canvas,
            buf,
            tc,
            evts,
            bg: Color::BLACK,
        })
    }

    /// Toggle the background color between black and magenta to help with debugging rendering
    /// issues
    pub fn toggle_debug_bg(&mut self) {
        if self.bg == Color::BLACK {
            self.bg = Color::MAGENTA;
        } else {
            self.bg = Color::BLACK;
        }
    }

    pub fn next_event(&mut self) -> Option<Event> {
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

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(self.bg);
        self.canvas.clear();
        self.buf.fill_rect(None, self.bg).unwrap();
    }

    pub fn blit_tile(
        &mut self,
        pos: Pos,
        color: Color,
        ts: &mut TileSet,
        r: Rect,
    ) -> anyhow::Result<()> {
        ts.blit_tile(pos, color, &mut self.buf, r)
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
