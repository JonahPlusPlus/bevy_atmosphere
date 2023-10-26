#define_import_path bevy_atmosphere::utils

const PI: f32 = 3.141592653589793;
const M_PI_2F: f32 = 6.283185307179586;
const D_PI_2F: f32 = 1.5707963267948966;
const D_2_PI: f32 = 0.6366197723675814;
const D_1_PI: f32 = 0.3183098861837907;
const D_1_6F: f32 = 0.1666666666666666666;

// translated from Blender's source code
const quadrature_nodes: array<f32, 8> = array<f32, 8>(0.006811185292f, 0.03614807107f, 0.09004346519f, 0.1706680068f, 0.2818362161f, 0.4303406404f, 0.6296271457f, 0.9145252695f);
const quadrature_weights: array<f32, 8> = array<f32, 8>(0.01750893642f, 0.04135477391f, 0.06678839063f, 0.09507698807f, 0.1283416365f, 0.1707430204f, 0.2327233347f, 0.3562490486f);

fn density_rayleigh(height: f32, scale_height: f32) -> f32 {
    return exp(-height / scale_height);
}

fn density_mie(height: f32, scale_height: f32) -> f32 {
    return exp(-height / scale_height);
}

fn density_ozone(height: f32) -> f32 {
    // I've seen 20000 and 35000 used, so something might be wrong here depending on what the units are
    if height >= 32000. {
        return exp(-(height - 32000.) / 10000.);
    } else if height >= 10000. {
        return (height - 10000.) / (32000. - 10000.);
    }
    return 0.0;
}

// Return the scale of the ray direction (near, far) for a ray-sphere intersection
fn rsi(ray_dir: vec3<f32>, ray_origin: vec3<f32>, sphere_radius: f32) -> vec2<f32> {
    // ray-sphere intersection that assumes
    // the sphere is centered at the origin.
    // No intersection when result.x > result.y
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(ray_dir, ray_origin);
    let c = dot(ray_origin, ray_origin) - (sphere_radius * sphere_radius);
    let d = (b * b) - (4.0 * a * c);

    if d < 0.0 {
        return vec2<f32>(1e5, -1e5);
    } else {
        let d = sqrt(d);
        return vec2<f32>(-b - d, -b + d) / (2.0 * a);
    }
}

// translated from Blender's source code
fn ray_optical_depth(ray_origin: vec3<f32>, ray_dir: vec3<f32>, planet_radius: f32, atmosphere_radius: f32, rayleigh_scale_height: f32, mie_scale_height: f32) -> vec3<f32> {
    // This function computes the optical depth along a ray.
    // Instead of using classic ray marching, the code is based on Gauss-Laguerre quadrature,
    // which is designed to compute the integral of f(x)*exp(-x) from 0 to infinity.
    // This works well here, since the optical depth along the ray tends to decrease exponentially.
    // By setting f(x) = g(x) exp(x), the exponentials cancel out and we get the integral of g(x).
    // The nodes and weights used here are the standard n=6 Gauss-Laguerre values, except that
    // the exp(x) scaling factor is already included in the weights.
    // The parametrization along the ray is scaled so that the last quadrature node is still within
    // the atmosphere.
    let ray_length = rsi(ray_dir, ray_origin, atmosphere_radius).y;

    let segment = ray_length * ray_dir;

    // instead of tracking the transmission spectrum across all wavelengths directly,
    // we use the fact that the density always has the same spectrum for each type of
    // scattering, so we split the density into a constant spectrum and a factor and
    // only track the factors
    var optical_depth: vec3<f32> = vec3<f32>(0.0);
    // for (var i = 0u; i < JSTEPS; i++) {}
        {let height = length(ray_origin + (quadrature_nodes[0] * segment)) - planet_radius;optical_depth += quadrature_weights[0] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}
        {let height = length(ray_origin + (quadrature_nodes[1] * segment)) - planet_radius;optical_depth += quadrature_weights[1] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}
        {let height = length(ray_origin + (quadrature_nodes[2] * segment)) - planet_radius;optical_depth += quadrature_weights[2] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}
        {let height = length(ray_origin + (quadrature_nodes[3] * segment)) - planet_radius;optical_depth += quadrature_weights[3] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}
        {let height = length(ray_origin + (quadrature_nodes[4] * segment)) - planet_radius;optical_depth += quadrature_weights[4] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}
        {let height = length(ray_origin + (quadrature_nodes[5] * segment)) - planet_radius;optical_depth += quadrature_weights[5] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}
        {let height = length(ray_origin + (quadrature_nodes[6] * segment)) - planet_radius;optical_depth += quadrature_weights[6] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}
        {let height = length(ray_origin + (quadrature_nodes[7] * segment)) - planet_radius;optical_depth += quadrature_weights[7] * vec3<f32>(density_rayleigh(height, rayleigh_scale_height), density_mie(height, mie_scale_height), density_ozone(height));}

    return (optical_depth * ray_length);
}

/// shell index to (0-1) proportional height
///
/// a rough approximation of the optimal shell radii using base 2 logarithms (factor of 6)
fn shell_index_to_height(shell_index: f32, n_shells: f32) -> f32 {
    return 1.0 - (D_1_6F * log2(64.0 - (63.0 * shell_index / n_shells)));
}

/// (0-1) proportional height to shell index
///
/// a rough approximation of the optimal shell radii using base 2 logarithms (factor of 6)
fn height_to_shell_index(height: f32, n_shells: f32) -> f32 {
    return clamp((n_shells / 63.0) * (64.0 - exp2(6.0 * (1.0 - height))), 0f, n_shells);
}
