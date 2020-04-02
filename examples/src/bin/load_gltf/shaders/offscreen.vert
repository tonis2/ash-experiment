#version 450

layout (location = 0) in vec3 pos;

layout (binding = 0) uniform Light {
    vec4 position;
    mat4 projection;
    vec4 color;
    vec4 ambient;
} light;

layout(push_constant) uniform Constants {
    mat4 model_transform;
    vec4 color;
} constants;
 
void main()
{
	   gl_Position = light.projection * constants.model_transform * vec4(pos, 1.0);
}