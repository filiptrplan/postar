use once_cell::sync::Lazy;
use testcontainers::{
    GenericImage,
    core::{IntoContainerPort, WaitFor},
    runners::SyncRunner,
};

struct IMAPContainerData {
    host: String,
    port: u16,
}

static IMAP_CONTAINER_DATA: Lazy<IMAPContainerData> = Lazy::new(|| {
    let container = GenericImage::new("greenmail/standalone", "2.0.1")
        .with_exposed_port(3143.tcp())
        .with_wait_for(WaitFor::healthcheck())
        .start()
        .unwrap();

    IMAPContainerData {
        host: container.get_host().unwrap().to_string(),
        port: 3143,
    }
});

#[test]
fn mytest() {
    println!("{}", IMAP_CONTAINER_DATA.host);
    assert_eq!(true, true);
}
