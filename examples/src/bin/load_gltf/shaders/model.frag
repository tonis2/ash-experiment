#version 450
#extension GL_ARB_separate_shader_objects : enable

precision highp float;


struct TextureInfo {
    int index;
    int channel;
};

struct Material {
    vec4 base_color;
    int metallic_factor;
    int roughness_factor;
    vec4 emissive_color;
    vec4 color;
    vec4 emissive;
    int occlusion;
    TextureInfo color_texture;
    TextureInfo emissive_texture;
    TextureInfo normals_texture;
    TextureInfo occlusion_texture;
};

layout (constant_id = 1) const uint MATERIALS_AMOUNT = 0U;
layout (constant_id = 0) const uint TEXTURE_AMOUNT = 0U;


#ifdef TEXTURE_AMOUNT
layout (binding = 2) uniform sampler2D textureSampler[TEXTURE_AMOUNT];
#endif

#if MATERIALS_AMOUNT > 0
layout (binding = 1) uniform MaterialData {
   Material material[MATERIALS_AMOUNT];
};
#endif


layout (location = 0) in vec4 fragColor;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;
layout (location = 0) out vec4 outColor;

void main() {
    #ifdef TEXTURE_AMOUNT
         outColor = texture(textureSampler[0], uv);
    #else
        outColor = fragColor;
    #endif  
}
