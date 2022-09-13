{
  description = "Rust GraphQL Exercise";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in
      with pkgs;
        {
          devShells.default = mkShell {
            # Unfortunately retrieving Rust through Flake didn't work with the intellij plugin. Only with VScdoe
            nativeBuildInputs = [ 
              terraform
              bunyan-rs
            ];  
          };
        }
      );
}

