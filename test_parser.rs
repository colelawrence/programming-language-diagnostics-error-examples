use editor_core::parser::parse_command;

fn main() {
    let input = "ffmpeg -i input.mp4 output.mp4";
    match parse_command(input) {
        Ok(cmd) => {
            println!("Parse successful!");
            println!("Inputs: {}", cmd.inputs.len());
            for (i, input) in cmd.inputs.iter().enumerate() {
                println!("  Input {}: {} ({} options)", i, input.file_path, input.options.len());
            }
            println!("Outputs: {}", cmd.outputs.len());
            for (i, output) in cmd.outputs.iter().enumerate() {
                println!("  Output {}: {} ({} options)", i, output.file_path, output.options.len());
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}

