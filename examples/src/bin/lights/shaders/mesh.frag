#version 450

layout (binding = 1) uniform UniformBufferObject {
    mat4 projection;
    vec3 pos;
    vec3 color;
} light;

layout (binding = 2) uniform sampler2D texSampler;

layout (location = 0) in vec2 text_cord;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 position;
layout (location = 3) in vec3 color;

layout (location = 0) out vec4 outColor;


void main() {

    outColor = vec4(color, 1.0) * texture(texSampler, text_cord);
}
