#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
layout (location = 0) in vec4 pos;
layout (location = 0) out vec4 ray_direction;
void main() {

    gl_Position = pos;
    ray_direction = pos;
}