#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HciAddressData {
    pub hci_dev: u16,
    pub hci_channel: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct HciMessage {
    pub header: HciMessageHeader,
    pub data: Vec<u8>,
}

impl HciMessage {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(<u16 as Into<usize>>::into(self.header.len) + 6);
        data.extend(self.header.op_code.to_le_bytes());
        data.extend(self.header.index.to_le_bytes());
        data.extend(self.header.len.to_le_bytes());
        data.extend(&self.data);
        data
    }

    pub fn from_bytes(buffer: &[u8]) -> Self {
        let op_code = u16::from_ne_bytes([buffer[0], buffer[1]]);
        let index = u16::from_ne_bytes([buffer[2], buffer[3]]);
        let len = u16::from_ne_bytes([buffer[4], buffer[5]]);
        let mut data = Vec::with_capacity(len.into());
        data.extend_from_slice(&buffer[6..][..len.into()]);
        HciMessage {
            header: HciMessageHeader {
                op_code,
                index,
                len,
            },
            data,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct HciMessageHeader {
    pub op_code: u16,
    pub index: u16,
    pub len: u16,
}
