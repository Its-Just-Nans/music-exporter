# music-exporter [![crates.io version](https://img.shields.io/crates/v/music-exporter)](https://crates.io/crates/music-exporter) ![crates.io downloads](https://img.shields.io/crates/d/music-exporter)

Export music from different sources

## Install

```sh
cargo install music-exporter
music-exporter
```

## Usage

```sh
music-exporter
Exports music files for given platforms

Usage: music-exporter [OPTIONS] --music-file <MUSIC_FILE> --platform <PLATFORMS>...

Options:
      --env-file <ENV_FILE>
          Path to optional .env file

      --music-file <MUSIC_FILE>
          Path to the music file

      --platform <PLATFORMS>...
          Target platforms (must provide at least one)

          Possible values:
          - deezer:  Deezer platform
          - spotify: Spotify platform
          - youtube: Youtube platform

      --youtube-playlist-id <YOUTUBE_PLAYLIST_ID>
          Custom youtube playlist id
          [aliases: --ytb-playlist-id]
```

## License

[MIT](LICENSE)
