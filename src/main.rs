use rustix::{
    fd::AsRawFd,
    net::{AddressFamily, Protocol, SocketFlags, SocketType},
};

mod address;
mod hci;

use address::SocketAddress;
use hci::*;

fn main() -> anyhow::Result<()> {
    // open a bluetooth socket for hci communication
    // see `mgmt_new_default` in BlueZ's `mgmt.c`
    let fd = rustix::net::socket_with(
        AddressFamily::BLUETOOTH,
        SocketType::RAW,
        SocketFlags::CLOEXEC, // TODO: SocketFlags::NONBLOCK
        Some(Protocol::from_raw(1.try_into().expect("3 is non-zero"))),
    )?;

    // abritrary socket addresses are not support by rustix, so we need to
    // use libc's bind function which needs a pointer to a sockaddr
    // sockaddr is a generic address with a 14 byte wide store for abritrary data
    // the easiest way to fill that store with the correct values is with a union
    // the address module abstracts all that stuff, ideally we could have support
    // in rustix
    let address = SocketAddress::new(
        AddressFamily::BLUETOOTH,
        HciAddressData {
            // means no specific device, which I guess means "choose whichever you like"
            hci_dev: 0xffff,
            // channel 3 is the control channel (as per `hci_sock.h` in linux kernel source)
            hci_channel: 3,
        },
    );

    let len = address.len().try_into()?;

    if unsafe { libc::bind(fd.as_raw_fd(), address.generic().as_raw(), len) } < 0 {
        Err(std::io::Error::last_os_error())?;
    }

    let request = HciMessage {
        header: HciMessageHeader {
            op_code: 0x0001,
            index: 0xffff,
            len: 0,
        },
        data: Vec::with_capacity(0),
    };

    // write the version request and read the response from the now bounded socket
    rustix::io::write(&fd, &request.as_bytes())?;

    let mut buffer: [u8; 512] = [0; 512];
    rustix::io::read(&fd, &mut buffer)?;

    let response = HciMessage::from_bytes(&buffer);

    let opcode = response.data[0];
    let status = u16::from_le_bytes([response.data[1], response.data[2]]);
    let version = response.data[3];
    let revision = u16::from_le_bytes([response.data[4], response.data[5]]);

    println!("got response from hci socket with opcode={opcode} and status={status}; management interface version is {}.{}", version, revision);

    // it works :)
    Ok(())
}
