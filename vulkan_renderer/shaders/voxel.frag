#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) out vec4 uFragColor;
layout (location = 0) in vec4 ray_direction;

void main() {
    vec2 box_min = vec2(-0.5,-0.5);
    vec2 box_max = vec2(0.5,0.5);
    bool in_box = ray_direction.x > box_min.x && 
        ray_direction.x < box_max.x &&
        ray_direction.y > box_min.y &&
        ray_direction.y < box_max.y;

    if(in_box){
        uFragColor = vec4(0.5,0.,0.8,1.);
    }else{
                uFragColor = vec4(0.0,0.,0.,1.);
    }
  
}