#version 450
#extension GL_ARB_separate_shader_objects : enable


layout (binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform Constants {
    mat4 model;
} constants;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 in_tex_cords;

layout (location = 0) out vec2 out_tex_cords;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = ubo.proj * ubo.view * constants.model * vec4(pos, 1.0);
    out_tex_cords = in_tex_cords;
}