#define_import_path bevy_atmosphere::types

struct Atmosphere {
    ray_origin: vec3<f32>,
    sun_position: vec3<f32>,
    sun_intensity: f32,
    planet_radius: f32,
    atmosphere_radius: f32,
    rayleigh_coefficient: vec3<f32>,
    rayleigh_scale_height: f32,
    mie_coefficient: f32,
    mie_scale_height: f32,
    mie_direction: f32,
}
