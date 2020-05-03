#version 450
layout (constant_id = 2) const uint LIGHT_COUNT = 0U;

struct Light {
    vec3 position;
    vec3 color;
    float intensity;
	float range;
	uint  light_type;
	float inner_cone_angle;
	float outer_cone_angle;
};

layout (binding = 0) uniform sampler2D samplerColor;
layout (binding = 1) uniform sampler2D samplerposition;
layout (binding = 2) uniform sampler2D samplerNormal;
layout (std140, binding = 3) uniform LightBuffer {
    Light lights[LIGHT_COUNT > 0 ? LIGHT_COUNT : 1];
};


layout (location = 0) in vec2 inUV;
layout (location = 0) out vec4 outFragcolor;

void main() 
{
	// Get G-Buffer values
	vec3 object_pos = texture(samplerposition, inUV).rgb;
	vec3 normal = texture(samplerNormal, inUV).rgb;
	vec4 albedo = texture(samplerColor, inUV);
	
	#define ambient 0.8
	
	// Ambient part
	vec3 fragcolor = albedo.rgb * ambient;
	
	if (LIGHT_COUNT > 0) {
		for(int i = 0; i < LIGHT_COUNT; ++i)
			{
				Light light = lights[i];

				vec3 light_direction;
				float light_strength = 1.0;

				if (light.light_type == 0) {
					//directional light
					light_direction = -light.position;
					light_strength = 1.0; //no attenuation for directional lights
				} else {
					//point light
					light_direction = light.position - object_pos;
					float distanceToLight = length(light.position - object_pos);

					light_strength = 1.0 / (light.intensity * pow(distanceToLight, 2));
				}

				float diffuseCoefficient = max(dot(normalize(normal), normalize(light_direction)), 0.0);
				vec3 diffuse = diffuseCoefficient * light.color.rgb * albedo.rgb;

				fragcolor += diffuse;
		}  
	}
   
  outFragcolor = vec4(fragcolor, 1.0);	
}