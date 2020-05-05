#version 450
#extension GL_ARB_separate_shader_objects : enable
const int TILE_SIZE = 16;


layout (constant_id = 0) const uint LIGHT_COUNT = 0U;
layout (constant_id = 1) const uint MATERIALS_AMOUNT = 0U;
layout (constant_id = 3) const uint TEXTURE_AMOUNT = 0U;


struct Light {
    vec3 position;
    vec3 color;
    float intensity;
	float range;
	uint  light_type;
	float inner_cone_angle;
	float outer_cone_angle;
};

struct TextureInfo {
    int index;
    uint channel;
};

struct LightVisiblity
{
	uint count;
	uint light_indices[1023];
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

layout (std140, binding = 0) uniform LightBuffer {
    Light lights[LIGHT_COUNT > 0 ? LIGHT_COUNT : 1];
};

layout (binding = 1) uniform MaterialData {
  Material materials[MATERIALS_AMOUNT > 0 ? MATERIALS_AMOUNT : 1];
};

layout (binding = 2) uniform sampler2D textureSampler[TEXTURE_AMOUNT > 0 ? TEXTURE_AMOUNT : 1];

layout(std430, binding = 3) buffer readonly TileLightVisiblities
{
    LightVisiblity light_visiblities[];
};

layout(push_constant) uniform Constant {
    mat4 model_transform;
    uint screen_width;
    uint screen_height;
    uint row_count;
    uint column_count;
} constant;

layout (location = 0) in vec4 fragColor;
layout (location = 1) in vec3 in_tangent;
layout (location = 2) in vec3 in_normal;
layout (location = 3) in vec2 uv;
layout (location = 4) in vec4 model_postion;
layout (location = 5) in flat int material_index;

layout(location = 0) out vec4 out_color;

void main() {
        ivec2 tile_id = ivec2(gl_FragCoord.xy / TILE_SIZE);
        uint tile_index = tile_id.y * constant.column_count + tile_id.x;

        out_color = fragColor;
}