use std::net::{Ipv4Addr};
use clap::clap_app;

pub struct ServerArgs {
    pub port: u16,
    pub host: Ipv4Addr
}

impl ServerArgs {
    pub fn new(host: Ipv4Addr, port: u16) -> ServerArgs {
        ServerArgs {
            port,
            host
        }
    }

    pub fn parse_program_args() -> ServerArgs {
        let matches = clap_app!(myapp =>
            (version: "0.1.0")
            (about: "Project Gamma Server")
            (@arg port: -p --port +takes_value "Sets the server port")
            (@arg host: -h --host +takes_value "Sets the server host")
        ).get_matches();

        let server_port = matches.value_of("port").unwrap_or("9001").parse().expect("Invalid port");
        let server_ip = matches.value_of("host")
            .map(|ip| ip.parse().expect("Invalid host"))
            .unwrap_or(Ipv4Addr::UNSPECIFIED);

        return ServerArgs::new(server_ip, server_port)
    }
}