use sstui::userdata::ServerGroup;

fn main() {
    let mut server_group = ServerGroup::new(
        "name",
        "https://fbapiv2.fbsublink.com/flydsubal/ar4erevrghsl6md4?sub=2&extend=1",
    );
    server_group.update().unwrap();
    println!(
        "{:?} {}",
        server_group.update_type,
        server_group.ss_servers.len()
    );
    let mut server_group = ServerGroup::new(
        "name",
        "https://fbapiv2.fbsublink.com/flydsubal/ar4erevrghsl6md4?list=ssa&extend=1",
    );
    server_group.update().unwrap();
    println!(
        "{:?} {}",
        server_group.update_type,
        server_group.ss_servers.len()
    );
}
