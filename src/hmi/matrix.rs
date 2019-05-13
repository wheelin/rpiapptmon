use std::io::Write;
use std::fs::OpenOptions;
use std::io;
use std::ops::Drop;
use std::time::Duration;
use std::thread;

use crate::hmi::color::*;
use crate::hmi::glcdfont::FONT;

#[allow(dead_code)]
pub enum Orientation {
    Cw0,
    Cw90,
    Cw180,
    Cw270,
}

pub struct Matrix {
    framebuffer : [u8;192 * 2],
    file_name   : String,
    orientation : Orientation,
}

impl Matrix {
    pub fn new(f : String, or : Orientation) -> Matrix {
        Matrix {
            framebuffer : [0; 192 * 2],
            file_name   : f,
            orientation : or,
        }
    }


    pub fn set_frame(&mut self, frame : &[Color]) -> io::Result<()> {
        for x in 0..8 {
            for y in 0..8 {
                self.set_pixel(x, y, frame[y * 8 + x])?;
            }
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let mut file = OpenOptions::new().write(true).open(&self.file_name)?;
        match file.write_all(&self.framebuffer) {
            Ok(()) => (),
            Err(x) => {if let Some(e) = x.raw_os_error() {
                if e != 27 {
                    return Err(x)
                }
            }}
        };
        Ok(())
    }

    pub fn set_pixel(&mut self, x : usize, y : usize, c : Color) -> io::Result<()> {
        if x > 7 || y > 7 {
            return Err(
                io::Error::new(
                    io::ErrorKind::InvalidInput, 
                    "Pixel out of display boundaries."
                )
            );
        }

        let color_lsb = ((c.1 << 5) & 0xE0) | ( c.2 & 0x1F);
        let color_msb = ((c.0 << 3) & 0xF8) | ((c.1 >> 3) & 0x07);

        let fb_idx = match self.orientation {
            Orientation::Cw0   => 2 * (     y  * 8 +      x),
            Orientation::Cw90  => 2 * ((7 - y) * 8 +      x),
            Orientation::Cw180 => 2 * ((7 - y) * 8 + (7 - x)),
            Orientation::Cw270 => 2 * (     y  * 8 + (7 - x)),
        };

        self.framebuffer[fb_idx + 0] = color_lsb;
        self.framebuffer[fb_idx + 1] = color_msb;

        Ok(())
    }

    pub fn draw_line(&mut self, x0 : i8, x1 : i8, y0 : i8, y1 : i8, c : Color) -> io::Result<()> {
        if (x0 > 7 || y0 > 7 || x1 > 7 || y1 > 7) ||
           (x0 < 0 || y0 < 0 || x1 < 0 || y1 < 0) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Pixel out of display boundaries"));
        }

        // Create local variables for moving start point
        let mut x0 = x0;
        let mut y0 = y0;

        // Get absolute x/y offset
        let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
        let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };

        // Get slopes
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        // Initialize error
        let mut err = if dx > dy { dx } else { -dy } / 2;
        let mut err2;

        loop {
            // Set pixel
            self.set_pixel(x0 as usize, y0 as usize, c)?;

            // Check end condition
            if x0 == x1 && y0 == y1 {
                break;
            };

            // Store old error
            err2 = 2 * err;

            // Adjust error and start position
            if err2 > -dx {
                err -= dy;
                x0 += sx;
            }
            if err2 < dy {
                err += dx;
                y0 += sy;
            }
        }
        Ok(())
    }

    pub fn draw_char(&mut self, c: char, fg : Color, bg: Color) -> io::Result<()> {
        let x = 0;
        let y = 0;
        for i in 0..6 {
            let mut line = if i == 5 {
                0x00
            } else {
                FONT[((c as u16) * 5 + i) as usize]
            };
            for j in 0..8 {
                if line & 0x01 != 0 {
                    self.set_pixel((x + i) as usize, y + j, fg)?;
                } else if bg != fg {
                    self.set_pixel((x + i) as usize, y + j, bg)?;
                }
                line >>= 1;
            }
        }
        Ok(())
    }

    pub fn write(&mut self, msg : String, fg : Color, bg : Color, d : Duration) -> io::Result<()> {
        for c in msg.chars() {
            self.draw_char(c, fg, bg)?;
            self.flush()?;
            thread::sleep(d);
        }
        Ok(())
    }
}

impl Drop for Matrix {
    fn drop(&mut self) {
        let mut file = OpenOptions::new().write(true).open(&self.file_name).expect("Cannot open /dev/fb1");
        match file.write_all(&[0;192 * 2]) {
            Ok(()) => (),
            Err(_) => (),
        };
    }
}