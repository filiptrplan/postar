mod cli {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/args.rs"));
}

fn main() -> std::io::Result<()> {
    let out_dir_main =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);

    let out_dir_man = out_dir_main.join("man");
    std::fs::create_dir_all(&out_dir_man)?;
    let cmd = <cli::Args as clap::CommandFactory>::command();
    clap_mangen::generate_to(cmd, &out_dir_man)?;

    let out_dir_comp = out_dir_main.join("comp");
    std::fs::create_dir_all(&out_dir_comp)?;

    let mut cmd2 = <cli::Args as clap::CommandFactory>::command();
    clap_complete::generate_to(
        clap_complete::Shell::Zsh,
        &mut cmd2,
        "postar",
        &out_dir_comp,
    )?;
    clap_complete::generate_to(
        clap_complete::Shell::Fish,
        &mut cmd2,
        "postar",
        &out_dir_comp,
    )?;
    clap_complete::generate_to(
        clap_complete::Shell::Bash,
        &mut cmd2,
        "postar",
        &out_dir_comp,
    )?;

    Ok(())
}
