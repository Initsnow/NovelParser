{
  description = "Tauri + React + Vite + pnpm Dev Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        libraries = with pkgs; [
          librsvg
          webkitgtk_4_1
          gtk3
          glib
          cairo
          pango
          atk
          gdk-pixbuf
          libsoup_3
          openssl
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            wrapGAppsHook3
            cargo
            rustc
            rustfmt
            rust-analyzer
            cargo-tauri
            nodejs
            pnpm
          ];

          buildInputs = libraries;

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          shellHook = ''
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}
