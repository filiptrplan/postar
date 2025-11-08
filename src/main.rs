use std::env;

use postar::inbox::Inbox;

fn main() {
    let domain = env::var("DOMAIN").unwrap();
    let user = env::var("USER").unwrap();
    let pass = env::var("PASS").unwrap();
    let port = env::var("PORT").unwrap().parse::<u16>().unwrap();

    let mut inbox = Inbox::new_tls(&domain, port, &user, &pass).unwrap();

    println!("{:?}", inbox.list_folders());

    // we want to fetch the first email in the INBOX mailbox
    // imap_session.select("INBOX").unwrap();
    //
    // // fetch message number 1 in this mailbox, along with its RFC822 field.
    // // RFC 822 dictates the format of the body of e-mails
    // let messages = imap_session.fetch("1", "RFC822").unwrap();
    // let message = if let Some(m) = messages.iter().next() {
    //     m
    // } else {
    //     return;
    // };
    //
    // // extract the message's body
    // let body = message.body().expect("message did not have a body!");
    // let body = std::str::from_utf8(body)
    //     .expect("message was not valid utf-8")
    //     .to_string();
    //
    // // be nice to the server and log out
    // imap_session.logout().unwrap();
    //
    // let email = MessageParser::default().parse(&body).unwrap();
    //
    // println!("{:?}", email.body_text(0));
}
