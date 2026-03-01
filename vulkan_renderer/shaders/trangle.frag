#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 o_color;
layout (location = 1) in vec2 o_uv;
layout (location = 0) out vec4 uFragColor;

layout(binding = 0) uniform sampler2D samplerColor;
void main() {
    vec4 texture_color = texture(samplerColor, o_uv);
    uFragColor = o_color * texture_color;

}