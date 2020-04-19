# ash-experiment


Practice rendering engine, built on top of vulkan and [ash](https://github.com/MaikKlein/ash)

Many thanks to these [vulkan examples](https://github.com/unknownue/vulkan-tutorial-rust)



##### Usage
----

Download assets `python3 ./download_assets.py` 


##### Running examples

Open examples folders and run with --bin.

```
cargo run --bin lights
```

Some examples can be rotated with mouse.


For shader building i used [glslang](https://github.com/KhronosGroup/glslang)
example `glslangValidator -V *shader glsl* -o shader.spv`



Platforms

 > windows 10
 
 > linux 18.04

