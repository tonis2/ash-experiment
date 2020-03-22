#version 450
#extension GL_ARB_separate_shader_objects : enable
layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 text_cord;
layout (location = 2) in vec3 color;
layout (location = 3) in vec3 normal;

layout (location = 0) out vec2 out_tex_cords;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 out_position;
layout (location = 3) out vec3 out_color;

layout (binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform Constants {
    mat4 model;
} constants;


void main() {
    out_normal = mat3(constants.model) * vec3(normal.xyz);
    out_position = ubo.proj * ubo.view * constants.model * vec4(pos, 1.0);
    out_color = color;
    out_tex_cords = text_cord;

    gl_Position = out_position;
}