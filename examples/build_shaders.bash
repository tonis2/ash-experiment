#!/bin/bash

# GLTF_loader
glslangValidator -V ./src/bin/load_gltf/shaders/model.vert -o ./src/bin/load_gltf/shaders/model.vert.spv
glslangValidator -V ./src/bin/load_gltf/shaders/model.frag -o ./src/bin/load_gltf/shaders/model.frag.spv


# Lights
glslangValidator -V ./src/bin/lights/shaders/mesh.vert -o ./src/bin/lights/shaders/mesh.vert.spv
glslangValidator -V ./src/bin/lights/shaders/mesh.frag -o ./src/bin/lights/shaders/mesh.frag.spv

glslangValidator -V ./src/bin/lights/shaders/offscreen.vert -o ./src/bin/lights/shaders/offscreen.vert.spv



# Defferred rendering 
glslangValidator -V ./src/bin/deferred_rendering/shaders/gbuffer.vert -o ./src/bin/deferred_rendering/shaders/gbuffer.vert.spv
glslangValidator -V ./src/bin/deferred_rendering/shaders/gbuffer.frag -o ./src/bin/deferred_rendering/shaders/gbuffer.frag.spv

glslangValidator -V ./src/bin/deferred_rendering/shaders/deferred.vert -o ./src/bin/deferred_rendering/shaders/deferred.vert.spv
glslangValidator -V ./src/bin/deferred_rendering/shaders/deferred.frag -o ./src/bin/deferred_rendering/shaders/deferred.frag.spv


# Forward plus
glslangValidator -V ./src/bin/forward_plus/shaders/depth.vert -o ./src/bin/forward_plus/shaders/depth.vert.spv
glslangValidator -V ./src/bin/forward_plus/shaders/light_culling.comp -o ./src/bin/forward_plus/shaders/light_culling.compute.spv
