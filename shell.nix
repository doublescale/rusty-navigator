with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  nativeBuildInputs = [
    # rustc cargo
    rustup
  ];
  buildInputs = [
    SDL2 SDL2_image
  ];
}
