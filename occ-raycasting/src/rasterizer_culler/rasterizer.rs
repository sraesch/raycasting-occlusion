use nalgebra_glm::Vec3;

use crate::math::clamp;

use super::{DepthBufferPrecisionType, Frame};

/// A simple rasterizer that rasterizes pixels into a frame buffer.
pub struct Rasterizer<D: DepthBufferPrecisionType> {
    /// The width of the frame buffer.
    pub width: usize,

    /// The height of the frame buffer.
    pub height: usize,

    /// The depth buffer of the rasterizer.
    pub depth_buffer: Vec<D>,

    /// The id buffer of the rasterizer.
    pub id_buffer: Vec<Option<u32>>,
}

impl<D: DepthBufferPrecisionType> Rasterizer<D> {
    /// Creates and returns a new rasterizer.
    ///
    /// # Arguments
    /// * `width` - The width of the frame buffer.
    /// * `height` - The height of the frame buffer.
    pub fn new(width: usize, height: usize) -> Self {
        let depth_buffer = vec![D::MAX; width * height];
        let id_buffer = vec![None; width * height];

        Self {
            width,
            height,
            depth_buffer,
            id_buffer,
        }
    }

    /// Returns the id buffer of the rasterizer.
    pub fn get_frame(&self) -> Frame {
        let mut frame = Frame::new_empty(self.width, self.height, true);

        let src_id_buffer = self.id_buffer.as_slice();
        let dst_id_buffer = frame.get_id_buffer_mut();

        for (src_id, dst_id) in src_id_buffer.iter().zip(dst_id_buffer.iter_mut()) {
            *dst_id = *src_id;
        }

        let src_depth_buffer = self.depth_buffer.as_slice();
        let dst_depth_buffer = frame.get_depth_buffer_mut().unwrap();

        for (src_depth, dst_depth) in src_depth_buffer.iter().zip(dst_depth_buffer.iter_mut()) {
            *dst_depth = src_depth.to_f32();
        }

        frame
    }

    /// Clears the framebuffer.
    #[inline]
    pub fn clear(&mut self) {
        self.depth_buffer.clear();
        self.id_buffer.fill(None);
    }

    /// Rasterizes the triangle given in its window coordinates.
    ///
    /// # Arguments
    /// * `id` - The object id to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    pub fn rasterize(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        // sort the vertices in ascending order with respect to their y coordinate

