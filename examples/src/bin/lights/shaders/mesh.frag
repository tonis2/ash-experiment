#version 450
#extension GL_ARB_separate_shader_objects : enable

struct Light {
    vec4 position;
    mat4 projection;
    vec4 color;
    vec4 ambient;
};

layout (binding = 1) uniform LightBuffer {
    Light light_data;
};

layout (binding = 2) uniform sampler2D shadowMap;

layout (location = 0) in vec3 model_normal;
layout (location = 1) in vec4 shadow_cordinate;
layout (location = 2) in vec4 color;
layout (location = 0) out vec4 outColor;

vec3 CalculateLightColor(Light light, vec4 object_color, vec3 normal, vec4 object_pos) {
    vec3 light_direction;
    float light_strength = 1.0;

    if (light.position.w == 0.0) {
        //directional light
        light_direction = light.position.xyz;
        light_strength = 1.0; //no attenuation for directional lights
    } else {
        //point light
        light_direction = light.position.xyz - object_pos.xyz;
        float distanceToLight = length(light.position.xyz - object_pos.xyz);

        //Todo add light_strength parameter to buffer
        light_strength = 1.0 / (1.0 + 1.0 * pow(distanceToLight, 2));
    }

     //ambient
    vec3 ambient = object_color.rgb * light.color.rgb;

    //diffuse
    float diffuseCoefficient = max(dot(normalize(normal), normalize(light_direction)), 0.0);
    vec3 diffuse = light.color.rgb * diffuseCoefficient * object_color.rgb;

    //linear color (color before gamma correction)
    return ambient.rgb + diffuse;
}

void main() {
    float shadow = 1.0;


    if (texture(shadowMap, shadow_cordinate.xy).z  <  shadow_cordinate.z) 
    {
        shadow = 0.5;
    }
  
    vec3 light_color = CalculateLightColor(light_data, color, model_normal, shadow_cordinate);

    outColor =  vec4(light_color, 1.0);
}
