use lazy_static::lazy_static;
use nalgebra_glm as glm;
use rand::Rng;
use statrs::function::gamma;
use std::f64::consts::PI;
use std::sync::Mutex;

// data structures and constants

pub struct Particle {
    // public so main.rs can construct and read sampled particles
    pub position: glm::DVec3,
    pub color: glm::Vec4,
}

const A0: f64 = 1.0; // bohr radius set to 1 (atomic units for simplicity)

// global quantum numbers defining the current hydrogenic orbital
// wrapped in mutex because they are set at runtime before sampling
// lazy_static is used since mutex cannot be const-initialized
lazy_static! {
    pub static ref N: Mutex<i32> = Mutex::new(2);
    pub static ref L: Mutex<i32> = Mutex::new(1);
    pub static ref M: Mutex<i32> = Mutex::new(0);
    pub static ref USE_REAL_SPHERICAL_HARMONICS: Mutex<bool> = Mutex::new(true);
}

// particle generation

// generates monte carlo samples of the hydrogenic orbital defined by (n, l, m)
// radial and angular parts are sampled independently and then converted to cartesian space
pub fn generate_particles(num_particles: usize) -> Vec<Particle> {
    // read currently selected quantum numbers
    let n = *N.lock().unwrap();
    let l = *L.lock().unwrap();
    let m = *M.lock().unwrap();
    let use_real_spherical_harmonics = *USE_REAL_SPHERICAL_HARMONICS.lock().unwrap();

    // preallocate memory to avoid repeated reallocations
    let mut particles = Vec::with_capacity(num_particles);

    for _ in 0..num_particles {
        // sample radial and angular coordinates. Real harmonics use the standard
        // z-axis spherical convention; the complex visualization keeps the
        // original renderer convention for compatibility with the moving path.
        let r = sample_r(n, l);
        let (pos, color) = if use_real_spherical_harmonics {
            let direction = sample_real_direction(l, m);
            (
                glm::vec3(direction.x * r, direction.y * r, direction.z * r),
                get_particle_color_real(r, &direction, n, l, m),
            )
        } else {
            let theta = sample_theta(l, m);
            let phi = sample_phi_uniform();
            (
                spherical_to_cartesian(r, theta, phi),
                get_particle_color_complex(r, theta, n, l, m),
            )
        };

        particles.push(Particle {
            position: pos,
            color,
        });
    }

    particles
}

// converts spherical coordinates (r, theta, phi) to cartesian
// theta is measured from +y axis, phi rotates around y axis
pub fn spherical_to_cartesian(r: f64, theta: f64, phi: f64) -> glm::DVec3 {
    let x = r * theta.sin() * phi.cos();
    let y = r * theta.cos();
    let z = r * theta.sin() * phi.sin();
    glm::vec3(x, y, z)
}

pub fn advance_particles(particles: &mut [Particle], dt: f64) {
    if *USE_REAL_SPHERICAL_HARMONICS.lock().unwrap() {
        return;
    }

    let m = *M.lock().unwrap();
    // if m == 0 {
    //     return;
    // }
    // let m = m as f64;
    let m = if m == 0 { 1.0 } else { m as f64 };

    for particle in particles {
        let x = particle.position.x;
        let y = particle.position.y;
        let z = particle.position.z;
        let r = (x * x + y * y + z * z).sqrt();

        if r <= 1e-6 {
            continue;
        }

        let min_orbit_radius = r * 1e-4;
        let orbit_radius_sq = (x * x + z * z).max(min_orbit_radius * min_orbit_radius);
        let delta_phi = m * dt / orbit_radius_sq;
        let (sin_phi, cos_phi) = delta_phi.sin_cos();

        particle.position.x = x * cos_phi - z * sin_phi;
        particle.position.z = x * sin_phi + z * cos_phi;
    }
}

// physics calculations and sampling

// Complex spherical harmonic phi sampler.
// Kept here for reference. It made phi uniform because |exp(i m phi)|^2 = 1.
//
// fn sample_phi() -> f64 {
//     let mut rng = rand::thread_rng();
//     rng.gen_range(0.0..2.0 * PI)
// }

fn sample_phi_uniform() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..2.0 * PI)
}

