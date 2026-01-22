fn main() {
    let res = postar::cli::run();
    if let Err(err) = res {
        log::error!("{:?}", err);
        std::process::exit(1);
    }
}
