# Atelier Tools

Tools and libraries for Atelier games by Gust.

Functionality will likely be limited to PC versions of modern Atelier games that I own.

## Functionality

- `.pak` decoding:
  - [x] Atelier Sophie DX
    - [ ] Atelier Sophie DX Artbook
  - [x] Atelier Firis DX
    - [ ] Atelier Firis DX Artbook
  - [x] Atelier Lydie and Suelle DX
    - [ ] Atelier Lydie and Suelle DX Artbook
  - [x] Atelier Ryza
  - [x] Atelier Ryza 2
  - [x] Atelier Sophie 2
  - [x] Atelier Ryza 3
- `.g1t` parsing and decoding:
  - Platforms:
    - [x] Windows
    - [ ] Playstation 2
    - [ ] Playstation 4
    - [ ] Platforms not present in PC versions
  - Texture formats:
    - [ ] RGBA8 (0x01, 0x02)
    - [x] BC1/DXT1 (0x59)
    - [x] BC3/DXT5 (0x5B)
    - [ ] BC6H (0x5E)
    - [x] BC7

g1t texture support:

- Atelier Sophie: 1716/1725 (99.5%) textures supported
- Atelier Firis: 2634/2648 (99.5%) textures supported
- Atelier Lydie and Suelle: 5166/5178 (99.8%) textures supported
- Atelier Ryza: 2184/2197 (99.4%) textures supported
- Atelier Ryza 2: 2931/2949 (99.4%) textures supported
- Atelier Sophie 2: 3272/3295 (99.3%) textures supported
- Atelier Ryza 3: 6217/6290 (98.8%) textures supported

<!-- Update note: make sure to use the -d flag -->

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
