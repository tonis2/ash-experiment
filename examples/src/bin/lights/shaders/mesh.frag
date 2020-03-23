#version 450

layout (binding = 1) uniform UniformBufferObject {
    mat4 projection;
    vec3 pos;
    vec3 color;
    float ambient;
    float specular;
} light;

layout (binding = 2) uniform sampler2D texSampler;

layout (location = 0) in vec2 text_cord;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 model_position;

layout (location = 3) in vec4 color;

layout (location = 0) out vec4 outColor;


void main() {
    //Lets find the vector ray between light source and model position
    vec3 light_ray_vector = normalize(light.pos - model_position.xyz);

    //Lets normalize some values
    vec3 _model_pos = normalize(model_position);
    vec3 _normal = normalize(normal);

    //Calculate the degree between model and light ray, with dot function
    float light_dot_function = dot(_normal, light_ray_vector);

    // Lets make sure light dot is > -1 and calculate with light color
    vec3 light_color = max(light_dot_function, 0.0) * light.color;

    vec4 result_color = vec4(light.ambient + light_color, 1.0);

    outColor = result_color * texture(texSampler, text_cord);
}
