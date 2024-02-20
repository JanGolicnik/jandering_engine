# Jandering engine

This is a simple rendering engine I made as both a learning project and hopefully something I can use for all my further graphics programming endeavors. I want the engine to be super bare bones, as to not limit me when I want to do something obscure, but still simple enough where it doesnt take more than a few lines of code to get a pleasing result.

I want to include things that may not necerssarily fall under a rendering engine such as object loading, ui, and various other utilities. They may be implemented as separate crates tho.

## Things to check out

- [instancing example](https://github.com/JanGolicnik/jandering_engine/tree/maste/examples/instancing) -> example using lots of instances, feel free to increase the number to the millions :P
- [ray marching example](https://github.com/JanGolicnik/jandering_engine/tree/maste/examples/ray_marching) -> example ray marching following [this](https://youtu.be/khblXafu7iA?si=WbOveB6sX3Wdz3dF) tutorial
- [wasm template](https://github.com/JanGolicnik/jandering_engine/tree/wasm) -> template for creating wasm projects
- [wgsltoy](https://janyg.itch.io/wgsltoy) -> real time wgsl shader editor like Shader Toy
- [dashy geometry](https://janyg.itch.io/wgsltoy)(geometry dash clone) -> geometry dash with map editing and bloom

## Checklist

- [x] rendering a simple triangle
- [x] instancing
- [x] custom shaders
- [x] custom cameras
- [x] proper bind groups
- [ ] model loading
- [ ] UI
- [ ] PBR (deffered with forward transparency?)
- [ ] skybox
- [x] wasm support (check out wasm branch)
- [x] textures