// General real spherical harmonic sampler.
// Samples a unit direction from |Y_lm_real(theta, phi)|^2 for any valid l,m.
fn sample_real_direction(l: i32, m: i32) -> glm::DVec3 {
    let max_density = real_angular_density_max(l, m);
    let mut rng = rand::thread_rng();

    loop {
        let z = rng.gen_range(-1.0..1.0);
        let phi = rng.gen_range(0.0..2.0 * PI);
        let xy = (1.0_f64 - z * z).sqrt();
        let direction = glm::vec3(xy * phi.cos(), xy * phi.sin(), z);
        let (theta, phi) = standard_angles_from_direction(&direction);
        let harmonic = real_spherical_harmonic(l, m, theta, phi);
        let density = harmonic * harmonic;

        if rng.gen::<f64>() <= density / max_density {
            return direction;
        }
    }
}

fn standard_angles_from_direction(direction: &glm::DVec3) -> (f64, f64) {
    let theta = direction.z.clamp(-1.0, 1.0).acos();
    let phi = direction.y.atan2(direction.x);
    (theta, phi)
}

fn real_angular_density_max(l: i32, m: i32) -> f64 {
    lazy_static! {
        static ref CACHE: Mutex<Vec<(i32, i32, f64)>> = Mutex::new(Vec::new());
    }

    let mut cache = CACHE.lock().unwrap();

    if let Some((_, _, max_density)) = cache.iter().find(|(cl, cm, _)| *cl == l && *cm == m) {
        return *max_density;
    }

    let mut max_density = 0.0;
    let theta_steps = 180;
    let phi_steps = 360;

    for i in 0..=theta_steps {
        let theta = PI * i as f64 / theta_steps as f64;
        for j in 0..phi_steps {
            let phi = 2.0 * PI * j as f64 / phi_steps as f64;
            let harmonic = real_spherical_harmonic(l, m, theta, phi);
            let density = harmonic * harmonic;
            if density > max_density {
                max_density = density;
            }
        }
    }

    // Small safety factor avoids rare rejection probabilities above 1 due to grid sampling.
    max_density = (max_density * 1.01).max(1e-12);
    cache.push((l, m, max_density));
    max_density
}

// sample radial coordinate using inverse transform sampling
// builds and caches a cdf for each (n, l) pair
fn sample_r(n: i32, l: i32) -> f64 {
    // cache stores precomputed radial cdfs keyed by (n, l)
    lazy_static! {
        static ref CDF_CACHE: Mutex<Vec<(i32, i32, Vec<f64>)>> = Mutex::new(Vec::new());
    }

    let mut cache = CDF_CACHE.lock().unwrap();

    // reuse cached cdf if available
    if let Some(entry) = cache.iter().find(|(cn, cl, _)| *cn == n && *cl == l) {
        let cdf = &entry.2;
        let u: f64 = rand::thread_rng().gen();

        // inverse transform sampling via binary search
        let idx = match cdf.binary_search_by(|v| v.partial_cmp(&u).unwrap()) {
            Ok(i) => i,
            Err(i) => i,
        };

        let r_max = 10.0 * (n * n) as f64 * A0;
        return idx as f64 * (r_max / (cdf.len() - 1) as f64);
    }

    const N_CDF: usize = 4096;
    let r_max = 10.0 * (n * n) as f64 * A0;
    let mut cdf = vec![0.0; N_CDF];
    let dr = r_max / (N_CDF - 1) as f64;

    let mut sum = 0.0;

    for i in 0..N_CDF {
        let r = i as f64 * dr;

        // scaled radial coordinate
        let rho = 2.0 * r / (n as f64 * A0);

        // radial polynomial component
        let laguerre_l = associated_laguerre(n - l - 1, 2 * l + 1, rho);

        // normalization factors
        let norm_part1 = (2.0 / (n as f64 * A0)).powi(3);
        let norm_part2 =
            gamma::gamma((n - l) as f64) / (2.0 * n as f64 * gamma::gamma((n + l + 1) as f64));

        let norm = (norm_part1 * norm_part2).sqrt();

        let r_wave = norm * (-rho / 2.0).exp() * rho.powi(l) * laguerre_l;

        // include r^2 jacobian term
        sum += r * r * r_wave * r_wave;
        cdf[i] = sum;
    }

    // normalize cdf to [0, 1]
    for val in cdf.iter_mut() {
        *val /= sum;
    }

    let cdf_clone = cdf.clone();
    cache.push((n, l, cdf));

    let u: f64 = rand::thread_rng().gen();
    let idx = match cdf_clone.binary_search_by(|v| v.partial_cmp(&u).unwrap()) {
        Ok(i) => i,
        Err(i) => i,
    };

    idx as f64 * (r_max / (N_CDF - 1) as f64)
}

