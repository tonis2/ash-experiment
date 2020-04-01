#version 450
#extension GL_ARB_separate_shader_objects : enable
layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 normal;

layout (location = 0) out vec3 out_normal;
layout (location = 1) out vec4 out_position;
layout (location = 2) out vec4 shadow_cordinate;
layout (location = 3) out vec4 out_color;


layout (binding = 0) uniform Camera {
    vec4 pos;
    mat4 view;
    mat4 proj;
} camera;

layout (binding = 1) uniform Light {
    vec4 position;
    mat4 projection;
    vec4 color;
    vec4 ambient;
} light;

layout(push_constant) uniform Constants {
    mat4 model_transform;
    vec4 color;
} constants;

const mat4 biasMat = mat4( 
	0.5, 0.0, 0.0, 0.0,
	0.0, 0.5, 0.0, 0.0,
	0.0, 0.0, 1.0, 0.0,
	0.5, 0.5, 0.0, 1.0 );

void main() {

    out_color = constants.color;
    out_position = constants.model_transform * vec4(pos, 1.0);
    out_normal = mat3(constants.model_transform) * normal;
    shadow_cordinate = biasMat * light.projection * constants.model_transform * vec4(pos, 1.0);
    gl_Position = camera.proj * camera.view * constants.model_transform * vec4(pos, 1.0);
}