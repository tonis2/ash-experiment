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
layout (location = 3) in vec4 color;

layout (location = 0) out vec4 outColor;


void main() {
    vec3 light_dir = normalize(light.pos.xyz - position.xyz);

    float diffuse = max(0.0, dot(normalize(normal), light_dir));
    vec4 light_color = color * vec4(0.05, 0.05, 0.05, 1.0) * light.projection * vec4(position, 1.0);

    outColor = texture(texSampler, text_cord) * light_color;
}
