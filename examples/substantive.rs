use bevy::prelude::*;
use bevy_atmosphere::{prelude::*, pipeline::{AtmosphereImage, ATMOSPHERE_IMAGE_TEXTURE_DESCRIPTOR, ATMOSPHERE_ARRAY_TEXTURE_VIEW_DESCRIPTOR}};

// In order to make sure this example is used properly, it panics when run with the `procedural` feature.
#[allow(unreachable_code)]
fn main() {
    #[cfg(feature = "procedural")]
    panic!("Compile without `procedural` feature: `cargo run --example substantive --no-default-features --features detection`");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CustomAtmosphere)
        .add_plugin(AtmospherePlugin)
        .add_startup_system(setup)
        .run();
}

struct CustomAtmosphere;

impl Plugin for CustomAtmosphere {
    fn build(&self, app: &mut App) {
        let asset_server = app.world.resource::<AssetServer>();
        let handle = asset_server.load("SkyboxTexture.png");

        let mut images = app.world.resource_mut::<Assets<Image>>();
        let mut image = images.get_mut(&handle).expect("Failed to get image from handle");
        image.texture_view_descriptor = Some(ATMOSPHERE_ARRAY_TEXTURE_VIEW_DESCRIPTOR);
        image.texture_descriptor = ATMOSPHERE_IMAGE_TEXTURE_DESCRIPTOR(1024);

        let atmosphere_image = AtmosphereImage {
            handle,
            array_view: None,
        };
        app.insert_resource(atmosphere_image);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), AtmosphereCamera(None)));
}
