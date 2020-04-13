#!/bin/bash

# GLTF_loader
glslangValidator -V ./src/bin/load_gltf/shaders/model.vert -o ./src/bin/load_gltf/shaders/model.vert.spv
glslangValidator -V ./src/bin/load_gltf/shaders/model.frag -o ./src/bin/load_gltf/shaders/model.frag.spv