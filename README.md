# Stripper File Assistant

Takes two VMFs and converts the differences between entities to a Stripper:Source (I didn't come up with the name) config file for use on servers. This was inspired by a friend's project.

## Installation

Binaries can be found on the releases page. If you can't find it, then click the download button below:

<a href="https://github.com/Awesomerly/vmf-to-stripper-cfg/releases">
  <img src="https://i.imgur.com/kQlA8P9.gif" alt="Obnoxious Download Button" width="140"/>
</a>

Alternatively:
- Install from Cargo:
  - `cargo install vmf_to_stripper`
- Using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)
  - `cargo binstall vmf_to_stripper`

## How to Use
The wonderful [bspsrc](https://github.com/ata4/bspsrc) can be used to decompile an existing map.
Make a copy of your VMF and then add/remove/modify entities in the copy.

Afterwards, run the following command to write the config to a file: `vmf_to_stripper [Unmodified VMF] [Modified VMF] > [MAPNAME].cfg`

## Thanks
- Thank you IaVashik for making the [vmf_forge](https://github.com/IaVashik/vmf-forge) library!
- Thanks to [cargo-dist](https://github.com/axodotdev/cargo-dist) for helping set up the CI :)

### Todo:

- [ ] diff connections
- [ ] create lasers around brushes when in visgroup
- [ ] annotate output better for readability
