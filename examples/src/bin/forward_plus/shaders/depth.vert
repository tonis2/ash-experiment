#version 450

layout (location = 0) in vec3 pos;

layout (binding = 0) uniform Camera {
    vec4 position;
    mat4 view;
    mat4 proj;
} camera;


layout(push_constant) uniform Constant {
    mat4 model_transform;
} constant;
 
void main()
{
	   gl_Position = camera.projection * constants.model_transform * vec4(pos, 1.0);
}