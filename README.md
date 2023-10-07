# Atelier Tools

Tools and libraries for Atelier games by Gust.

Functionality will likely be limited to modern Atelier games that I own.

## Functionality

- `.pak` decoding:
  - [ ] Atelier Sophie
  - [ ] Atelier Firis
  - [ ] Atelier Lydie & Suelle
  - [ ] Atelier Ryza
  - [ ] Atelier Ryza 2
  - [x] Atelier Ryza 3
  - [ ] Atelier Sophie 2
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

Tested games:

- Atelier Ryza 3: 5740/5964 (96.2%) textures supported

## Goals

- Be easier to understand and use than [gust_tools](https://github.com/VitaSmith/gust_tools)
- Support unpacking enough file types to create a decent auto-generated wiki

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
