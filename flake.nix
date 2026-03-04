{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      with pkgs;
      rec {
        devShell = mkShell rec {
          buildInputs = [
            libxkbcommon
            libGL

            # for smithay client toolkit
            pkg-config
            fontconfig

            # WINIT_UNIX_BACKEND=wayland
            wayland

            # for servo
            clang
            libclang
            cmake
            # make
            glibc.dev
            llvmPackages.stdenv
            # # llvmPackages_22.libc
            # llvmPackages_22.libllvm
            # llvmPackages_22.libc-full
            # llvmPackages_22.clang-unwrapped
            llvmPackages_20.libc
            llvmPackages_20.libllvm
            llvmPackages_20.libc-full
            llvmPackages_20.clang-unwrapped
            glib
            # fontsan dependencies:
            opentype-sanitizer # ots
            lz4
            brotli
            woff2

            # # something python fails without this
            # python3Packages.pyyaml
          ];
          # LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          # LIBCLANG_PATH = "${}";
        };
      }
    );
}
