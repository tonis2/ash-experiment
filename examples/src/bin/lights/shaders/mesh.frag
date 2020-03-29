#version 450
#extension GL_ARB_separate_shader_objects : enable

struct Light {
    vec4 position;
    vec3 color;
    float ambient;
    float specular;
};

layout (binding = 1) uniform LightBuffer {
    Light light_data;
};

layout (binding = 2) uniform sampler2D shadowMap;

layout (location = 0) in vec3 model_normal;
layout (location = 1) in vec3 model_position;
layout (location = 2) in vec4 color;
layout (location = 0) out vec4 outColor;

vec3 CalculateLightColor(Light light, vec3 object_color, vec3 normal, vec3 object_pos) {
    vec3 light_direction;
    float light_strength = 1.0;

    if (light.position.w == 0.0) {
        //directional light
        light_direction = normalize(light.position.xyz);
        light_strength = 1.0; //no attenuation for directional lights
    } else {
        //point light
        light_direction = normalize(light.position.xyz - object_pos);
        float distanceToLight = length(light.position.xyz - object_pos);

        //Todo add light_strength parameter to buffer
        light_strength = 1.0 / (1.0 + 1.0 * pow(distanceToLight, 2));
    }

     //ambient
    vec3 ambient = light.ambient * object_color.rgb * light.color;

    //diffuse
    float diffuseCoefficient = max(0.0, dot(normal, light_direction));
    vec3 diffuse = diffuseCoefficient * object_color.rgb * light.color;

    //linear color (color before gamma correction)
    return ambient + light_strength * diffuse;
}

void main() {
    // vec4 shadow_cordinate = light.projection * vec4(model_position, 1.0);
    // float shadow = 1.0;

    // if (texture( shadowMap, shadow_cordinate.xy ).z  <  shadow_cordinate.z) 
    // {
    //     shadow = light.ambient;
    // }
  
    vec3 light_color = CalculateLightColor(light_data, color.rgb, model_normal, model_position);

    outColor = color;
}
