use crate::protocol::{
    CMD_REFRESH_DISPLAY, CMD_WRITE_DISPLAY_MEM, DedCommand, ICP_HID_PAYLOAD_LENGTH, IcpPacket,
};
use anyhow::{Result, anyhow};
use hidapi::{HidApi, HidDevice};

pub struct DedDriver {
    device: HidDevice,
    seq: u8,
}

impl DedDriver {
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self> {
        let api = HidApi::new()?;
        let device = api
            .open(vendor_id, product_id)
            .map_err(|e| anyhow!("Failed to open HID device: {}", e))?;

        let mut driver = Self { device, seq: 1 };

        // Initialize device
        driver.write_packet(0x02, &[0])?;

        Ok(driver)
    }

    fn write_packet(&mut self, op_type: u8, payload: &[u8]) -> Result<()> {
        let packet = IcpPacket {
            op_type,
            sequence_num: self.seq,
            payload: payload.to_vec(),
        };
        self.seq = self.seq.wrapping_add(1);
        self.device.write(&packet.to_bytes())?;
        Ok(())
    }

    pub fn send_display_data(&mut self, data: &[u8]) -> Result<()> {
        let write_mem = DedCommand {
            command_type: CMD_WRITE_DISPLAY_MEM,
            timestamp: 0xffff,
            data: data.to_vec(),
        };
        self.send_command(write_mem)?;

        let refresh = DedCommand {
            command_type: CMD_REFRESH_DISPLAY,
            timestamp: 0xffff,
            data: vec![0],
        };
        self.send_command(refresh)?;

        Ok(())
    }

    fn send_command(&mut self, cmd: DedCommand) -> Result<()> {
        let bytes = cmd.to_bytes();
        for chunk in bytes.chunks(ICP_HID_PAYLOAD_LENGTH) {
            self.write_packet(0, chunk)?;
        }
        Ok(())
    }
}
