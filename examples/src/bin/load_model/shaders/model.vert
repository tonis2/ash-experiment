
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (binding = 0) uniform Camera {
    vec4 position;
    mat4 view;
    mat4 proj;
} camera;


layout(push_constant) uniform Constants {
    mat4 transform;
} constants;

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inColor;
layout (location = 2) in vec2 inTexCoord;

layout (location = 0) out vec3 fragColor;
layout (location = 1) out vec2 fragTexCoord;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = camera.proj * camera.view * constants.transform * vec4(inPosition, 1.0);
    fragColor = inColor;
    fragTexCoord = inTexCoord;
}