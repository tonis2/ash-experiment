#version 450

layout (location = 0) in vec3 in_position;
layout (location = 1) in vec2 in_uv;

layout (location = 0) out vec2 outUV;

out gl_PerVertex
{
	vec4 gl_Position;
};

void main() 
{
	outUV = in_uv;
	gl_Position = vec4(in_position, 1.0);
}