use std::io::{BufWriter, Write};

use log::debug;
use nalgebra_glm::Vec3;

use crate::Error;

#[derive(Clone)]
pub struct Frame {
    width: usize,
    height: usize,

    /// The id-buffer contains per pixel ids
    id_buffer: Vec<Option<u32>>,

    /// The depth buffer contains the per pixel depth.
    /// The depth buffer is optional.
    depth_buffer: Option<Vec<f32>>,
}

impl Frame {
    /// Creates a new empty frame with the given width and height.
    ///
    /// # Arguments
    /// * `width` - The width of the frame.
    /// * `height` - The height of the frame.
    /// * `with_depths` - If true, the frame will contain a depth buffer.
    pub fn new_empty(width: usize, height: usize, with_depths: bool) -> Self {
        let id_buffer: Vec<Option<u32>> = vec![None; width * height];

        let depth_buffer = if with_depths {
            Some(vec![0f32; width * height])
        } else {
            None
        };

        Self {
            width,
            height,
            id_buffer,
            depth_buffer,
        }
    }

    /// Returns the width of the frame.
    #[inline]
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Returns the height of the frame.
    #[inline]
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Returns the id-buffer of the frame. That is, the buffer containing a per pixel id.
    #[inline]
    pub fn get_id_buffer(&self) -> &[Option<u32>] {
        &self.id_buffer
    }

    /// Returns the id-buffer of the frame. That is, the buffer containing a per pixel id.
    /// The buffer is mutable.
    #[inline]
    pub fn get_id_buffer_mut(&mut self) -> &mut [Option<u32>] {
        &mut self.id_buffer
    }

    /// Returns the depth-buffer of the frame. That is, the buffer containing a per pixel depth.
    #[inline]
    pub fn get_depth_buffer(&self) -> Option<&[f32]> {
        self.depth_buffer.as_deref()
    }

    /// Returns the depth-buffer of the frame. That is, the buffer containing a per pixel depth.
    /// The buffer is mutable.
    #[inline]
    pub fn get_depth_buffer_mut(&mut self) -> Option<&mut [f32]> {
        self.depth_buffer.as_deref_mut()
    }

    /// Writes the depths of the given frame as PGM file with gray colors.
    ///
    /// # Arguments
    /// * `writer` - The writer to which the depth-buffer will be serialized as PGM.
    pub fn write_depth_buffer_as_pgm<W: Write>(&self, writer: W) -> Result<(), Error> {
        let mut out = BufWriter::new(writer);

        let depths = self.get_depth_buffer().unwrap();
        let ids = self.get_id_buffer();

        // determine min/max
        let (min, max) = if depths.is_empty() {
            (0f32, 1f32)
        } else {
            let mut min = f32::MAX;
            let mut max = 0f32;

            for (depth, id) in depths.iter().zip(ids.iter()) {
                match id {
                    Some(_) => {
                        min = min.min(*depth);
                        max = max.max(*depth);
                    }
                    None => {}
                }
            }

            (min, max)
        };

        debug!("Writing depth buffer: Min/Max={}/{}", min, max);

        writeln!(out, "P2")?;
        writeln!(out, "{} {}", self.get_width(), self.get_height())?;
        writeln!(out, "255")?;

        ids.iter()
            .zip(depths.iter())
            .map(|(id, depth)| match id {
                Some(_) => {
                    if max > min {
                        ((1f32 - ((*depth - min) / (max - min))) * 255f32).round() as u32
                    } else {
                        128u32
                    }
                }
                None => 0,
            })
            .enumerate()
            .try_for_each(|(index, depth)| -> std::io::Result<()> {
                write!(out, "{} ", depth)?;

                if index > 0 && index % self.get_width() == 0 {
                    writeln!(out)?;
                }

                Ok(())
            })?;

        Ok(())
    }

