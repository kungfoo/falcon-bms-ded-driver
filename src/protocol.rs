pub const ICP_USB_VENDOR_ID: u16 = 0x4098;
pub const ICP_USB_PRODUCT_ID: u16 = 0xbf06;
pub const ICP_HID_PAYLOAD_LENGTH: usize = 60;

pub const CMD_WRITE_DISPLAY_MEM: u32 = 0x102;
pub const CMD_REFRESH_DISPLAY: u32 = 0x103;

#[derive(Debug)]
pub struct IcpPacket {
    pub op_type: u8,
    pub sequence_num: u8,
    pub payload: Vec<u8>,
}

impl IcpPacket {
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        buf[0] = 0xf0; // HID Report ID
        buf[1] = self.op_type;
        buf[2] = self.sequence_num;
        buf[3] = self.payload.len() as u8;
        let len = self.payload.len().min(60);
        buf[4..4 + len].copy_from_slice(&self.payload[..len]);
        buf
    }
}

#[derive(Debug)]
pub struct DedCommand {
    pub command_type: u32,
    pub timestamp: u32,
    pub data: Vec<u8>,
}

impl DedCommand {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(17 + self.data.len());
        buf.extend_from_slice(&(ICP_USB_PRODUCT_ID as u32).to_le_bytes());
        buf.extend_from_slice(&self.command_type.to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.push(0);
        buf.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icp_packet_serialization() {
        let packet = IcpPacket {
            op_type: 0x02,
            sequence_num: 0x01,
            payload: vec![0x00],
        };
        let bytes = packet.to_bytes();
        assert_eq!(bytes[0], 0xf0);
        assert_eq!(bytes[1], 0x02);
        assert_eq!(bytes[2], 0x01);
        assert_eq!(bytes[3], 0x01);
        assert_eq!(bytes[4], 0x00);
    }

    #[test]
    fn test_ded_command_serialization() {
        let cmd = DedCommand {
            command_type: 0x102,
            timestamp: 0xffff,
            data: vec![0xaa, 0xbb],
        };
        let bytes = cmd.to_bytes();
        assert_eq!(bytes.len(), 17 + 2);
        assert_eq!(&bytes[0..4], &(0xbf06u32).to_le_bytes());
        assert_eq!(&bytes[4..8], &(0x102u32).to_le_bytes());
        assert_eq!(&bytes[8..12], &(0xffffu32).to_le_bytes());
        assert_eq!(bytes[12], 0);
        assert_eq!(&bytes[13..17], &(2u32).to_le_bytes());
        assert_eq!(bytes[17], 0xaa);
        assert_eq!(bytes[18], 0xbb);
    }
}
