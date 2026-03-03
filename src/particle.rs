use nalgebra_glm as glm;

// simple data container representing one rendered particle
// position is in world space
// color is precomputed intensity mapped from probability density
pub struct Particle {
    pub position: glm::Vec3,
    pub color: glm::Vec4,
}
