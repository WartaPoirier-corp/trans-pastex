{
  description = "SSSS";
  inputs.nixpkgs.url = "github:loewenheim/nixpkgs/89816";

  outputs = { self, nixpkgs }:
  with import nixpkgs { system = "x86_64-linux"; };
  {
    packages.x86_64-linux.serpent = nixpkgs.buildRustCrate {
      crateName = "serpent";
      src = ./.;
    };

    defaultPackages.x86_64-linux = self.packages.x86_64-linux.serpent;

    devShell.x86_64-linux = pkgs.mkShell {
      name = "serpent";
      buildInputs = with pkgs; [
        alsaLib
        glslang
        vulkan-loader
        x11
        httplz
        wasm-bindgen-cli udev wayland
        gnumake vulkan-validation-layers
      ];
      nativeBuildInputs = [ pkgs.pkg-config pkgs.renderdoc ];
      LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib:${pkgs.libxkbcommon}/lib:${pkgs.wayland}/lib";
      RUST_BACKTRACE = 1;
      VK_ICD_FILENAMES="/run/opengl-driver/share/vulkan/icd.d/intel_icd.x86_64.json"
    };
  };
}
