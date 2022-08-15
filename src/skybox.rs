use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

/// Generates an inverted box mesh with face UVs that fit inside a `pipeline::SIZE` square with a 1 pixel border
pub fn mesh(far: f32, res: f32) -> Mesh {
    let size = (far * f32::sqrt(0.5)) - 1.0; // sqrt(0.5) is the ratio between squares separated by a circle
                                             // where one lies on the outside of the circle (edges) and the other lies on the inside of the circle (corners)
                                             // this is necessary since while the faces of the skybox may be seen, the corners and edges probably won't, since they don't lie on the radius of the far plane
    let (vertices, indices) = (
        &[
            // (+, 0, 0) Left Side
            (
                [size, size, size],
                [-1., 00., 00.],
                [
                    (0. / 6.) + (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, +, +)
            (
                [size, size, -size],
                [-1., 00., 00.],
                [
                    (0. / 6.) + (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (+, +, -)
            (
                [size, -size, size],
                [-1., 00., 00.],
                [
                    (1. / 6.) - (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, -, +)
            (
                [size, -size, -size],
                [-1., 00., 00.],
                [
                    (1. / 6.) - (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (+, -, -)
            // (0, +, 0) Top Side
            (
                [-size, size, size],
                [00., -1., 00.],
                [
                    (1. / 6.) + (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, +, +)
            (
                [-size, size, -size],
                [00., -1., 00.],
                [
                    (1. / 6.) + (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, +, +)
            (
                [size, size, size],
                [00., -1., 00.],
                [
                    (2. / 6.) - (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, +, -)
            (
                [size, size, -size],
                [00., -1., 00.],
                [
                    (2. / 6.) - (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, +, -)
            // (0, 0, +) Front Side
            (
                [size, size, size],
                [00., 00., -1.],
                [
                    (2. / 6.) + (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, +, +)
            (
                [size, -size, size],
                [00., 00., -1.],
                [
                    (2. / 6.) + (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (+, -, +)
            (
                [-size, size, size],
                [00., 00., -1.],
                [
                    (3. / 6.) - (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (-, +, +)
            (
                [-size, -size, size],
                [00., 00., -1.],
                [
                    (3. / 6.) - (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, -, +)
            // (-, 0, 0) Right Side
            (
                [-size, size, size],
                [01., 00., 00.],
                [
                    (3. / 6.) + (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (-, +, +)
            (
                [-size, -size, size],
                [01., 00., 00.],
                [
                    (3. / 6.) + (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, -, +)
            (
                [-size, size, -size],
                [01., 00., 00.],
                [
                    (4. / 6.) - (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (-, +, -)
            (
                [-size, -size, -size],
                [01., 00., 00.],
                [
                    (4. / 6.) - (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, -, -)
            // (0, -, 0) Bottom Side
            (
                [-size, -size, size],
                [00., 01., 00.],
                [
                    (4. / 6.) + (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, -, +)
            (
                [size, -size, size],
                [00., 01., 00.],
                [
                    (4. / 6.) + (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (+, -, -)
            (
                [-size, -size, -size],
                [00., 01., 00.],
                [
                    (5. / 6.) - (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (-, -, +)
            (
                [size, -size, -size],
                [00., 01., 00.],
                [
                    (5. / 6.) - (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, -, -)
            // (0, 0, -) Back Side
            (
                [size, size, -size],
                [00., 00., 01.],
                [
                    (5. / 6.) + (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, +, -)
            (
                [-size, size, -size],
                [00., 00., 01.],
                [
                    (5. / 6.) + (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, +, -)
            (
                [size, -size, -size],
                [00., 00., 01.],
                [
                    (6. / 6.) - (1. / (6. * res)),
                    0. + (1. / res),
                ],
            ), // (+, -, -)
            (
                [-size, -size, -size],
                [00., 00., 01.],
                [
                    (6. / 6.) - (1. / (6. * res)),
                    1. - (1. / res),
                ],
            ), // (-, -, -)
        ],
        &[
            00, 01, 02, 02, 01, 03, // (+, 0, 0)
            04, 05, 06, 06, 05, 07, // (0, +, 0)
            08, 09, 10, 10, 09, 11, // (0, 0, +)
            12, 13, 14, 14, 13, 15, // (-, 0, 0)
            16, 17, 18, 18, 17, 19, // (0, -, 0)
            20, 21, 22, 22, 21, 23, // (0, 0, -)
        ],
    );

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U16(indices.to_vec())));
    mesh
}
