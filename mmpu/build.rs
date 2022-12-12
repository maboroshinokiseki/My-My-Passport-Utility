use clap::{Command, CommandFactory};
use clap_complete::{generate, Shell};
use std::{
    env,
    fs::{self, OpenOptions},
    io::BufWriter,
    path::Path,
};
#[path = "src/args.rs"]
mod args;

fn generate_complete(cmd: &mut Command, out_dir: &Path, filename: &str, shell: Shell) {
    let file_path = out_dir.join(filename);
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)
        .unwrap();
    let mut writter = BufWriter::new(file);

    generate(shell, cmd, cmd.get_name().to_owned(), &mut writter);
}

fn main() {
    let mut cmd = args::Cli::command();

    let out_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = Path::new(&out_dir).join("complete");
    fs::create_dir_all(&out_dir).unwrap();

    generate_complete(&mut cmd, &out_dir, "mmpu.bash", clap_complete::Shell::Bash);
    generate_complete(&mut cmd, &out_dir, "mmpu.zsh", clap_complete::Shell::Zsh);
    generate_complete(&mut cmd, &out_dir, "mmpu.fish", clap_complete::Shell::Fish);
}