// sample theta from angular probability distribution
// includes sin(theta) from spherical volume element
fn sample_theta(l: i32, m: i32) -> f64 {
    lazy_static! {
        static ref CDF_CACHE: Mutex<Vec<(i32, i32, Vec<f64>)>> = Mutex::new(Vec::new());
    }

    let m_abs = m.abs();
    let mut cache = CDF_CACHE.lock().unwrap();

    if let Some(entry) = cache.iter().find(|(cl, cm, _)| *cl == l && *cm == m_abs) {
        let cdf = &entry.2;
        let u: f64 = rand::thread_rng().gen();

        let idx = match cdf.binary_search_by(|v| v.partial_cmp(&u).unwrap()) {
            Ok(i) => i,
            Err(i) => i,
        };

        return idx as f64 * (PI / (cdf.len() - 1) as f64);
    }

    const N_CDF: usize = 2048;
    let mut cdf = vec![0.0; N_CDF];
    let d_theta = PI / (N_CDF - 1) as f64;

    let mut sum = 0.0;

    for i in 0..N_CDF {
        let theta = i as f64 * d_theta;
        let plm = associated_legendre(l, m_abs, theta.cos());

        sum += theta.sin() * plm * plm;
        cdf[i] = sum;
    }

    for val in cdf.iter_mut() {
        *val /= sum;
    }

    let cdf_clone = cdf.clone();
    cache.push((l, m_abs, cdf));

    let u: f64 = rand::thread_rng().gen();
    let idx = match cdf_clone.binary_search_by(|v| v.partial_cmp(&u).unwrap()) {
        Ok(i) => i,
        Err(i) => i,
    };

    idx as f64 * (PI / (N_CDF - 1) as f64)
}

// associated laguerre polynomial via recurrence
// used in radial hydrogen wavefunction
fn associated_laguerre(k: i32, alpha: i32, x: f64) -> f64 {
    if k == 0 {
        return 1.0;
    }

    let mut lm1 = 1.0 + alpha as f64 - x;
    if k == 1 {
        return lm1;
    }

    let mut lm2 = 1.0;
    let mut l_val = 0.0;

    for j in 2..=k {
        l_val = ((2.0 * j as f64 - 1.0 + alpha as f64 - x) * lm1
            - (j as f64 - 1.0 + alpha as f64) * lm2)
            / j as f64;

        lm2 = lm1;
        lm1 = l_val;
    }

    l_val
}

// associated legendre polynomial via upward recurrence
// used in angular part of hydrogen wavefunction
fn associated_legendre(l: i32, m: i32, x: f64) -> f64 {
    let m_abs = m.abs();
    let mut pmm = 1.0;

    if m_abs > 0 {
        let somx2 = ((1.0 - x) * (1.0 + x)).sqrt();
        let mut fact = 1.0;

        for _ in 1..=m_abs {
            pmm *= -fact * somx2;
            fact += 2.0;
        }
    }

    if l == m_abs {
        return pmm;
    }

    let mut pm1m = x * (2 * m_abs + 1) as f64 * pmm;
    if l == m_abs + 1 {
        return pm1m;
    }

    let mut pmm_temp = pmm;

    for ll in (m_abs + 2)..=l {
        let pll = ((2 * ll - 1) as f64 * x * pm1m - (ll + m_abs - 1) as f64 * pmm_temp)
            / (ll - m_abs) as f64;

        pmm_temp = pm1m;
        pm1m = pll;
    }

    pm1m
}

