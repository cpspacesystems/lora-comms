
mod publisher;
mod subscriber;
fn main() {
    let p = publisher::Pubs::new("test".to_string());
    p.send_str("Hello World");
    let s = subscriber::Subs::new("test".to_string());
    s.get();
}


// functions that make an interface
// tells things to subscribe to
// let us do callbacks
// if something happens, callbacks happens
// configurations