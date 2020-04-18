
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
} camera;

layout(push_constant) uniform Constant {
    mat4 model_transform;
} constant;

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec4 inColor;
layout (location = 2) in vec3 normal;
layout (location = 3) in vec2 uv;
layout (location = 4) in int material_index;

layout (location = 0) out vec4 fragColor;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec2 out_uv;
layout (location = 3) out flat int out_material_index;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = camera.proj * camera.view * constant.model_transform * vec4(inPosition, 1.0);
    fragColor = inColor;
    out_normal = normal;
    out_uv = uv;
    out_material_index = material_index;
}
