let
  mozillaOverlay = import (builtins.fetchTarball "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  pkgs = import <nixpkgs> {
    overlays = [ mozillaOverlay ];
  };
  rustChannel = (pkgs.rustChannelOf { rustToolchain = ./rust-toolchain; });
in pkgs.mkShell {
  nativeBuildInputs = [
    (rustChannel.rust.override (old:
      { extensions = ["rust-src" "rust-analysis"]; }))
    pkgs.pkg-config
    pkgs.openssl
    pkgs.fuse3
    pkgs.protobuf
    pkgs.pv
  ];
}
