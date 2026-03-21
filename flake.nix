{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        # cargo-expand は nightly の機能を使うため、
        # ツールチェーン自体に nightly を含めるか、拡張として追加します
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "llvm-tools-preview" ];
          # ここで nightly を直接使うか、
          # あるいは pkgs.cargo-expand を別途 inputs に入れます
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.cargo-expand  # Nixpkgs から導入
            pkgs.rustfmt       # 展開後の整形に必須
            pkgs.cargo-pgo
          ];
        };
      }
    );
}
