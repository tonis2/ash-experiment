#version 450

layout (location = 0) in vec3 pos;

layout (binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform Constants {
    mat4 model_transform;
    vec4 color;
} constants;
 
void main()
{
	   gl_Position = ubo.proj * ubo.view * constants.model_transform * vec4(pos, 1.0);
}