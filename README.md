# Transcriber
An audio playback application meant to aid in music practice and transcription.

The idea is to have a simple app that focuses on playback features to support fine grain listening controls for a single track. Features like playlists and more generic 'audio player' features will likely not be added.

# Installation
I plan to eventually try to add this package to the AUR. But for now installation via source is required. A simple clone and `cargo install --path .` should get the binary installed.

# Usage

`transcriber /path/to/wave/file.wav`

## Features
* Create loop section
* Create and jump to bookmarks within a track
* Currently only supports wave files

## Goals
* Save configuration per song so that bookmarks preserve across runs
* Be able to load more audio file formats
* Create audio wave form visualisation (May require a non TUI interface)
* Create multiple loop sections i.e. bookmarks for sections
* Be able to dynamically slow down a track (For now I recommend using the rubberband cli tool to pre process slow downed tracks)
* Pitch shifting (Again for now use rubberband to pre process shifted tracks)

# Implementation details
Implemented using the cpal crate for audio playback and the ratatui library for a simple TUI interface. I like the simplicity and keyboard driven style that TUI brings but I may explore porting or supporting a full GUI version as well.


