extern crate ntcore;

use ntcore::Instance;

fn delay() {
    ::std::thread::sleep(::std::time::Duration::new(1, 0));
}

fn main() { 
    println!("Starting Client!");
    let client = Instance::start_client_multi(vec![("127.0.0.1", 1735)]);

    delay();
    println!("--- CLIENT CONNECTIONS ---");
    for connection in client.get_connections().into_iter() {
        println!("ID   {}", connection.remote_id());
        println!("IP   {}", connection.remote_ip_str());
        println!("PORT {}", connection.remote_port());
    }

    println!("--- ENTRIES ---");
    for entry in client.get_all_entries() {
        println!("{} {:?}", entry.name().unwrap(), entry.value());
    }
    println!("--- ------- ---");

    let mut table = client.get_table("/foo/bar".into());

    table.set("baz", 0.0).unwrap();

    loop {
        delay();
        table
            .get("baz")
            .edit(|val| val.map_double(|val| val + 1.0));
    }

    println!("Done!");
}
