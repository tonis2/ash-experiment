#!/bin/bash

# GLTF_loader
glslangValidator -V ./src/bin/load_gltf/shaders/model.vert -o ./src/bin/load_gltf/shaders/model.vert.spv
glslangValidator -V ./src/bin/load_gltf/shaders/model.frag -o ./src/bin/load_gltf/shaders/model.frag.spv


# Load json model
glslangValidator -V ./src/bin/load_model/shaders/model.vert -o ./src/bin/load_model/shaders/model.vert.spv
glslangValidator -V ./src/bin/load_model/shaders/model.frag -o ./src/bin/load_model/shaders/model.frag.spv


# Defferred rendering 
glslangValidator -V ./src/bin/deferred_rendering/shaders/gbuffer.vert -o ./src/bin/deferred_rendering/shaders/gbuffer.vert.spv
glslangValidator -V ./src/bin/deferred_rendering/shaders/gbuffer.frag -o ./src/bin/deferred_rendering/shaders/gbuffer.frag.spv

glslangValidator -V ./src/bin/deferred_rendering/shaders/deferred.vert -o ./src/bin/deferred_rendering/shaders/deferred.vert.spv
glslangValidator -V ./src/bin/deferred_rendering/shaders/deferred.frag -o ./src/bin/deferred_rendering/shaders/deferred.frag.spv