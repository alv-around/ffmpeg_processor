{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
    rust-overlay,
  }: let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
      overlays = [(import rust-overlay)];
    };
    rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    naerskLib = pkgs.callPackage naersk {};
  in {
    packages."x86_64-linux".default =
      (naerskLib.override {
        cargo = rustToolchain;
        rustc = rustToolchain;
      }).buildPackage {
        src = ./.;
      };

    devShells."x86_64-linux".default = pkgs.mkShell {
      buildInputs = with pkgs; [
        rustToolchain

        # add additional required libraries for project. ex: glib
      ];

      nativeBuildInputs = [pkgs.pkg-config];

      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
