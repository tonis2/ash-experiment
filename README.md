# ash-experiment


Practice rendering engine, built on top of vulkan and [ash](https://github.com/MaikKlein/ash)

Many thanks to these [vulkan examples](https://github.com/unknownue/vulkan-tutorial-rust)


To run examples download assets running `python3 ./download_assets.py`  
and then you can choose examle with 
`cargo run --example` + example name 


For building shaders i used `glslangValidator -V *shader glsl* -o shader.spv`

Tested on `windows 10` and `linux 18.04`