        if p0.y <= p1.y && p0.y <= p2.y {
            // case 1: p0 has smallest y-coordinate
            if p1.y <= p2.y {
                self.fill_triangle(id, p0, p1, p2);
            } else {
                self.fill_triangle(id, p0, p2, p1);
            }
        } else if p1.y <= p0.y && p1.y <= p2.y {
            // case 2: p1 has smallest y-coordinate
            if p0.y <= p2.y {
                self.fill_triangle(id, p1, p0, p2);
            } else {
                self.fill_triangle(id, p1, p2, p0);
            }
        } else {
            // case 3: p2 has smallest y-coordinate
            if p0.y <= p1.y {
                self.fill_triangle(id, p2, p0, p1);
            } else {
                self.fill_triangle(id, p2, p1, p0);
            }
        }
    }

    /// Rasterizes the given triangle with the assumption that the points are sorted in ascending
    /// order with respect to their y-coordinates.
    ///
    /// # Arguments
    /// * `id` - The object id to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    fn fill_triangle(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        let (y0, y1, y2) = (p0[1], p1[1], p2[1]);

        debug_assert!(y0 <= y1 && y1 <= y2);

        if y0.round() == y2.round() {
            // check special case, where the triangle is a line
            let y = y0.round();

            // make sure that the line is inside the frame
            if y >= 0f32 && y < self.height as f32 {
                let y = y as usize;

                let (x0, x1, depth0, depth1) = if p0.x <= p2.x {
                    (p0.x, p2.x, p0.z, p2.z)
                } else {
                    (p2.x, p0.x, p2.z, p0.z)
                };

                self.draw_scanline(id, y, x0, x1, depth0, depth1);
            }
        } else if y0.round() == y1.round() {
            // check for top-flat case
            self.fill_top_flat_triangle(id, p0, p1, p2);
        } else if y1.round() == y2.round() {
            // check for bottom-flat case
            self.fill_bottom_flat_triangle(id, p0, p1, p2);
        } else {
            // ok we have that the y-coordinates define a strict ascending order
            // thus we split the triangle in a bottom and top flat triangle, but need to define
            // a new point p3

            let lambda = (y1 - y0) / (y2 - y0);
            assert!(
                (0f32..=1f32).contains(&lambda),
                "Lambda must be between 0 and 1, but is {}. y0={}, y1={}, y2={}",
                lambda,
                y0,
                y1,
                y2
            );

            let x3 = p0[0] + lambda * (p2[0] - p0[0]);
            let z3 = p0[2] + lambda * (p2[2] - p0[2]);

            let p3 = Vec3::new(x3, y1, z3);

            self.fill_bottom_flat_triangle(id, p0, p1, &p3);
            self.fill_top_flat_triangle(id, p1, &p3, p2);
        }
    }

    /// Draws a triangle with a horizontal bottom, i.e. p1[1] == p2[1]
    ///
    /// # Arguments
    /// * `id` - The object id to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    fn fill_bottom_flat_triangle(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        let max_y = self.height as f32 - 1f32;

        // p1 and p2 are both on the same height and p0 is at least lower or equal
        debug_assert!(p1[1].round() == p2[1].round());
        debug_assert!(p0[1] <= p1[1]);
        let y1 = p2[1];

        // if p0 is not strictly lower, then the triangle is degenerated and we won't draw it
        if p0[1] == p1[1] {
            return;
        }

        let y0 = p0[1];

        debug_assert!(y0 < y1);

        // sort out extreme cases
        if y1 < 0f32 || y0 > max_y {
            return;
        }

        // clamp y0 and y1 s.t. they fit into the current frame
        let y0m = y0.round().max(0f32) as usize;
        let y1m = y1.round().min(max_y) as usize;

        // compute the start and end of the bottom
        let (left_x, right_x, left_depth, right_depth) = if p1[0] < p2[0] {
            (p1[0], p2[0], p1[2], p2[2])
        } else {
            (p2[0], p1[0], p2[2], p1[2])
        };

        for y in y0m..=y1m {
            let yf = clamp((y as f32 - y0) / (y1 - y0), 0f32, 1f32);
            debug_assert!((0f32..=1f32).contains(&yf));

            let x0 = p0[0] + yf * (left_x - p0[0]);
            let x1 = p0[0] + yf * (right_x - p0[0]);
            debug_assert!(x0 <= x1);

            let depth0 = p0[2] + yf * (left_depth - p0[2]);
            let depth1 = p0[2] + yf * (right_depth - p0[2]);

            self.draw_scanline(id, y, x0, x1, depth0, depth1);
        }
    }

    /// Draws a triangle with a horizontal top, i.e. p0[1] == p1[1]
    ///
    /// # Arguments
    /// * `id` - The id of the object to which the triangle belongs to.
    /// * `p0` - The first vertex of the triangle in window coordinates.
    /// * `p1` - The second vertex of the triangle in window coordinates.
    /// * `p2` - The third vertex of the triangle in window coordinates.
    fn fill_top_flat_triangle(&mut self, id: u32, p0: &Vec3, p1: &Vec3, p2: &Vec3) {
        let max_y = self.height as f32 - 1f32;

        // p0 and p1 are both on the same height and p2 is at least higher or equal
        debug_assert!(p0[1].round() == p1[1].round());
        debug_assert!(p1[1] <= p2[1]);
        let y1 = p2[1];

        // if p2 is not strictly higher, then the triangle is degenerated and we won't draw it
        if p2[1] == p0[1] {
            return;
        }

        let y0 = p0[1];

        debug_assert!(y0 < y1);

        // sort out extreme cases
        if y1 < 0f32 || y0 > max_y {
            return;
        }

        // clamp y0 and y1 s.t. they fit into the current frame
        let y0m = y0.round().max(0f32) as usize;
        let y1m = y1.round().min(max_y) as usize;

        // compute the start and end of the top
        let (left_x, right_x, left_depth, right_depth) = if p0[0] < p1[0] {
            (p0[0], p1[0], p0[2], p1[2])
        } else {
            (p1[0], p0[0], p1[2], p0[2])
        };

        // draw the scan lines
        for y in y0m..=y1m {
            let yf = clamp((y1 - y as f32) / (y1 - y0), 0f32, 1f32);
            debug_assert!((0f32..=1f32).contains(&yf));

            let x0 = p2[0] + yf * (left_x - p2[0]);
            let x1 = p2[0] + yf * (right_x - p2[0]);
            debug_assert!(x0 <= x1);

            let depth0 = p2[2] + yf * (left_depth - p2[2]);
            let depth1 = p2[2] + yf * (right_depth - p2[2]);

            self.draw_scanline(id, y, x0, x1, depth0, depth1);
        }
    }

    /// Draws a single horizontal line at y-th position from x0 to x1 with given respective depth
    /// values depth0 and depth1.
    ///
    /// # Arguments
    /// * `id` - The id of the object to which the scan-line belongs to.
    /// * `y` - The y-value of the horizontal line.
    /// * `x0` - The left x-value of the line
    /// * `x1` - The right x-value of the line
    /// * `depth0` - The depth-value of the left side of the line.
    /// * `depth1` - The depth-value of the right side of the line.
    fn draw_scanline(&mut self, id: u32, y: usize, x0: f32, x1: f32, depth0: f32, depth1: f32) {
        debug_assert!(y < self.height);
        debug_assert!(x0 <= x1);

        let x0 = x0.round();
        let x1 = x1.round();

        let max_x = self.width as f32 - 1f32;

        // check special case where the line is completely out of the frame
        if x1 < 0f32 || x0 > max_x {
            return;
        }

        // clamp line to the window coordinates
        let x0m = x0.round().max(0f32) as usize;
        let x1m = x1.round().min(max_x) as usize;
        let dd: f32 = if x1 > x0 {
            (depth1 - depth0) / (x1 - x0)
        } else {
            0f32
        };

        for x in x0m..=x1m {
            let depth = depth0 + ((x as f32) - x0) * dd;
            self.draw_pixel(id, x, y, depth);
        }
    }

    /// Draws a single pixel with the given id and depth and checks if the pixel is within bounds.
    ///
    /// # Arguments
    /// * `id` - The id of the pixel.
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `depth` - The depth value of the pixel.
    #[inline]
    fn draw_pixel(&mut self, id: u32, x: usize, y: usize, depth: f32) {
        debug_assert!(x < self.width || y < self.height);

        // compute pixel index
        let index = y * self.width + x;

        // make sure depth is within bounds and valid
        if !(0f32..=1f32).contains(&depth) || depth.is_infinite() || depth.is_nan() {
            return;
        }

        let depth = D::from_f32(depth);
        let ref_depth = &mut self.depth_buffer[index];

        if depth < *ref_depth {
            *ref_depth = depth;
            self.id_buffer[index] = Some(id);
        }
    }
}

