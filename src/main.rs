use std::io::{Read, Write};

fn main() {
	let last_arg = std::env::args().last().unwrap_or_default();
	if last_arg == "--help" {
		eprintln!(
			"l3p3 metrofi v{}\n\
			 usage: metrofi [--show | --logout]",
			env!("CARGO_PKG_VERSION")
		);
		return;
	}

	let socket_addr = std::net::SocketAddr::from(([10, 101, 8, 84], 80));
	let mut stream = std::net::TcpStream::connect_timeout(
		&socket_addr,
		std::time::Duration::from_secs(5)
	).expect("cannot connect");

	// get current status
	let get_request = "GET /de/ HTTP/1.1\r\n\
	                   Host: wifi.metronom.de\r\n\
				       Connection: close\r\n\r\n";
	stream.write_all(get_request.as_bytes()).expect("failed to send request");
	let mut response = String::new();
	stream.read_to_string(&mut response).expect("failed to read response");

	// parse
	let token = {
		let index = response.find("name=\"CSRFToken\"").expect("CSRFToken not found");
		response.get((index + 24)..(index + 24 + 32)).unwrap()
	};
	let logged_in = response.contains("name=\"logout\" value=\"true\"");
	eprintln!(
		"token:     {}\n\
		 logged in: {}\n",
		token,
		logged_in
	);

	if last_arg == "--show" {
		return;
	}
	let should_logout = last_arg == "--logout";

	if logged_in != should_logout {
		eprintln!("nothing to do.");
		return;
	}
	let should_str = if should_logout { "out" } else { "in" };

	// log in/out
	let post_request = format!(
		"POST /de/ HTTP/1.1\r\n\
		 Host: wifi.metronom.de\r\n\
		 Connection: close\r\n\
		 Content-Type: application/x-www-form-urlencoded\r\n\
		 Cookie: csrf={}\r\n\
		 Content-Length: {}\r\n\
		 \r\n\
		 log{}=true&CSRFToken={}&",
		token,
		3 + 3 + 16 + 32, // "log" + "out" + "=true&CSRFToken=" + token; HACK: additional & ignored
		should_str,
		token
	);
	let mut stream = std::net::TcpStream::connect(socket_addr).expect("cannot connect");
	stream.write_all(post_request.as_bytes()).expect("failed to send request");
	let mut response = String::new();
	stream.read_to_string(&mut response).expect("failed to read response");

	eprintln!("logged {}.", should_str);
}
