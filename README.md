# Electron Cloud

A modular electronic structure visualization toolkit written in Rust.

Electron Cloud explores the computational foundations of hydrogenic wavefunctions by directly sampling and rendering their probability densities in three dimensions.  

It is both a quantum mechanics study and a systems-level programming experiment.

---

## Overview

Electron Cloud generates Monte Carlo samples of hydrogenic orbitals defined by quantum numbers (n, l, m) and renders them in real time using GPU instanced rendering.

Rather than plotting analytical surfaces, the wavefunction is sampled probabilistically:

- Radial distribution sampled from |Rₙₗ(r)|² r²  
- Angular distribution sampled from |Pₗᵐ(cosθ)|² sinθ  
- Azimuthal angle φ sampled uniformly  

Each sample becomes a particle rendered as a small sphere.  
Color encodes probability density intensity.

The result is a spatial electron cloud consistent with the underlying quantum mechanical distribution.

---

## Motivation

This project was developed alongside formal training in:

- Electronic Structure Theory  
- Density Functional Theory  
- Vibronic Coupling Models  
- Quantum Chemical Wavefunctions  

Instead of relying on black-box visualization software, this repository constructs both the numerical sampling and the rendering pipeline from first principles.

The objective is not photorealism.

The objective is understanding:

- Construction of hydrogenic radial functions  
- Numerical behavior of associated Laguerre and Legendre polynomials  
- Probability density → spatial sampling transformations  
- CPU-side physics integration with GPU rendering pipelines  
- Instanced rendering architecture in modern graphics APIs  

This project serves as both a physics laboratory and a systems programming exercise.

---

## Architecture

### Physics Layer (`physics.rs`)

- Associated Laguerre polynomial implementation  
- Associated Legendre polynomial implementation  
- Radial and angular cumulative distribution construction  
- Inverse transform sampling  
- Monte Carlo particle generation  

All physics calculations are performed in `f64` for numerical stability.

---

### Geometry Layer (`geometry.rs`)

- Procedural sphere mesh generation  
- Indexed triangle construction  
- Base mesh reused for instanced rendering  

---

### Camera System (`camera.rs`)

- Orbit-style spherical coordinate camera  
- Spherical → Cartesian coordinate conversion  
- View matrix construction using `look_at`  

The camera logic mirrors the spherical coordinate framework used in orbital sampling.

---

### Rendering Pipeline

- `wgpu` backend  
- GPU instanced rendering for particle efficiency  
- Depth buffering for proper occlusion  
- Minimal shader pipeline (no lighting model)  

All physics remains CPU-side.  
The GPU is used exclusively for visualization.

---

## Numerical Strategy

### Radial Sampling

- Discretized CDF construction  
- Cached per (n, l) pair  
- Binary search inversion for sampling  

### Angular Sampling

- Discretized CDF construction  
- Cached per (l, |m|) pair  

Caching prevents recomputation of expensive polynomial evaluations during large particle simulations.

---

## Controls

```
Mouse Drag  → Orbit camera  
Scroll      → Zoom  
Esc         → Exit  
```

---

## Running the Project

```bash
cargo run --release
```

You will be prompted for:

- Principal quantum number (n)
- Azimuthal quantum number (l)
- Magnetic quantum number (m)
- Particle count

---

## Limitations

- Hydrogenic orbitals only  
- No relativistic corrections  
- No spin or many-electron effects  
- No lighting or physically-based shading  
- CPU-bound sampling  

This is not a production quantum chemistry package.  
It is a computational learning and visualization tool.

---

## Design Philosophy

Physics and rendering are intentionally decoupled.

The mathematical structure (wavefunctions, sampling, normalization) was implemented deliberately to reinforce conceptual understanding of quantum mechanical foundations.

Portions of the GPU pipeline were developed with assistance from AI tools.  
The numerical methods, orbital sampling logic, and mathematical formulation were implemented directly as part of ongoing theoretical chemistry training.

This repository represents a self-taught systems programming effort layered on top of a computational chemistry background.

---

## Future Directions

- Gradient-based shading
- Nodal surface highlighting
- Radial distribution plotting
- GPU compute-based sampling
- Extension toward simple multi-electron approximations
- Modular electronic structure components

---

#### Author

Lnifelias Stargarden  
Real name: Bhaskar Malviya  

Computational Chemistry | Quantum Chemistry | Scientific Programming

---
