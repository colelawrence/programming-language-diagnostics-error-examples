pub mod pathfinder;
pub mod router;
pub mod utils;

#[cfg(test)]
#[cfg(feature = "codegen")]
mod generate {
    use std::{path::PathBuf, process::Command};
    #[test]
    #[ignore]
    fn generate_typescript() {
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap()
            .parse::<PathBuf>()
            .unwrap();

        let mut typescript_generation = derive_codegen::Generation::for_tag("protocol-cg");
        typescript_generation.include_tag("protocol-router");

        let mut typescript_command = Command::new("bun");
        typescript_command
            .arg("./generateTypescript.ts")
            .current_dir(cargo_dir.join("generators"));

        typescript_generation
            .pipe_into(&mut typescript_command)
            .with_output_path(cargo_dir.join("../../packages/cg-types"))
            .write();

        let mut rust_command = Command::new("bun");
        rust_command
            .arg("./generateRustRouter.ts")
            .current_dir(cargo_dir.join("generators"));

        derive_codegen::Generation::for_tag("protocol-cg")
            .pipe_into(&mut rust_command)
            // crates/cg-types/src/router/router_goal.rs
            .with_output_path(cargo_dir.join("src/router"))
            .write();
    }
}
