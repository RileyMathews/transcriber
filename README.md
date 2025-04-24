# Transcriber
An audio playback application meant to aid in music practice and transcription.

![example screenshot of application](./docs/example.png)

The idea is to have a simple app that focuses on playback features to support fine grain listening controls for a single track. Features like playlists and more generic 'audio player' features will likely not be added.

# Installation
I plan to eventually try to add this package to the AUR. But for now installation via source is required. A simple clone and `cargo install --path .` should get the binary installed.

# Usage

First you can pre process some speed variations (requires the rubberband cli to be installed).

`transcriber /path/to/wave/file.wav --process-speed 1.1 --process-speed 1.5`

Note here the time passed in is the 'time stretch' value so values higher than 1 will be 'slower' tempo wise.

Then to load the song.

`transcriber /path/to/wave/file.wav`

Once the TUI is loaded you can use the playback controls to set bookmarks, jump between them, and switch between the processed speed versions on the fly.

## Features
* Create loop section
* Create and jump to bookmarks within a track
* Currently only supports wave files
* Pre process speed versions of the song and switch between them on the fly once playing

## Goals
* Be able to load more audio file formats
* Create audio wave form visualisation (May require a non TUI interface)
* Create multiple loop sections i.e. bookmarks for sections

# Implementation details
Implemented using the cpal crate for audio playback and the ratatui library for a simple TUI interface. I like the simplicity and keyboard driven style that TUI brings but I may explore porting or supporting a full GUI version as well.


