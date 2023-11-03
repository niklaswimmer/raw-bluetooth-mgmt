#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SocketAddressHci {
    hci_family: libc::sa_family_t,
    hci_dev: u16,
    hci_channel: u16,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SocketAddressGeneric(libc::sockaddr);

impl SocketAddressGeneric {
    pub fn as_raw(&self) -> *const libc::sockaddr {
        &**self
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SocketAddressContainer<T: Copy> {
    sa_family: libc::sa_family_t,
    sa_data: T,
}

#[repr(C)]
pub union SocketAddress<T: Copy> {
    generic: SocketAddressGeneric,
    container: SocketAddressContainer<T>,
}

impl<T: Copy> SocketAddress<T> {
    pub fn new(sa_family: rustix::net::AddressFamily, sa_data: T) -> Self {
        // make sure we cannot read uninitialized data by accident by zeroing all
        let mut socket_address: SocketAddress<T> = unsafe { std::mem::zeroed() };
        socket_address.container = SocketAddressContainer {
            sa_family: sa_family.as_raw(),
            sa_data,
        };
        socket_address
    }

    pub fn len(&self) -> usize {
        std::mem::size_of_val(unsafe { &self.container })
    }

    pub fn generic(&self) -> &SocketAddressGeneric {
        unsafe { &self.generic }
    }
}

impl core::fmt::Debug for SocketAddressGeneric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketAddressGeneric")
            .field("sa_family", &self.0.sa_family)
            .field(
                "sa_data",
                &self
                    .0
                    .sa_data
                    .iter()
                    // because we are interested in raw bytes, u8 makes more sense than i8
                    .map(|v| *v as u8)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<T: core::fmt::Debug + Copy> core::fmt::Debug for SocketAddress<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketAddressUnion")
            .field("container", unsafe { &self.container })
            .field("generic", unsafe { &self.generic })
            .finish()
    }
}

impl std::ops::Deref for SocketAddressGeneric {
    type Target = libc::sockaddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
