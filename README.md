# Atelier Tools

Tools and libraries for Atelier games by Gust.

Functionality will likely be limited to modern Atelier games that I own.

## Functionality

- `.pak` decoding:
  - [x] Atelier Sophie DX
    - [ ] Atelier Sophie DX Artbook
  - [x] Atelier Firis DX
    - [ ] Atelier Firis DX Artbook
  - [x] Atelier Lydie & Suelle DX
    - [ ] Atelier Lydie & Suelle DX Artbook
  - [x] Atelier Ryza
  - [x] Atelier Ryza 2
  - [ ] Atelier Sophie 2 (untested but should work)
  - [x] Atelier Ryza 3
- `.g1t` parsing and decoding:
  - Platforms:
    - [x] Windows
    - [ ] Playstation 2
    - [ ] Playstation 4
  - Texture formats:
    - [ ] RGBA8 (0x01, 0x02)
    - [x] DXT1 (0x59)
    - [ ] DXT5 (0x5B)
    - [ ] BC6H (0x5E)
    - [x] BC7 (missing mode 2 support which is unused in tested games)
    - [ ] Other types that are unused in tested games

g1t texture support:

- Atelier Sophie: 463/1725 (26.8%) textures supported
- Atelier Firis: 1459/2648 (55.1%) textures supported
- Atelier Lydie & Suelle: 1447/5178 (27.9%) textures supported
- Atelier Ryza: 481/2197 (21.9%) textures supported
- Atelier Ryza 2: 620/2949 (21.0%) textures supported
- Atelier Ryza 3: 5740/5964 (96.2%) textures supported

## Goals

- Be easier to understand than [gust_tools](https://github.com/VitaSmith/gust_tools)
- Be usable as a library
- Support unpacking enough file types to create a decent auto-generated wiki
- (ideally) be faster than gust_tools

## Anti-goals

- Be a complete replacement for [gust_tools](https://github.com/VitaSmith/gust_tools): This project
  is (currently) not focused on encoding external assets to be used in the game, it only tries to
  read these assets or convert them to useable formats.
- Support every single format and every single game: This project was born to support
  [atelier-wiki](https://github.com/holly-hacker/atelier-wiki) (as to not need any C dependencies)
  and because I thought it was a fun project. As such, I won't try to support every Atelier game
  ever made and I'm unlikely to accept pull requests for games I don't personally own.

## Acknowledgements

- [gust_tools](https://github.com/VitaSmith/gust_tools) by [VitaSmith](https://github.com/VitaSmith), which was used to help figure out various algorithms and file formats.
