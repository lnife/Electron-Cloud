use nalgebra_glm as glm;
use std::f32::consts::PI;

// generates vertices for a triangulated sphere mesh
// this is purely geometric and used as the base model for instanced rendering
// each particle is rendered as a small scaled copy of this sphere
pub fn generate_sphere(radius: f32, sectors: u32, stacks: u32) -> Vec<glm::Vec3> {
    let mut vertices = Vec::new();

    // angular step sizes
    let sector_step = 2.0 * PI / sectors as f32; // horizontal slices
    let stack_step = PI / stacks as f32; // vertical slices

    // generate sphere vertices in spherical coordinates
    for i in 0..=stacks {
        // stack angle moves from +pi/2 to -pi/2
        let stack_angle = PI / 2.0 - i as f32 * stack_step;

        // projection of radius onto xy plane
        let xy = radius * stack_angle.cos();

        // vertical coordinate
        let z = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;

            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            vertices.push(glm::vec3(x, y, z));
        }
    }

    // build triangle indices connecting stacks and sectors
    let mut indices = Vec::new();

    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;

        for _ in 0..sectors {
            // first triangle (except top stack)
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }

            // second triangle (except bottom stack)
            if i != (stacks - 1) {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    // expand indexed mesh into flat triangle list
    // wgpu pipeline expects direct vertex list here
    let mut sphere_vertices = Vec::new();

    for index in indices {
        sphere_vertices.push(vertices[index as usize]);
    }

    sphere_vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sphere_creates_vertices() {
        let vertices = generate_sphere(1.0, 10, 10);

        // simple sanity check to ensure geometry is produced
        assert!(
            !vertices.is_empty(),
            "generate_sphere should produce vertices"
        );
    }
}
