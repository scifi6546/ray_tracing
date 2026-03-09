glslc .\shaders\trangle.frag  --target-env=vulkan1.3 -o .\shaders\frag.spv 
glslc .\shaders\trangle.vert  --target-env=vulkan1.3 -o .\shaders\vert.spv

slangc.exe .\shaders\slang\present.slang -target spirv -o .\shaders\slang\present.spv

slangc.exe .\shaders\slang\voxel.slang -target spirv -o .\shaders\slang\voxel-frag.spv