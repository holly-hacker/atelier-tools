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
- `.g1t` parsing for most formats
- DDS decoding:
  - Texture formats:
    - [x] RGBA8
    - [x] BC1/DXT1
    - [x] BC3/DXT5
    - [ ] BC6H
    - [x] BC7

g1t texture support:

- Atelier Sophie: 1725/1725 (100.0%) textures supported
- Atelier Firis: 2645/2648 (99.9%) textures supported
- Atelier Lydie and Suelle: 5176/5178 (100.0%) textures supported
- Atelier Ryza: 2194/2197 (99.9%) textures supported
- Atelier Ryza 2: 2941/2949 (99.7%) textures supported
- Atelier Sophie 2: 3284/3295 (99.7%) textures supported
- Atelier Ryza 3: 6270/6290 (99.7%) textures supported

<!-- Update note: make sure to use the -d flag -->

## Goals

- Be easier to understand than [gust_tools](https://github.com/VitaSmith/gust_tools)
- Be usable as a library
- Support unpacking enough file types to create a decent auto-generated wiki
- (ideally) be faster than gust_tools

## Anti-goals

- Be a complete replacement for [gust_tools](https://github.com/VitaSmith/gust_tools): This project
  is (currently) not focused on encoding external assets to be used in the game, it only tries to
  read these assets or convert them to usable formats.
- Support every single format and every single game: This project was born to support
  [atelier-wiki](https://github.com/holly-hacker/atelier-wiki) (as to not need any C dependencies)
  and because I thought it was a fun project. As such, I won't try to support every Atelier game
  ever made, and I'm unlikely to accept pull requests for games I don't personally own.

## Acknowledgements

- [gust_tools](https://github.com/VitaSmith/gust_tools) by [VitaSmith](https://github.com/VitaSmith), which was used to help figure out various algorithms and file formats.
