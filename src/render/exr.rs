use render::film;

use std;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use byteorder::{LittleEndian, WriteBytesExt};

const MAGIC_NUMBER: i32 = 20000630;
const VERSION: i32 = 2;
const PIXEL_TYPE_FLOAT: i32 = 2;
const COMPRESSION_NONE: u8 = 0;
const LINE_ORDER_INCREASING_Y: u8 = 0;

pub struct ExrWriter {
    pub buffer: std::vec::Vec<u8>
}

impl ExrWriter {
    fn write_header(&mut self) {
        self.buffer.write_i32::<LittleEndian>(MAGIC_NUMBER).unwrap();
        self.buffer.write_i32::<LittleEndian>(VERSION).unwrap();
    }

    fn write_str(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
        self.buffer.push(0);
    }

    fn write_channels_attr(&mut self) {
        self.write_str("channels");
        self.write_str("chlist");

        let size: i32 =
                2 * 3 +  // Three channels named B, G, R, plus a null-terminator for each.
                16 * 3 + // Four ints (16 bytes) of data per channel.
                1;       // One extra null byte.
        self.buffer.write_i32::<LittleEndian>(size).unwrap();

        for channel in ["B", "G", "R"].iter() {
            self.write_str(channel);
            self.buffer.write_i32::<LittleEndian>(PIXEL_TYPE_FLOAT).unwrap();
            self.buffer.write_i32::<LittleEndian>(0).unwrap(); // pLinear and reserved
            self.buffer.write_i32::<LittleEndian>(1).unwrap(); // xSampling
            self.buffer.write_i32::<LittleEndian>(1).unwrap(); // ySampling
        }
        self.buffer.push(0); // Null terminator.
    }

    fn write_compression_attr(&mut self) {
        self.write_str("compression");
        self.write_str("compression");
        self.buffer.write_i32::<LittleEndian>(1).unwrap(); // Size = 1 byte.
        self.buffer.push(COMPRESSION_NONE);
    }

    fn write_data_display_window_attrs(&mut self, width: usize, height: usize) {
        let size = 4 * 4; // 4 ints = 16 bytes.
        let window = [0, 0, width as i32 - 1, height as i32 - 1];

        self.write_str("dataWindow");
        self.write_str("box2i");
        self.buffer.write_i32::<LittleEndian>(size).unwrap();
        for i in window.iter() {
            self.buffer.write_i32::<LittleEndian>(*i).unwrap();
        }

        self.write_str("displayWindow");
        self.write_str("box2i");
        self.buffer.write_i32::<LittleEndian>(size).unwrap();
        for i in window.iter() {
            self.buffer.write_i32::<LittleEndian>(*i).unwrap();
        }
    }

    fn write_line_order_attr(&mut self) {
        self.write_str("lineOrder");
        self.write_str("lineOrder");
        self.buffer.write_i32::<LittleEndian>(1).unwrap(); // Size = 1 byte.
        self.buffer.push(LINE_ORDER_INCREASING_Y);
    }

    fn write_pixel_aspect_ratio_attr(&mut self) {
        self.write_str("pixelAspectRatio");
        self.write_str("float");
        self.buffer.write_i32::<LittleEndian>(4).unwrap(); // 1 float = 4 bytes.
        self.buffer.write_f32::<LittleEndian>(1.0).unwrap();
    }

    fn write_screen_window_center_attr(&mut self) {
        self.write_str("screenWindowCenter");
        self.write_str("v2f");
        self.buffer.write_i32::<LittleEndian>(8).unwrap(); // 2 floats = 8 bytes.
        self.buffer.write_f32::<LittleEndian>(0.0).unwrap();
        self.buffer.write_f32::<LittleEndian>(0.0).unwrap();
    }

    fn write_screen_window_width(&mut self, width: usize) {
        self.write_str("screenWindowWidth");
        self.write_str("float");
        self.buffer.write_i32::<LittleEndian>(4).unwrap(); // 1 float = 4 bytes.
        self.buffer.write_f32::<LittleEndian>(width as f32).unwrap();
    }

    fn write_line_offset_table(&mut self, film: &film::Film) {
        let size_of_table = 8 * film.height; // 1 ulong (8 bytes) per line.
        let data_offset = self.buffer.len() + size_of_table;
        let line_header_size = 4 + 4; // Scan line number (int) and bytes in line (uint).
        let line_data_size = film.width * 4 * 3; // 1 float (4 bytes) for 3 channels per pixel.
        let line_full_size = line_header_size + line_data_size;

        for y in 0..film.height {
            let line_offset = data_offset + y * line_full_size;
            self.buffer.write_u64::<LittleEndian>(line_offset as u64).unwrap();
        }

        debug_assert!(self.buffer.len() == data_offset);
    }

    fn write_channels(&mut self, film: &film::Film) {
        let line_data_size = film.width * 4 * 3; // 1 float (4 bytes) for 3 channels per pixel.

        // For each line in the image...
        let mut i = (film.pixels.len() - film.width) as isize;
        let mut line = 0i32;
        while i >= 0 {
            self.buffer.write_i32::<LittleEndian>(line).unwrap(); // Scan line number
            self.buffer.write_u32::<LittleEndian>(line_data_size as u32).unwrap(); // Bytes in line

            let len_before = self.buffer.len();

            // For each channel in BGR order...
            for channel in [2, 1, 0usize].iter() {
                // Write channel value for all pixels in line.
                for j in i..(i + film.width as isize) {
                    let pixel = &film.pixels[j as usize];
                    let val = &pixel.accum / pixel.weight;
                    self.buffer.write_f32::<LittleEndian>(val[*channel] as f32).unwrap();
                }
            }

            debug_assert!(self.buffer.len() - len_before == line_data_size);

            i -= film.width as isize;
            line += 1;
        }
    }

    pub fn store(&mut self, film: &film::Film) {
        self.buffer.clear();

        // Begin header.
        self.write_header();
        self.write_channels_attr();
        self.write_compression_attr();
        self.write_data_display_window_attrs(film.width, film.height);
        self.write_line_order_attr();
        self.write_pixel_aspect_ratio_attr();
        self.write_screen_window_center_attr();
        self.write_screen_window_width(film.width);
        self.buffer.push(0); // End header.

        // Begin line offset table.
        self.write_line_offset_table(film); // End line offset table.

        // Begin data.
        self.write_channels(film); // End data.
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) {
        let mut buffer = File::create(path).unwrap();
        buffer.write(&self.buffer).unwrap();
    }
}
