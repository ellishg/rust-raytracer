# rust-raytracer
[![Build Status](https://travis-ci.com/ellishg/rust-raytracer.svg?token=g46Mfub8GMWqdPYXVqEs&branch=master)](https://travis-ci.com/ellishg/rust-raytracer)

![scene](media/scene.png)
This 1024x1024 image with about 2.3 million triangles was rendered with 16 samples per pixel in about 39 seconds (generating the BVH tree took about 27 seconds) on a 4 core i5-6600K CPU. Thanks to the Stanford Computer Graphics Laboratory for providing these models  at [https://graphics.stanford.edu/data/3Dscanrep/](https://graphics.stanford.edu/data/3Dscanrep/).

## TODO
- [x] Move structs into their own files
- Lights
  - [x] Ambient
  - [x] Point
  - [x] Cone
  - [x] Directional
- Shadows
  - [ ] Soft shadows
  - [ ] Correct shadows from transparent surfaces
- Objects
  - [x] Sphere
  - [x] Plane
  - [x] Triangle
  - [x] Triangle Meshes
    - [x] BVH
      - [ ] Parallelize
- Materials
  - [x] Flat
  - [x] Phong
  - [x] Reflective
  - [x] Transparent
  - [x] Texture
    - [ ] Super sample texture
- Tracing
  - [x] Anti-aliasing
  - [ ] Focus blur
- [x] Threads
