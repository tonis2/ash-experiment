#version 450
#extension GL_ARB_separate_shader_objects : enable

struct TextureInfo {
    int index;
    uint channel;
};

struct Material {
    vec4 base_color;
    vec4 color;
    vec4 emissive_color;
    vec4 emissive;
    float metallic_factor;
    float roughness_factor;
    float occlusion;
    TextureInfo color_texture;
    TextureInfo emissive_texture;
    TextureInfo normals_texture;
    TextureInfo occlusion_texture;
};

layout (constant_id = 1) const uint MATERIALS_AMOUNT = 0U;
layout (constant_id = 0) const uint TEXTURE_AMOUNT = 0U;

layout (binding = 1) uniform MaterialData {
  Material materials[MATERIALS_AMOUNT > 0 ? MATERIALS_AMOUNT : 1];
};

//Cant use sampler array with 0 entries, so i used this hack, not sure if it's good idea
layout (binding = 2) uniform sampler2D textureSampler[TEXTURE_AMOUNT > 0 ? TEXTURE_AMOUNT : 1];

layout (location = 0) in vec4 fragColor;
layout (location = 1) in vec4 tangents;
layout (location = 2) in vec3 normal;
layout (location = 3) in vec2 uv;
layout (location = 4) in flat int material_index;

layout (location = 0) out vec4 outColor;


void main() {
    if (MATERIALS_AMOUNT > 0) {
        Material mesh_material = materials[material_index];
        
        //Apply Material data to mesh
        if (mesh_material.color_texture.index != -1) {
             outColor = texture(textureSampler[mesh_material.color_texture.index], uv);
        }

        if (mesh_material.normals_texture.index != -1) {
             outColor = texture(textureSampler[mesh_material.color_texture.index], uv);
        }
    } else {
        outColor = fragColor;
    } 
}
