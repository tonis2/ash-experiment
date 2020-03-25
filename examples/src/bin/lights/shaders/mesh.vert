#version 450
#extension GL_ARB_separate_shader_objects : enable
layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 normal;


layout (location = 0) out vec3 out_normal;
layout (location = 1) out vec3 out_position;
layout (location = 2) out vec4 out_color;

layout (binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform Constants {
    mat4 model_transform;
    vec4 color;
} constants;


void main() {
    vec4 mesh_world_position = constants.model_transform * vec4(pos, 1.0);

    out_color = constants.color;

    out_position = mesh_world_position.xyz;
    out_normal = (constants.model_transform * vec4(normal, 0.0)).xyz;

    gl_Position = ubo.proj * ubo.view * mesh_world_position;
}