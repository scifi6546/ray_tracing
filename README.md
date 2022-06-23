# Minya Renderer
A CPU raytracer based off of the Raytracing in a Weekend book series by Peter Shirley (https://raytracing.github.io/) and an in progress real time Vulkan rendering engine. It uses a bounding volume hierarchy inorder to accelerate ray tracing.

## Cpu Renderer

The cpu raytracer is based off of the Raytracing in a Weekend series by Peter Shirley. It features multiple materials, a GUI written with egui and support for loading arbitrary scenes. It is currently single threaded. The supported materials are described below.

###  Lambertian

A diffuse material that scatters light in random directions and absorbs a specifc color based on the color of the object.

INSERT PRETTY SCREENSHOT!!!!!

### Metal

Models a metalic surface. It reflects rays about the normal vector with a random blur based on a predifined "fuzz" variable inorder to model metalic surfaces that are not completly polished.

INSERT SCREENSHOT OF NO FUZZ

INDERT SCREENSHOT OF A LOT OF FUZZ


### Dielectric


### DiffuseLight

The renderer also supports several shapes. The shapes are

### Sphere

### Axis Aligned Box

### Axis Aligned Rectangles

## Egui Gui

The gui was built inorder to make debuging and interacting with the renderer easier. It currently has a window for selecting new scenes and a debug log window. The debug log window utilizes the [log](https://crates.io/crates/log) crate inorder to provide a convient API that differentiates between debug, info, warnings and errors.

INSERT PICTURE HERE


## Vulkan Renderer

The vulkan renderer curently supports loading textures and arbitrary scenes.

INSERT SCREENSHOTS HERE!!!


# Performance on desktop
initial: 3.61 per frame

removed rc on translate and rotate: 3.59

refactor 3.4

improved pdf list: 3.2
