#version 450

layout (binding = 1) uniform UniformBufferObject {
    mat4 projection;
    vec3 pos;
    vec3 color;
    float ambient;
    float specular;
} light;

layout (binding = 2) uniform sampler2D shadowMap;


layout (location = 0) in vec3 normal;
layout (location = 1) in vec3 model_position;
layout (location = 2) in vec4 color;

layout (location = 0) out vec4 outColor;


void main() {
    vec4 shadow_cordinate = light.projection * vec4(model_position, 1.0);
    float shadow = 1.0;

    if (texture( shadowMap, shadow_cordinate.xy ).z  <  shadow_cordinate.z) 
    {
        shadow = light.ambient;
    }


    //Lets find the vector ray between light source and model position
    vec3 light_ray_vector = normalize(light.pos - model_position);

    //Calculate the degree between model and light ray, with dot function
    float light_dot_function = dot(normalize(normal), light_ray_vector);

    // Lets make sure light dot is > -1 and calculate with light color
    vec3 light_color = max(light_dot_function, 0.0) * light.color ;

    vec4 result_color = vec4(light.ambient + light_color, 1.0) ;

    outColor = result_color * shadow * color;
}
