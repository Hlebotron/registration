{
    description = "Registration";
    inputs = {
        nixpkgs.url = "nixpkgs/nixos-24.11";
    };
    outputs = { self, nixpkgs, ... }@inputs: 
    let
        system = "x86_64-linux";     
        pkgs = import nixpkgs { 
            inherit system; 
            rustPlatform = nixpkgs.makeRustPlatform { 
                cargo = "stable-x86_64-unknown-gnu-linux"; 
                rust = "stable-x86_64-unknown-gnu-linux"; 
            };
        };
    in {
        packages.${system} =  {
            registration = pkgs.rustPlatform.buildRustPackage {
                name = "registration";
                inherit system;
                src = ./.;
                cargoLock.lockFile = ./Cargo.lock;
            };
            default = self.packages.${system}.registration;
        };
    };
}
