use std::sync::mpsc::Receiver;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use crate::color::Color;

pub(crate) fn sdl_thread(
    image_width: usize,
    image_height: usize,
    receiver: Receiver<((usize, usize), (usize, usize), Vec<Color>)>,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("raytracer", image_width as u32, image_height as u32)
        //.position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            canvas.output_size().unwrap().0,
            canvas.output_size().unwrap().1,
        )
        .map_err(|e| e.to_string())
        .unwrap();

    canvas.copy(&texture, None, None).unwrap();

    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        while let Ok((top_left, size, result)) = receiver.try_recv() {
            texture
                .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                    for dy in 0..size.0 {
                        for dx in 0..size.1 {
                            let index = (((top_left.0 + dy) * image_width) + (top_left.1 + dx)) * 3;
                            let (ir, ig, ib) = result[(dy * size.1) + dx].into_u8();
                            buffer[index + 0] = ir;
                            buffer[index + 1] = ig;
                            buffer[index + 2] = ib;
                        }
                    }
                })
                .unwrap();
            let rect = Rect::new(
                top_left.1 as i32,
                top_left.0 as i32,
                size.1 as u32,
                size.0 as u32,
            );
            canvas.copy(&texture, Some(rect), Some(rect)).unwrap();
        }
        //canvas.copy(&texture, None, None).unwrap();

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    drop(receiver);
}
