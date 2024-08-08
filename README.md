# Game Boy Emulator

This is an emulator of the original Game Boy (model DMG) I'm making in Rust.

## Progress

- CPU: Finished (halt bug not implemented perfectly).
- Timer: Finished.
- Button input: Finished.
- Memory management: Working so far.
- Graphics: Work in progress.
- Sound: Not yet started.

## Requirements

SDL2 is required for sound and graphics output.

For Arch:

```bash
sudo pacman -S sdl2
```

or for Debian-based:

```bash
sudo apt install libsdl2-dev
```

## Usage

Run the desired rom file with:

```bash
cargo run -- [ROM]
```

## Controls

The controls I picked are the same as [mGBA](https://github.com/mgba-emu/mgba/blob/master/README.md#controls).

- **A**: X
- **B**: Z
- **Start**: Enter
- **Select**: Backspace
- **D-Pad**: Arrow Keys

## Resources

- [Pan Docs](https://gbdev.io/pandocs/)
- [awesome-gbdev](https://github.com/gbdev/awesome-gbdev)
- [Game Boy CPU (SM83) instruction set](https://gbdev.io/gb-opcodes/optables/)
- [Game Boy: Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
- [Gameboy Doctor](https://github.com/robert/gameboy-doctor) (this one lowkey saved the project)
- [RGBDS Language Reference](https://rgbds.gbdev.io/docs/v0.7.0/gbz80.7)

Thank you so much to everyone who took the time to create these awesome resources.
