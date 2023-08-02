# Atelier Tools

Tools and libraries for Atelier games by Gust.

Functionality will likely be limited to the Atelier Ryza games, as those are the only ones I own.

## Functionality

- `.pak` decoding:
  - [ ] Atelier Ryza
  - [ ] Atelier Ryza 2
  - [x] Atelier Ryza 3
- `.g1t` parsing and decoding:
  - [ ] RGBA8 (0x01, 0x02)
  - [ ] DXT1 (0x59)
  - [ ] DXT5 (0x5B)
  - [ ] BC6H (0x5E)
  - [x] BC7 (missing mode 2 support which is unused in tested games)
  - [ ] Other types that are unused in tested games

Tested games:

- Atelier Ryza 3

## Goals

- Be easier to understand and use than gust_tools
- Support unpacking enough file types to create a decent auto-generated wiki

## Acknowledgements

- [gust_tools](https://github.com/VitaSmith/gust_tools) by [VitaSmith](https://github.com/VitaSmith), which was used to figure out algorithms and file formats.
