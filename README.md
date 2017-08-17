gammaray
========

A renderer written in Rust. Buzz-words include:
* Physically-based!
* Global illumination!
* Monte Carlo ray tracing!
* Artist-directable shading via the Disney BRDF!

Building
--------
Install the latest version of Rust (I'm using 1.19),
then simply do `cargo run` and you're set!
It currently writes to `output.exr`; you will need an EXR viewer.

Credits
-------
References are cited at specific locations in the code.
Licenses for third-party code are listed in `LICENSE.md`.
* Physically Based Rendering, 2nd and 3rd editions, by Matt Pharr and
  Greg Humphreys (Morgan Kaufmann).
  [Book website](http://pbrt.org/).
  [Source code](https://github.com/mmp/pbrt-v3/).
* Universal Scene Description.
  [Source code](https://github.com/PixarAnimationStudios/USD/).
* Brent Burley's SIGGRAPH course notes on the Disney BRDF/BSSRDF:
    - http://blog.selfshadow.com/publications/s2012-shading-course/
    - http://blog.selfshadow.com/publications/s2015-shading-course/