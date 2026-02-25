# Electron-Cloud

Electron-Cloud is a personal learning project written in Rust. It is inspired by the behavior and concepts of an existing C++ project and was created as an exercise to learn Rust, systems programming concepts, and safe memory management.  

The original project that inspired this work is:  

https://github.com/kavan010/Atoms    
Original author: kavan010    

All credit for the original idea and design belongs to the original author.  

---  

## Overview

This repository contains a Rust implementation that follows similar goals and behavior as the original C++ project. The purpose of this project is educational: to practice translating program behavior into Rust, understand ownership and borrowing, and explore Rust tooling.  

This project is not related to the Electron desktop framework. The name originates from the original project.  

---  

## Project Structure
```
Electron-Cloud  
├── src/ # Rust source files  
├── Cargo.toml # Rust manifest and dependencies  
├── Cargo.lock # Locked dependency versions  
└── GEMINI.md # Project notes and auxiliary documentation
```
---  

## Requirements

You need the Rust toolchain installed:  

```
rustup install stable  
rustup default stable
```
Verify installation:
```
rustc --version  
cargo --version
```
---

## Build

To compile the project
```
cargo build --release
```
---

## Run

To run the project:
```
cargo run
```
---

## Learning Goals

This repository exists primarily as a learning exercise:

- Practice writing idiomatic Rust

- Understand ownership, borrowing, and lifetimes

- Translate program behavior across languages

- Explore Rust project structure and tooling using Cargo

---

## Attribution and Status

This project is inspired by the behavior and ideas of the original C++ repository:

[https://github.com/kavan010/Atoms](https://github.com/kavan010/Atoms?utm_source=chatgpt.com)

The original repository does not specify a license. In the absence of an explicit license, no reuse rights are assumed for the original work. This repository is therefore shared as a personal learning and portfolio artifact only.

This repository is not offered as open source software, and no license is granted for reuse or redistribution at this time. The code is published for educational reference and personal documentation purposes.

If the original author later provides explicit permission or adds a license to the original project, the status of this repository may be updated accordingly.

---

## AI Assistance

This project, including this README file and parts of the source code, was developed with the assistance of an AI programming partner.

---

## Disclaimer

This repository is not an official continuation of the original project. It is an independent educational implementation created for learning Rust.
