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
layout (location = 1) in vec3 in_tangent;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec2 uv;
layout (location = 4) in vec3 model_postion;
layout (location = 5) in flat int material_index;

layout (location = 0) out vec4 outColor;
layout (location = 1) out vec4 outNormal;
layout (location = 2) out vec4 outPosition;

void main() {
    if (MATERIALS_AMOUNT > 0) {
        Material mesh_material = materials[material_index];
        
        //Apply Material data to mesh
        if (mesh_material.color_texture.index != -1) {
             outColor = texture(textureSampler[mesh_material.color_texture.index], uv);
        }

        //Normal texture
        if (mesh_material.normals_texture.index != -1) {
                vec3 normal_texture = texture(textureSampler[mesh_material.color_texture.index], uv).xyz;

             // Calculate normal in tangent space
                vec3 Normal = normalize(in_normal);
                Normal.y = -Normal.y;
                vec3 Tangent = normalize(in_tangent);
                vec3 Bittanget = cross(Normal, Tangent);
                mat3 TBN = mat3(Tangent, Bittanget, Normal);
                vec3 tnorm = TBN * normalize(normal_texture * 2.0 - vec3(1.0));
                outNormal = vec4(tnorm, 1.0);
        } else {
            outNormal = vec4(0.0, 0.0, 0.0, 0.0);
        }

    } else {
        outColor = fragColor;
        outNormal = vec4(0.0, 0.0, 0.0, 0.0);
    }

    outPosition = vec4(model_postion, 1.0); 
}
