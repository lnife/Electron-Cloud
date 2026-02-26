use nalgebra_glm as glm;
use std::f32::consts::PI;
use winit::event::{ElementState, MouseButton};

pub struct Camera {
    pub target: glm::Vec3,
    pub radius: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub orbit_speed: f32,
    pub zoom_speed: f32,
    pub dragging: bool,
    pub last_x: f64,
    pub last_y: f64,
}

impl Camera {
    pub fn new(target: glm::Vec3, radius: f32) -> Self {
        Self {
            target,
            radius,
            azimuth: 0.0,
            elevation: PI / 2.0,
            orbit_speed: 0.01,
            zoom_speed: 1.0,
            dragging: false,
            last_x: 0.0,
            last_y: 0.0,
        }
    }

    pub fn get_position(&self) -> glm::Vec3 {
        let elevation = glm::clamp_scalar(self.elevation, 0.01, PI - 0.01);
        glm::vec3(
            self.radius * elevation.sin() * self.azimuth.cos(),
            self.radius * elevation.cos(),
            self.radius * elevation.sin() * self.azimuth.sin(),
        )
    }

    pub fn get_view_matrix(&self) -> glm::Mat4 {
        glm::look_at(&self.get_position(), &self.target, &glm::vec3(0.0, 1.0, 0.0))
    }

    pub fn process_mouse_move(&mut self, x: f64, y: f64) {
        let dx = x - self.last_x;
        let dy = y - self.last_y;

        if self.dragging {
            self.azimuth += dx as f32 * self.orbit_speed;
            self.elevation -= dy as f32 * self.orbit_speed;
            self.elevation = glm::clamp_scalar(self.elevation, 0.01, PI - 0.01);
        }

        self.last_x = x;
        self.last_y = y;
    }

    pub fn process_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            if state == ElementState::Pressed {
                self.dragging = true;
            } else if state == ElementState::Released {
                self.dragging = false;
            }
        }
    }

    pub fn process_scroll(&mut self, y_offset: f64) {
        self.radius -= y_offset as f32 * self.zoom_speed;
        if self.radius < 1.0 {
            self.radius = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_zoom() {
        let mut cam = Camera::new(glm::vec3(0.0, 0.0, 0.0), 10.0);
        cam.process_scroll(2.0); // Zoom in
        assert_eq!(cam.radius, 8.0);
        cam.process_scroll(-3.0); // Zoom out
        assert_eq!(cam.radius, 11.0);
    }

    #[test]
    fn test_camera_drag_movement() {
        let mut cam = Camera::new(glm::vec3(0.0, 0.0, 0.0), 10.0);
        cam.dragging = true;
        cam.last_x = 100.0;
        cam.last_y = 100.0;

        cam.process_mouse_move(150.0, 120.0); // Drag right and down

        assert_ne!(cam.azimuth, 0.0);
        assert_ne!(cam.elevation, PI / 2.0);
    }

    #[test]
    fn test_camera_button_press_and_release() {
        let mut cam = Camera::new(glm::vec3(0.0, 0.0, 0.0), 10.0);
        assert!(!cam.dragging);
        
        // Press left mouse button
        cam.process_mouse_button(MouseButton::Left, ElementState::Pressed);
        assert!(cam.dragging);

        // Release left mouse button
        cam.process_mouse_button(MouseButton::Left, ElementState::Released);
        assert!(!cam.dragging);
    }
}
