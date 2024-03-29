# examples

## [basic.rs](basic.rs)

![basic example image](images/basic-example.png)

The most simple usage of bevy_atmosphere. It uses the default `Nishita` model, which has Earth-like parameters. Feel free to copy it as a template!

## [cycle.rs](cycle.rs)

![cycle example image](images/cycle-example.png)

A daylight cycle with `Nishita`, updating every 50ms.

## [detection.rs](detection.rs)

![detection example image](images/detection-example.gif)

Demonstrates adding and removing the skybox with the `detection` feature. Use `LMouse` to add and `RMouse` to remove.

## [gradient.rs](gradient.rs)

![gradient example image](images/gradient-example.png)

Demonstrates using `Gradient` model. Use the number keys to switch presets. (Preset 3 shown here)

## [models.rs](models.rs)

![models example image](images/models-example.gif)

Demonstrates using the different models available. Use the letter keys to switch models.

## [nishita.rs](nishita.rs)

![nishita example image](images/nishita-example.png)

Demonstrates using `Nishita` model. Use the number keys to switch presets. (Preset 2 shown here)

## [settings.rs](settings.rs)

![settings example image](images/settings-example.png)

Demonstrates using `AtmosphereSettings` to update resolution and dithering on the fly, similar to how an in-game quality settings menu could operate.
Use the number keys to switch resolution presets and the spacebar to toggle dithering.

## [splitscreen.rs](splitscreen.rs)

![splitscreen example image](images/splitscreen-example.png)

A split-screen application, demonstrating bevy_atmosphere's flexibility for local multiplayer games.
