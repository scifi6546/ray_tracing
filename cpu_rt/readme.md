### Lambertian

A diffuse material that scatters light in random directions and absorbs a specific color based on the color of the
object. The following example render can be used by loading the "Lambertian Demonstration" scenario.

![lambertian](examples/lambertian.png)

### Metal

Models a metallic surface. It reflects rays about the normal vector with a random blur based on a predifined "fuzz"
variable inorder to model metalic surfaces that are not completly polished.
An example of a smooth render is shown below.
![metallic smooth](examples/metallic_smooth.png)

When a surface is rough the rays are more likely to be reflected in a random direction and the same scene as above
is shown except with a fuzz of 0.6.

![metallic](examples/metallic_rough.png)

### Dielectric

A Dielectric Material simulates a transparent material with diffraction. A dielectric material can be used to simulate
glass. An example with low refraction shown below.
![low refraction](../examples/refraction_low.png)

An example with high refraction is shown below. Notice how the orange sphere is distorted by the glassy sphere.
![high refraction](../examples/refraction_high.png)

### DiffuseLight

A diffuse light source simply emits light. The light can emmit according to a texture. The texture can be proceedurally
generated noise, an image texture or a constant background.

An example of a diffuse light emmitting light according to a texture.
![diffuse light](../examples/light_texture.png)

## Supported Shapes

Many different renderable shapes are supported

### Sphere

Spheres support

### Axis Aligned Box

### Axis Aligned Rectangles

### Voxel Grid

### Voxel Oct Tree

## Supported Transformations

### Translation

### Rotation

## Egui Gui

The gui was built inorder to make debuging and interacting with the renderer easier. It currently has a window for
selecting new scenes and a debug log window. The debug log window utilizes the [log](https://crates.io/crates/log) crate
inorder to provide a convient API that differentiates between debug, info, warnings and errors.

![gui](../examples/gui.png)

## Scene Storage Format

The renderer supports saving scenes to disk. The
scenes are stored in a sqlite3 database.
The schema is shown below.

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
