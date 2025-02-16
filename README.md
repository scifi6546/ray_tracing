This is the Root repository for my raytracing project.
There are two ray tracers, a multithreaded cpu based raytracer and an in progress vulkan raytracer.

# Cpu Raytracer

![example render](examples/voxel_world.png)
The cpu raytracer initially used Raytracing in a Weekend book series by Peter Shirley (https://raytracing.github.io/) as
a guide.
It has however expanded far past the material taught in the book.

Supported Features

* Gui Editor
* Multithreading
* Post Processing Pipeline
* Many Materials

see https://github.com/scifi6546/ray_tracing/tree/master/cpu_rt for more information.

![example render](examples/cornell_smoke.png)
![diffuse light](examples/light_texture.png)

# Vulkan Raytracer

The vulkan raytracer is still in progress however it supports runtime configurable render pipelines.
It uses the vulkan raytracing extensions. It uses shaders writen in glsl and compiled to spir-v at build time.

# Minya Renderer and an inprogress vulkan based raytracer.

A CPU raytracer based off of the Raytracing in a Weekend book series by Peter Shirley (https://raytracing.github.io/)
and an in progress real time Vulkan rendering engine. It uses a bounding volume hierarchy inorder to accelerate ray
tracing.

## Cpu Renderer

![example render](examples/cornell_smoke.png)

The cpu raytracer is based off of the Raytracing in a Weekend series by Peter Shirley. It features multiple materials, a
GUI written with egui and support for loading arbitrary scenes. The supported materials are described below.

## Vulkan Renderer

An in progress vulkan rendering engine is currently under construction. It is planned to use ray tracing. Currently it
supports loading some of the scenes from the cpu Minya renderer.

A rendering of the cornell box is shown below.

![vulkan cornell_box](examples/vulkan%20scene.png)

## Scene Schema

```mermaid
erDiagram
    MetaData["MetaData"] {
        int version "current version is 0"
        int last_save_unix_time
    }
    scene {
        blob scene_id PK
        string shader
        bigint background_id FK
        blob camera_id FK
        blob sun_id FK
    }
    Sun {
        double phi
        double theta
        double radius
    }
    Camera["Camera"] {
        blob camera_id PK
        blob camera_info_id FK
    }
    CameraInfo["camera_info"] {
        blob camera_info_id
        double aspect_ratio
        double fov
        double origin_x
        double origin_y
        double origin_z
        double look_at_x
        double look_at_y
        double look_at_z
        double up_vector_x
        double up_vector_y
        double up_vector_z
        double aperture
        double focus_distance
        double start_time
        double end_time
    }

    Entity {
        t entity_id PK
        t hittable_0
        t hittable_1
    }
    Background["Background"] {
        blob background_id PK
        bigint constant_color_id FK
        bigint sun_sky_id FK
        bigint sky_id FK
    }
    Sky["Sky Background"] {
        blob sky_id PK
        double intensity
    }
    SunSky["SunSky Background"] {
        blob sun_sky_id
        double intensity
        double sun_radius
        double sun_theta
        double sun_phi
        double sun_brightness
    }

    ConstantColor["Constant Color Background"] {
        blob constant_id
        double red
        double green
        double blue
    }
%% Material
    Material["Material"] {
    }
%% Hittable Tables
    ConstantMedium["Constant Medium"] {
        blob constant_medium_id PK
        blob boundry_id FK
        double density
    }
    OctTree["Oct Tree"] {
    }
    XYRect["XY Rectangle"] {
    }
    XZRect["XZ Rectangle"] {
    }
    YZRect["YZ Rectangle"] {
    }
    RenderBox["Render Box"] {
    }
    Sphere["Sphere"] {
    }
    Voxel["Voxel World"] {
    }
%% Connections
    scene |{ -- || Background: ""
    scene || -- o{ Entity: ""
    scene || -- o| Sun: ""
    Camera || -- || scene: ""
    Camera || -- || CameraInfo: ""
    Background |{ -- o| SunSky: ""
    Sky |o -- |{ Background: ""
    Background |{ -- o| ConstantColor: ""
%% Hittable Connections
    Entity || -- || ConstantMedium: ""
    Entity || -- || OctTree: ""
    Entity || -- || XYRect: ""
    Entity || -- || XZRect: ""
    Entity || -- || YZRect: ""
    Entity || -- || RenderBox: ""
    Entity || -- || Sphere: ""
    Entity || -- || Voxel: ""
%% Inside Hittable Connections
    ConstantMedium || -- || Entity: ""
```

## Voxel Tree Schema

```mermaid
erDiagram
    RootNodes["Root Nodes"] {
        id ChildId
        int size
    }

    Node {
        id NodeId PK
        VarChar(5) LeafOrParent "Must be 'leaf' or 'parent'"
    }
    Parent {
        id ParentId PK
        id NodeId FK
        id Child00 FK
        id Child01 FK
        id Child10 FK
        id Child11 FK
    }
    Leaf {
        id ParentId PK
        id LeafId FK
        VarChar MaterialType
    }
    SolidMaterial {
        id LeafId
        blob MaterialData
    }
    TranslucentMaterial["Translucent Material"] {
        id LeafId
        blob MaterialData
    }
    Node |{--|| Parent: ""
    Node |{--|| Leaf: ""
    RootNodes ||--o| Node: ""
    SolidMaterial o{--|| Leaf: ""
    TranslucentMaterial o{--|| Leaf: ""
```