#[cfg(test)]
mod test {
    use nalgebra_glm::Vec3;

    use super::*;

    /// Small helper function to compute the area for the given triangle.
    fn compute_triangle_area(p0: &Vec3, p1: &Vec3, p2: &Vec3) -> f32 {
        let a: Vec3 = p1 - p0;
        let b: Vec3 = p2 - p0;

        a.cross(&b).norm() / 2f32
    }

    #[test]
    fn test_fill_bottom_flat_triangle() {
        let size = 128;

        let mut r = Rasterizer::<u32>::new(size, size);

        let id = 42;

        let p0 = Vec3::new(20f32, 10f32, 0.5f32);
        let p1 = Vec3::new(40f32, 40f32, 0.5f32);
        let p2 = Vec3::new(10f32, 40f32, 0.5f32);

        r.fill_bottom_flat_triangle(id, &p0, &p1, &p2);

        let area = compute_triangle_area(&p0, &p1, &p2);

        let num_ids = r.id_buffer.iter().filter(|i| **i == Some(id)).count();
        println!("Num Ids: {}", num_ids);
        println!("Triangle Area: {}", area);

        let height = 30f32;
        let error_eps = height * 2f32;

        assert!(((num_ids as f32) - area).abs() <= error_eps);

        let mut last_line_length: usize = 0;

        for y in 0..size {
            let mut x_start = size;
            let mut x_end: usize = 0;

            for x in 0..size {
                let id = r.id_buffer[y * size + x];
                if id == Some(42) {
                    x_start = x_start.min(x);
                    x_end = x_end.max(x);
                }
            }

            let line_length = if x_start > x_end {
                0
            } else {
                x_end - x_start + 1
            };

            assert!(!(10..=40).contains(&y) || line_length > 0);
            assert!(!(10..=40).contains(&y) || last_line_length <= line_length);

            last_line_length = line_length;
        }
    }

    #[test]
    fn test_fill_top_flat_triangle() {
        let size = 128;

        let mut r = Rasterizer::<u32>::new(size, size);

        let id = 42;

        let p0 = Vec3::new(40f32, 10f32, 0.5f32);
        let p1 = Vec3::new(10f32, 10f32, 0.5f32);
        let p2 = Vec3::new(20f32, 40f32, 0.5f32);

        r.fill_top_flat_triangle(id, &p0, &p1, &p2);

        let area = compute_triangle_area(&p0, &p1, &p2);

        let num_ids = r.id_buffer.iter().filter(|i| **i == Some(id)).count();
        println!("Num Ids: {}", num_ids);
        println!("Triangle Area: {}", area);

        let height = 30f32;
        let error_eps = height * 2f32;

        assert!(((num_ids as f32) - area).abs() <= error_eps);

        let mut last_line_length: usize = size;

        for y in 0..size {
            let mut x_start = size;
            let mut x_end: usize = 0;

            for x in 0..size {
                let id = r.id_buffer[y * size + x];
                if id == Some(42) {
                    x_start = x_start.min(x);
                    x_end = x_end.max(x);
                }
            }

            let line_length = if x_start > x_end {
                0
            } else {
                x_end - x_start + 1
            };

            assert!(!(10..=40).contains(&y) || line_length > 0);
            assert!(y <= 10 || y > 40 || last_line_length >= line_length);

            last_line_length = line_length;
        }
    }
}
