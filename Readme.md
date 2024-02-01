# WGSLToy

Real time shader editor like Shader Toy but with wgsl.

Also on [itch](https://janyg.itch.io/wgsltoy)!

## How to run

Make sure you have jandering engine where Cargo.toml can pick it up!

    wasm-pack build --target web

## Tips and tricks

You get uTime and uResolution in these two structs, use them wisely !

    struct Time {
        delta: f32,
        elapsed: f32,
    };

    struct Resolution {
        res: vec2<f32>,
    };
