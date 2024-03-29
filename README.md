# Minya Renderer
A CPU raytracer based off of the Raytracing in a Weekend book series by Peter Shirley (https://raytracing.github.io/) and an in progress real time Vulkan rendering engine. It uses a bounding volume hierarchy inorder to accelerate ray tracing.

## Cpu Renderer

![example render](examples/cornell_smoke.png)

The cpu raytracer is based off of the Raytracing in a Weekend series by Peter Shirley. It features multiple materials, a GUI written with egui and support for loading arbitrary scenes. It is currently single threaded. The supported materials are described below.

###  Lambertian

A diffuse material that scatters light in random directions and absorbs a specific color based on the color of the object. The following example render can be used by loading the "Lambertian Demonstration" scenario.

![lambertian](examples/lambertian.png)

### Metal

Models a metallic surface. It reflects rays about the normal vector with a random blur based on a predifined "fuzz" variable inorder to model metalic surfaces that are not completly polished.
An example of a smooth render is shown below.
![metallic smooth](examples/metallic_smooth.png)

When a surface is rough the rays are more likely to be reflected in a random direction and the same scene as above
is shown except with a fuzz of 0.6.

![metallic](examples/metallic_rough.png)




### Dielectric

A Dielectric Material simulates a transparent material with diffraction. A dielectric material can be used to simulate glass. An example with low refraction shown below.
![low refraction](examples/refraction_low.png)

An example with high refraction is shown below. Notice how the orange sphere is distorted by the glassy sphere.
![high refraction](examples/refraction_high.png)


### DiffuseLight

A diffuse light source simply emits light. The light can emmit according to a texture. The texture can be proceedurally generated noise, an image texture or a constant background.

An example of a diffuse light emmitting light according to a texture. 
![diffuse light](examples/light_texture.png)

## Supported Shapes

### Sphere

### Axis Aligned Box

### Axis Aligned Rectangles

## Supported Transformations

### Translation

### Rotation

## Egui Gui

The gui was built inorder to make debuging and interacting with the renderer easier. It currently has a window for selecting new scenes and a debug log window. The debug log window utilizes the [log](https://crates.io/crates/log) crate inorder to provide a convient API that differentiates between debug, info, warnings and errors.

INSERT PICTURE HERE


## Vulkan Renderer

An in progress vulkan rendering engine is currently under construction. It is planned to use ray tracing. Currently it supports loading some of the scenes from the cpu Minya renderer.

A rendering of the cornell box is shown below.

![vulkan cornell_box](examples/vulkan%20scene.png)