// Complex spherical harmonic color function for the phi-independent angular density.
// Kept here for reference. It used P_l^m(cos theta)^2 without real
// spherical harmonic cos(m phi) / sin(|m| phi) lobes.
//
// fn get_particle_color(r: f64, theta: f64, _phi: f64, n: i32, l: i32, m: i32) -> glm::Vec4 {
//     let rho = 2.0 * r / (n as f64 * A0);
//
//     let laguerre = associated_laguerre(n - l - 1, 2 * l + 1, rho);
//
//     let norm_part1 = (2.0 / (n as f64 * A0)).powi(3);
//
//     let norm_part2 =
//         gamma::gamma((n - l) as f64) / (2.0 * n as f64 * gamma::gamma((n + l + 1) as f64));
//
//     let r_wave = (norm_part1 * norm_part2).sqrt() * (-rho / 2.0).exp() * rho.powi(l) * laguerre;
//
//     let angular = associated_legendre(l, m.abs(), theta.cos());
//
//     let raw = r_wave * r_wave * angular * angular;
//
//     let intensity = (raw * 100.0).ln_1p().min(1.0);
//     heatmap_cool(intensity)
// }

fn get_particle_color_complex(r: f64, theta: f64, n: i32, l: i32, m: i32) -> glm::Vec4 {
    let rho = 2.0 * r / (n as f64 * A0);

    let laguerre = associated_laguerre(n - l - 1, 2 * l + 1, rho);

    let norm_part1 = (2.0 / (n as f64 * A0)).powi(3);

    let norm_part2 =
        gamma::gamma((n - l) as f64) / (2.0 * n as f64 * gamma::gamma((n + l + 1) as f64));

    let r_wave = (norm_part1 * norm_part2).sqrt() * (-rho / 2.0).exp() * rho.powi(l) * laguerre;

    let angular = associated_legendre(l, m.abs(), theta.cos());

    let raw = r_wave * r_wave * angular * angular;

    let intensity = (raw * 100.0).ln_1p().min(1.0);
    heatmap_cool(intensity)
}

// compute probability density using real spherical harmonics
// and color positive/negative lobes differently
fn get_particle_color_real(r: f64, direction: &glm::DVec3, n: i32, l: i32, m: i32) -> glm::Vec4 {
    let rho = 2.0 * r / (n as f64 * A0);
    let (theta, phi) = standard_angles_from_direction(direction);

    let laguerre = associated_laguerre(n - l - 1, 2 * l + 1, rho);

    let norm_part1 = (2.0 / (n as f64 * A0)).powi(3);

    let norm_part2 =
        gamma::gamma((n - l) as f64) / (2.0 * n as f64 * gamma::gamma((n + l + 1) as f64));

    let r_wave = (norm_part1 * norm_part2).sqrt() * (-rho / 2.0).exp() * rho.powi(l) * laguerre;

    let angular = real_spherical_harmonic(l, m, theta, phi);

    let raw = r_wave * r_wave * angular * angular;

    // Real harmonics are normalized, so use a stronger visual scale.
    let intensity = (raw * 600.0).ln_1p().min(1.0);
    heatmap_real(intensity, angular >= 0.0)
}

fn real_spherical_harmonic(l: i32, m: i32, theta: f64, phi: f64) -> f64 {
    let m_abs = m.abs();
    let plm = associated_legendre(l, m_abs, theta.cos());
    let norm = (((2 * l + 1) as f64 / (4.0 * PI))
        * gamma::gamma((l - m_abs + 1) as f64)
        / gamma::gamma((l + m_abs + 1) as f64))
    .sqrt();

    let base = norm * plm;

    if m > 0 {
        (2.0_f64).sqrt() * base * (m_abs as f64 * phi).cos()
    } else if m < 0 {
        (2.0_f64).sqrt() * base * (m_abs as f64 * phi).sin()
    } else {
        base
    }
}

fn heatmap_real(value: f64, positive_lobe: bool) -> glm::Vec4 {
    let v = value.max(0.0).min(1.0) as f32;

    if positive_lobe {
        // blue heat map: dark blue -> bright cyan-blue
        glm::vec4(0.02 * v, 0.10 + 0.65 * v, 0.35 + 0.65 * v, 0.6)
    } else {
        // red heat map: dark red -> hot red-orange
        glm::vec4(0.35 + 0.65 * v, 0.04 + 0.28 * v, 0.02 * v, 0.6)
    }
}

// simple linear heatmap from black to white
// purely for visual contrast, not physical meaning
fn heatmap_cool(value: f64) -> glm::Vec4 {
    let v = value.max(0.0).min(1.0) as f32;

    // smooth gradient: dark → blue → cyan (NO WHITE)
    let r = 0.0;
    let g = v * 0.9;
    let b = 0.4 + 0.6 * v;

    glm::vec4(r, g, b, 0.6)
}
