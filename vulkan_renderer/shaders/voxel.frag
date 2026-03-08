#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) out vec4 uFragColor;
layout (location = 0) in vec4 ray_direction;

void main() {
    uFragColor = vec4(ray_direction.x * 0.5 +0.5,0.,0.,1.);
}