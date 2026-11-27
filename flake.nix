{
  # The tools for this repository: `nix develop`, or direnv (`.envrc`).
  description = "apex: the personal site (Rust: native, browser, SSH; Typst)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, ... }@inputs:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              # the Rust toolchain with the wasm32 target (nixpkgs rustc has no wasm std)
              overlays = [ inputs.rust-overlay.overlays.default ];
            }
          )
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShellNoCC {
          packages = [
            (pkgs.rust-bin.stable.latest.default.override {
              targets = [ "wasm32-unknown-unknown" ];
            })
            pkgs.trunk
            # trunk takes the wasm-bindgen from PATH when its version matches Cargo.lock
            pkgs.wasm-bindgen-cli_0_2_130
            pkgs.typst
            pkgs.uv
            # `just unicode`, `just polyring`
            (pkgs.python3.withPackages (ps: [ ps.fonttools ]))
            pkgs.just
          ];
        };
      });
      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
