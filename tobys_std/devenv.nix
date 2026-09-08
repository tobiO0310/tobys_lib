{ pkgs, lib, ... }:

{
    languages.rust = {
        enable = true;
        channel = "nightly";
        components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
    };
    packages = with pkgs; [
        bacon
        cargo-seek
        cargo-nextest
        cargo-binstall
        cargo-release
        cargo-msrv
        cargo-hack
    ];
    scripts.watcher = {
        exec = ''
            watchexec -c -e rs \
            "cargo clippy && cargo test && cargo run"
        '';
        packages = [ pkgs.watchexec ];
    };
    env.LD_LIBRARY_PATH = lib.makeLibraryPath [
        pkgs.zlib
    ];
    env = {
        DATABASE_URL = "postgres://user:pass@localhost/dbname";
    };
    enterShell = ''
        echo "Crates ready to update with 'cargo update'":
        cargo update -n
    '';
    git-hooks.hooks = {
        rustfmt.enable = true;
        clippy.enable = true;
    };
}
