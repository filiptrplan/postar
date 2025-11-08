use std::{env::current_dir, thread, time};

use testcontainers::{
    Container, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor},
    runners::SyncRunner,
};

use crate::inbox::Inbox;

struct IMAPContainerData {
    host: String,
    port: u16,
    container: Container<GenericImage>,
}

fn get_container() -> IMAPContainerData {
    let port = 3993;
    let container = GenericImage::new("greenmail/standalone", "2.1.7")
        .with_exposed_port(port.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Starting GreenMail"))
        .with_env_var(
            "GREENMAIL_OPTS",
            "-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.auth.disabled -Dgreenmail.preload.dir=/tmp/preload -Dgreenmail.verbose",
        )
        .with_mount(Mount::bind_mount(current_dir().unwrap().to_str().unwrap().to_owned() + "/mock_emails", "/tmp/preload"))
        .start()
        .unwrap();

    IMAPContainerData {
        host: container.get_host().unwrap().to_string(),
        port: container.get_host_port_ipv4(port).unwrap(),
        container,
    }
}

#[test]
fn mytest() -> anyhow::Result<()> {
    let container_data = get_container();
    let inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    Ok(())
}
