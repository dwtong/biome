# Biome

## What is it?

Biome is a long-form performance focused sample player built to be portable and run on small platforms, such as a Raspberry Pi/Pisound.

## How does it work?

Biome has four channels, and each channel can play a stereo sample from file. Each channel has it's own directory which it will load samples from.

Sample selection, playback, and manipulation is controlled with a [monome grid](https://monome.org/docs/grid/) and a midi controller.

Midi parameters and sample directories can be configured in `settings.yml`.

## Building

Biome can be built using [cross](https://github.com/cross-rs/cross/blob/main/docs/getting-started.md), by running:
```sh
cross run --target aarch64-unknown-linux-gnu
```
