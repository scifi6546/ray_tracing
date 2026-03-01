glslc .\shaders\trangle.frag  --target-env=vulkan1.3 -o .\shaders\frag.spv 
glslc .\shaders\trangle.vert  --target-env=vulkan1.3 -o .\shaders\vert.spv


glslc .\shaders\voxel.frag  --target-env=vulkan1.3 -o .\shaders\voxel_frag.spv 
glslc .\shaders\voxel.vert  --target-env=vulkan1.3 -o .\shaders\voxel_vert.spv