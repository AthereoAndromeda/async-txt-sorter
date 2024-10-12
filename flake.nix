{
  description = "Rust dev flake w/ fenix";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system: 
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fenix-pkgs = fenix.packages.${system};
        toolchain = fenix-pkgs.stable.toolchain;
      in {
          devShell = pkgs.mkShell {
          
            nativeBuildInputs = with pkgs; [
              pkg-config # Enable to allow locating other external deps
            ] ++ [ 
              toolchain 
              # fenix-pkgs.rust-analyzer
            ];
        
            # buildInputs = with pkgs; [
            #   # systemd # Enable if udev is required
            # ];

            # shellHook = ''
            #   export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath (with pkgs; [
            #     # systemd
            #   ])}"
            # '';

       };
    });
}
