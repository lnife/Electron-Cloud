# Orbital Space — Electron Cloud Simulator

A Rust-based 3D visualizer for hydrogenic atomic orbitals.

This project samples the probability density of idealized one-electron hydrogen-like orbitals and renders the resulting electron cloud in real time using `wgpu`.

## Demo

### 2p<sub>z</sub> orbital

![2pz orbital demo](assets/2pz.gif)

### 3d<sub>z²</sub> orbital

![3dz2 orbital demo](assets/3dz2.gif)

## Overview

Orbital Space generates Monte Carlo samples of hydrogenic orbitals defined by the quantum numbers `(n, l, m)`:

- `n` — principal quantum number
- `l` — azimuthal quantum number
- `m` — magnetic quantum number

Rather than drawing analytical orbital surfaces, the program samples the probability distribution directly. Each sample becomes a small rendered particle in 3D space.

The visualization represents orbital probability density. It does not show a classical electron trajectory.

## Motivation

This project was built as both a quantum-mechanics study and a systems-programming exercise.

The goal was to understand how analytical hydrogenic wavefunctions can be turned into a numerical visualization pipeline:

- constructing hydrogenic radial functions
- evaluating associated Laguerre and Legendre polynomials
- sampling probability distributions using cumulative distribution functions
- transforming spherical samples into Cartesian coordinates
- connecting CPU-side physics calculations with a GPU rendering pipeline
- rendering many sampled particles efficiently using modern graphics tools

The project is intended for learning, visualization, and scientific-programming practice rather than production quantum chemistry.

## Physics Background

For a hydrogen-like one-electron atom, the orbital wavefunction can be separated into radial and angular parts:

```text
ψ(r, θ, φ) = R_nl(r) Y_lm(θ, φ)
```

The probability density is proportional to:

```text
|ψ(r, θ, φ)|²
```

In spherical coordinates, the volume element is:

```text
dV = r² sin(θ) dr dθ dφ
```

The implementation samples:

- the radial distribution using the `r²` Jacobian factor
- the polar angular distribution using the `sin(θ)` factor
- the azimuthal angle `φ` uniformly from `0` to `2π`

The Bohr radius is set to `1.0`, so the visualization uses atomic-unit-style scaling.

## How It Works

1. The program asks the user for quantum numbers and particle count.
2. It validates that the quantum numbers are physically allowed.
3. It builds cumulative distribution functions for the radial and angular probability distributions.
4. It samples `r`, `θ`, and `φ`.
5. It converts the spherical coordinates to Cartesian coordinates.
6. It maps probability-density intensity to particle color.
7. It renders the sampled cloud in a 3D window.

## Quantum Number Validation

The input quantum numbers must satisfy:

```text
n > 0
0 ≤ l < n
-l ≤ m ≤ l
```

Invalid inputs are rejected before rendering starts.

## Architecture

### Physics Layer

The physics layer handles the orbital sampling logic:

- associated Laguerre polynomial recurrence
- associated Legendre polynomial recurrence
- radial cumulative distribution construction
- angular cumulative distribution construction
- inverse transform sampling
- Monte Carlo particle generation
- probability-density-based color mapping

Most physics calculations are performed in `f64` before being converted for rendering.

### Geometry Layer

The geometry layer generates the base sphere mesh used for rendered particles:

- procedural sphere vertex generation
- indexed triangle construction
- expansion into a flat vertex list for the rendering pipeline

Each sampled point is rendered as a small sphere-like particle.

### Camera System

The camera system supports orbit-style viewing:

- mouse-based camera rotation
- scroll-based zooming
- spherical-coordinate-style camera movement around the cloud

### Rendering Pipeline

The renderer uses:

- `wgpu` for GPU rendering
- `winit` for window and event handling
- depth buffering for occlusion
- instanced rendering for efficient particle visualization
- WGSL shaders for the GPU pipeline

The physics calculations are performed on the CPU. The GPU is used for visualization.

## Numerical Strategy

### Radial Sampling

The radial coordinate is sampled by constructing a discretized cumulative distribution function for each `(n, l)` pair.

The sampled radial probability includes the spherical Jacobian term:

```text
r² |R_nl(r)|²
```

The CDF is cached so that the same `(n, l)` distribution does not need to be rebuilt repeatedly.

### Angular Sampling

The polar angle `θ` is sampled from a discretized angular CDF based on:

```text
sin(θ) |P_l^m(cosθ)|²
```

The angular CDF is cached for each `(l, |m|)` pair.

### Azimuthal Sampling

The azimuthal angle `φ` is sampled uniformly from:

```text
0 ≤ φ < 2π
```

## Controls

```text
Mouse drag    Rotate camera
Scroll wheel  Zoom in/out
Esc           Exit
```

## Running the Project

Make sure Rust is installed, then clone the repository and run:

```bash
cargo run --release
```

Release mode is recommended for smoother rendering.

When the program starts, it prompts for:

- principal quantum number `n`
- azimuthal quantum number `l`
- magnetic quantum number `m`
- particle count

## Example Inputs

For a `2p_z`-like orbital:

```text
n = 2
l = 1
m = 0
```

For a `3d_z²`-like orbital:

```text
n = 3
l = 2
m = 0
```

## Project Structure

```text
src/
├── main.rs        # Program entry point, input handling, and render loop
├── physics.rs     # Orbital sampling, radial/angular functions, color mapping
├── geometry.rs    # Sphere mesh generation for rendered particles
├── camera.rs      # Camera movement, rotation, and zoom controls
├── texture.rs     # Depth texture setup for rendering
└── shaders.wgsl   # GPU shader code

assets/
├── 2pz.gif
└── 3dz2.gif
```

## Dependencies

This project uses Rust crates for rendering, math, random sampling, and special functions, including:

- `wgpu`
- `winit`
- `nalgebra-glm`
- `rand`
- `statrs`
- `lazy_static`
- `pollster`

See `Cargo.toml` for the exact dependency list.

## Limitations

This project currently focuses on visualization of analytical hydrogenic orbitals.

Current limitations include:

- hydrogenic one-electron orbitals only
- no multi-electron atoms
- no spin treatment
- no electron-electron correlation
- no relativistic corrections
- no molecular orbitals
- no time-dependent dynamics
- CPU-side sampling
- simple color mapping for visual contrast rather than a calibrated physical observable

## Design Philosophy

Physics and rendering are intentionally separated.

The project focuses on building the sampling and rendering pipeline directly instead of relying on black-box orbital visualization software. The main emphasis is understanding how quantum-mechanical probability distributions become numerical samples and how those samples can be rendered efficiently.

## Future Improvements

Possible extensions include:

- real combinations of spherical harmonics
- improved angular normalization
- nodal surface highlighting
- radial distribution plotting
- adjustable particle size and color scaling
- UI controls for changing orbitals without restarting
- frame or animation export
- GPU compute-based sampling
- simple extensions toward multi-electron visualization models

## Author

Bhaskar Malviya  
Computational Chemistry | Quantum Chemistry | Scientific Programming
