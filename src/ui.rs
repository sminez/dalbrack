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
        })
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
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    pub fn blit_tile(
        &mut self,
        ch: char,
        color: Color,
        ts: &mut TileSet,
        r: Rect,
    ) -> anyhow::Result<()> {
        ts.blit_tile(ch, color, &mut self.buf, r)
    }

    pub fn blit_pos(
        &mut self,
        pos: Pos,
        color: Color,
        ts: &mut TileSet,
        r: Rect,
    ) -> anyhow::Result<()> {
        ts.blit_pos(pos, color, &mut self.buf, r)
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
