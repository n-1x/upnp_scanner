use std::collections::HashMap;
use std::net::UdpSocket;
use std::str;
use std::time::Duration;

struct RootDeviceInfo {
    location: String,
    server: String,
    usn: String,
}

const MX: u8 = 3;

fn parse_headers(lines: std::str::Lines) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in lines {
        let line_lowercase = line.to_string().to_ascii_lowercase();

        let s: Vec<&str> = line_lowercase.splitn(2, ":").collect();

        if s.len() == 2 {
            map.insert(s[0].to_string(), s[1].to_string());
        }
    }

    map
}

fn parse_response(buf: &[u8; 1024]) -> Result<RootDeviceInfo, &str> {
    let s = str::from_utf8(buf).unwrap_or("").to_string();
    let mut lines = s.lines();
    let status_line = lines.next().unwrap();
    let v: Vec<&str> = status_line.split(" ").collect();
    let mut iter = v.iter();

    iter.next(); //skip http version
    let status_code = match iter.next() {
        Some(s) => s,
        None => "",
    };

    let headers = parse_headers(lines);

    let location = (match headers.get("location") {
        Some(s) => s,
        None => "No location found",
    })
    .to_string();

    let usn = (match headers.get("usn") {
        Some(s) => s,
        None => "No USN found",
    })
    .to_string();

    let server = (match headers.get("server") {
        Some(s) => s,
        None => "No USN found",
    })
    .to_string();

    match status_code.as_ref() {
        "200" => Ok(RootDeviceInfo {
            location,
            usn,
            server,
        }),
        _ => Err("Failed"),
    }
}

fn main() {
    {
        let socket = UdpSocket::bind("0.0.0.0:42425").expect("Failed to bind laddr");
        socket
            .set_read_timeout(Some(Duration::new(MX as u64, 0)))
            .expect("Failed to set read timeout");
        let raddr_string = "239.255.255.250:1900";
        let msg = format!(
            concat!(
                "M-SEARCH * HTTP/1.1\r\n",
                "HOST:{}\r\n",
                "MAN:\"ssdp:discover\"\r\n",
                "MX:{}\r\n",
                "ST:upnp:rootdevice\r\n\r\n"
            ),
            raddr_string, MX
        );
        let mut devices = HashMap::new();

        println!("Starting UPNP root device search");
        loop {
            socket
                .send_to(msg.as_bytes(), raddr_string)
                .expect("Failed to send message");
            loop {
                let mut buf = [0; 1024];
                let response = match socket.recv_from(&mut buf) {
                    Ok(_) => parse_response(&buf),
                    Err(_) => {
                        break;
                    }
                };
                if response.is_err() {
                    println!("Failed to parse response");
                } else {
                    let r = response.unwrap();
                    //lookup device in devices hashmap, add if not found
                    if devices.get(&r.usn).is_none() {
                        println!("{}\n\t{}", &r.server, &r.location);
                        devices.insert(r.usn.clone(), r);
                    }
                }
                //println!("{}", std::str::from_utf8(&buf).unwrap_or(""));
            }
        }
    }
}
