{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in with pkgs; rec {
        devShell = mkShell rec {
          buildInputs = [
            libxkbcommon
            libGL

            # for smithay client toolkit
            pkg-config
            fontconfig

            # WINIT_UNIX_BACKEND=wayland
            wayland

            # gpu stuff (turns out this wasn't the issue)
            # opencl-headers
            ocl-icd
            # intel-ocl # this is apparently unfree
            # mesa
            # # for dbg purposes
            # clinfo

            # # WINIT_UNIX_BACKEND=x11
            # xorg.libXcursor
            # xorg.libXrandr
            # xorg.libXi
            # xorg.libX11
          ];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      });
}
