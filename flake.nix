{
  description = "SSSS";
  inputs.nixpkgs.url = "github:loewenheim/nixpkgs/89816";

  outputs = { self, nixpkgs }:
  with import nixpkgs { system = "x86_64-linux"; };
  {
    packages.x86_64-linux.transpastex = nixpkgs.buildRustCrate {
      crateName = "client";
      src = ./client;
    };

    defaultPackages.x86_64-linux = self.packages.x86_64-linux.transpastex;

    devShell.x86_64-linux = pkgs.mkShell {
      name = "trans-pastex";
      buildInputs = with pkgs; [
        alsaLib
        glslang
        vulkan-loader
        x11 xorg.libxcb
        httplz
        wasm-bindgen-cli udev wayland
        gnumake vulkan-validation-layers
      ];
      nativeBuildInputs = [ pkgs.pkg-config pkgs.renderdoc ];
      LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib:${pkgs.libxkbcommon}/lib:${pkgs.wayland}/lib:${pkgs.xorg.libxcb}/lib";
      RUST_BACKTRACE = 1;
      VK_ICD_FILENAMES="/run/opengl-driver/share/vulkan/icd.d/intel_icd.x86_64.json";
    };
  };
}
