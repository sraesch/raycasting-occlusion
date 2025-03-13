use macroquad::input::MouseButton;
use nalgebra_glm::{Mat4, Vec3, length, mat4_to_mat3, rotate};
use occ_raycasting::math::AABB;

use crate::camera_data::CameraData;

use anyhow::Result;

#[derive(Debug)]
enum Mode {
    Nothing,
    Zoom,
    Move,
    Rotate,
}

pub struct Camera {
    data: CameraData,
    mode: Mode,
    save_cursor: [f64; 2],
    saved_data: CameraData,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            data: CameraData::new(),
            mode: Mode::Nothing,
            save_cursor: [0.0, 0.0],
            saved_data: CameraData::new(),
        }
    }

    /// Updates the window size
    ///
    ///* `w` - The new width of the window
    ///* `h` - The new height of the window
    pub fn update_window_size(&mut self, w: u32, h: u32) {
        self.data.set_window_size(w, h);
    }

    pub fn update_mouse_button(&mut self, x: f64, y: f64, btn: MouseButton, pressed: bool) {
        if pressed {
            self.save_cursor[0] = x;
            self.save_cursor[1] = y;

            self.saved_data = self.data.clone();

            match btn {
                MouseButton::Right => self.mode = Mode::Zoom,
                MouseButton::Middle => self.mode = Mode::Move,
                MouseButton::Left => self.mode = Mode::Rotate,
                _ => {}
            }
        } else {
            self.modify(x, y);
            self.mode = Mode::Nothing;
        }
    }

    pub fn update_mouse_motion(&mut self, x: f64, y: f64) {
        self.modify(x, y);
    }

    /// Sets the internal camera radius
    ///
    ///* `radius` - The new radius.
    #[inline]
    pub fn set_radius(&mut self, radius: f32) {
        self.data.set_radius(radius.ln())
    }

    /// Focuses the camera on the given scene volume
    ///
    ///* `volume` - The scene volume for the camera to focus on
    pub fn focus(&mut self, volume: &AABB) -> Result<()> {
        let center = volume.get_center();
        let size = volume.get_size();
        let box_size = length(&size);

        self.set_radius(box_size * 1.5);

        let camera_data = &mut self.data;
        camera_data.set_center(&center);

        let scene_center = volume.get_center();
        let scene_radius = length(&volume.get_size()) / 2f32;
        camera_data.set_scene(scene_center, scene_radius)?;

        Ok(())
    }

    fn modify(&mut self, new_x: f64, new_y: f64) {
        let x_drift_func = || {
            return ((new_x - self.save_cursor[0]) as f32) / (self.data.get_window_size().0 as f32);
        };

        let y_drift_func = || {
            return ((new_y - self.save_cursor[1]) as f32) / (self.data.get_window_size().1 as f32);
        };

        match self.mode {
            Mode::Zoom => {
                let y_diff = y_drift_func() * 2.0;
                let new_radius = self.saved_data.get_radius() + y_diff;
                self.data.set_radius(new_radius);
            }
            Mode::Move => {
                let cam_axis = self.data.get_axis();

                let x_axis = cam_axis.column(0);
                let y_axis = cam_axis.column(1);

                let factor = self.data.get_radius().exp();

                let x_drift = -x_drift_func() * factor;
                let y_drift = y_drift_func() * factor;

                let new_center =
                    *self.saved_data.get_center() + x_axis * x_drift + y_axis * y_drift;
                self.data.set_center(&new_center);
            }
            Mode::Rotate => {
                let x_drift = x_drift_func();
                let y_drift = y_drift_func();

                let cam_axis = self.saved_data.get_axis();

                let axis0: Vec3 = cam_axis.column(0).into();
                let axis1: Vec3 = cam_axis.column(1).into();

                let x_rot_mat = rotate(&Mat4::identity(), -x_drift * 2.5, &axis1);
                let rot_mat = rotate(&x_rot_mat, -y_drift * 2.5, &axis0);

                let rot_mat = mat4_to_mat3(&rot_mat);

                self.data
                    .set_rotated_cam_axis(self.saved_data.get_axis(), &rot_mat);
            }
            Mode::Nothing => {}
        }
    }

    /// Returns reference onto the internal camera data
    pub fn get_data(&self) -> &CameraData {
        &self.data
    }
}
