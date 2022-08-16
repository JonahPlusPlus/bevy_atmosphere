#define_import_path bevy_atmosphere::math

let PI: f32 = 3.141592653589793;
let ISTEPS: u32 = 16u;
let JSTEPS: u32 = 8u;

fn rsi(rd: vec3<f32>, r0: vec3<f32>, sr: f32) -> vec2<f32> {
    // ray-sphere intersection that assumes
    // the sphere is centered at the origin.
    // No intersection when result.x > result.y
    let a = dot(rd, rd);
    let b = 2.0 * dot(rd, r0);
    let c = dot(r0, r0) - (sr * sr);
    let d = (b * b) - (4.0 * a * c);

    if d < 0.0 {
        return vec2<f32>(1e5, -1e5);
    } else {
        return vec2<f32>(
            (-b - sqrt(d)) / (2.0 * a),
            (-b + sqrt(d)) / (2.0 * a)
        );
    }
}

fn render_atmosphere(r: vec3<f32>, r0: vec3<f32>, p_sun: vec3<f32>, i_sun: f32, r_planet: f32, r_atmos: f32, k_rlh: vec3<f32>, k_mie: f32, sh_rlh: f32, sh_mie: f32, g: f32) -> vec3<f32> {
    // Normalize the ray direction and sun position.
    let r = normalize(r);
    let p_sun = normalize(p_sun);

    // Calculate the step size of the primary ray.
    var p = rsi(r, r0, r_atmos);
    if (p.x > p.y) { return vec3<f32>(0f, 0f, 0f); }
    p.y = min(p.y, rsi(r, r0, r_planet).x);
    let i_step_size = (p.y - p.x) / f32(ISTEPS);

    // Initialize the primary ray time.
    var i_time = 0.0;

    // Initialize accumulators for Rayleigh and Mie scattering.
    var total_rlh = vec3<f32>(0f, 0f, 0f);
    var total_mie = vec3<f32>(0f, 0f, 0f);

    // Initialize optical depth accumulators for the primary ray.
    var i_od_rlh = 0f;
    var i_od_mie = 0f;

    // Calculate the Rayleigh and Mie phases.
    let mu = dot(r, p_sun);
    let mumu = mu * mu;
    let gg = g * g;
    let p_rlh = 3.0 / (16.0 * PI) * (1.0 + mumu);
    let p_mie = 3.0 / (8.0 * PI) * ((1.0 - gg) * (mumu + 1.0)) / (pow(1.0 + gg - 2.0 * mu * g, 1.5) * (2.0 + gg));

    // Sample the primary ray.
    for (var i = 0u; i < ISTEPS; i++) {
        // Calculate the primary ray sample position.
        let i_pos = r0 + r * (i_time + i_step_size * 0.5);

        // Calculate the height of the sample.
        let i_height = length(i_pos) - r_planet;

        // Calculate the optical depth of the Rayleigh and Mie scattering for this step.
        let od_step_rlh = exp(-i_height / sh_rlh) * i_step_size;
        let od_step_mie = exp(-i_height / sh_mie) * i_step_size;

        // Accumulate optical depth.
        i_od_rlh += od_step_rlh;
        i_od_mie += od_step_mie;

        // Calculate the step size of the secondary ray.
        let j_step_size = rsi(p_sun, i_pos, r_atmos).y / f32(JSTEPS);

        // Initialize the secondary ray time.
        var j_time = 0f;

        // Initialize optical depth accumulators for the secondary ray.
        var j_od_rlh = 0f;
        var j_od_mie = 0f;

        // Sample the secondary ray.
        for (var j = 0u; j < JSTEPS; j++) {

            // Calculate the secondary ray sample position.
            let j_pos = i_pos + p_sun * (j_time + j_step_size * 0.5);

            // Calculate the height of the sample.
            let j_height = length(j_pos) - r_planet;

            // Accumulate the optical depth.
            j_od_rlh += exp(-j_height / sh_rlh) * j_step_size;
            j_od_mie += exp(-j_height / sh_mie) * j_step_size;

            // Increment the secondary ray time.
            j_time += j_step_size;
        }

        // Calculate attenuation.
        let attn = exp(-(k_mie * (i_od_mie + j_od_mie) + k_rlh * (i_od_rlh + j_od_rlh)));

        // Accumulate scattering.
        total_rlh += od_step_rlh * attn;
        total_mie += od_step_mie * attn;

        // Increment the primary ray time.
        i_time += i_step_size;
    }

    // Calculate and return the final color.
    return i_sun * (p_rlh * k_rlh * total_rlh + p_mie * k_mie * total_mie);
}