    /// Writes the depths of the given frame as PGM file with gray colors.
    ///
    /// # Arguments
    /// * `writer` - The writer to which the depth-buffer will be serialized as PGM.
    /// * `create_palette` - Callback for creating color palette for the given number of ids.
    ///
    pub fn write_id_buffer_as_ppm<W, F>(
        &self,
        writer: W,
        mut create_palette: F,
    ) -> Result<(), Error>
    where
        W: Write,
        F: FnMut(usize) -> Vec<Vec3>,
    {
        let mut out = BufWriter::new(writer);

        let ids = self.get_id_buffer();

        // determine the maximal id
        let num_ids: usize = if ids.is_empty() {
            0
        } else {
            let n: u32 = ids.iter().map(|id| id.unwrap_or(0)).max().unwrap();
            (n as usize) + 1
        };

        let colors = create_palette(num_ids);
        assert_eq!(colors.len(), num_ids);

        writeln!(out, "P3")?;
        writeln!(out, "{} {}", self.get_width(), self.get_height())?;
        writeln!(out, "255")?;

        ids.iter()
            .map(|id| match id {
                Some(id) => colors[*id as usize],
                None => Vec3::new(0f32, 0f32, 0f32),
            })
            .enumerate()
            .try_for_each(|(index, color)| -> std::io::Result<()> {
                let r = (color[0] * 255f32) as u32;
                let g = (color[1] * 255f32) as u32;
                let b = (color[2] * 255f32) as u32;

                write!(out, "{} {} {} ", r, g, b)?;

                if index > 0 && index % self.get_width() == 0 {
                    writeln!(out)?;
                }

                Ok(())
            })?;

        Ok(())
    }

    /// Writes the frame as binary data.
    ///
    /// # Arguments
    /// * `w` - The writer to which the frame will be serialized as binary data.
    pub fn write_binary<W: Write>(&self, mut w: W) -> Result<(), Error> {
        let width = self.get_width() as u32;
        let height = self.get_height() as u32;

        let has_depth: u32 = if self.get_depth_buffer().is_some() {
            1
        } else {
            0
        };

        w.write_all(&width.to_le_bytes())?;
        w.write_all(&height.to_le_bytes())?;
        w.write_all(&has_depth.to_le_bytes())?;

        // write ids
        for id in self.get_id_buffer() {
            match id {
                Some(id) => {
                    let id = id.to_le_bytes();
                    w.write_all(&id)?;
                }
                None => {
                    // write max u32 value for None
                    let id = u32::MAX.to_le_bytes();
                    w.write_all(&id)?;
                }
            }
        }

        // write depths
        if let Some(depths) = self.get_depth_buffer() {
            for depth in depths {
                let depth = depth.to_le_bytes();
                w.write_all(&depth)?;
            }
        }

        Ok(())
    }

    /// Reads the frame from binary data.
    ///
    /// # Arguments
    /// * `r` - The reader from which the frame will be deserialized.
    pub fn read_binary<R: std::io::Read>(mut r: R) -> Result<Self, Error> {
        let mut buffer = [0u8; 4];

        r.read_exact(&mut buffer)?;
        let width = u32::from_le_bytes(buffer) as usize;

        r.read_exact(&mut buffer)?;
        let height = u32::from_le_bytes(buffer) as usize;

        r.read_exact(&mut buffer)?;
        let has_depth = u32::from_le_bytes(buffer);

        let mut id_buffer = vec![None; width * height];

        // read ids
        for id in id_buffer.iter_mut() {
            r.read_exact(&mut buffer)?;
            let id_value = u32::from_le_bytes(buffer);

            if id_value == u32::MAX {
                *id = None;
            } else {
                *id = Some(id_value);
            }
        }

        // read depths
        let mut depth_buffer = None;
        if has_depth == 1 {
            let mut depth_buffer_vec = vec![0f32; width * height];

            for depth in depth_buffer_vec.iter_mut() {
                r.read_exact(&mut buffer)?;
                *depth = f32::from_le_bytes(buffer);
            }

            depth_buffer = Some(depth_buffer_vec);
        }

        Ok(Self {
            width,
            height,
            id_buffer,
            depth_buffer,
        })
    }
}
