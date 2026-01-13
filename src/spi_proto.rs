


use crc::{Crc, CRC_16_IBM_SDLC};
use embedded_hal::spi::{Operation as SpiOperation, SpiDevice};
use defmt::{info, Format};

pub const PROTOCOL_OVERHEAD : usize = 5; // 1 byte reg, 2 bytes len, 2 bytes CRC
pub const PROTOCOL_DATA_OFFSET: usize = 3;
pub const PROTOCOL_CRC_SIZE: usize = 2;
pub const REGISTERS_COUNT: usize = 40;
pub const REGISTERS_SIZE_BYTES: usize = REGISTERS_COUNT * 4;
pub const REGISTERS_PROTO_SIZE: usize = REGISTERS_SIZE_BYTES + PROTOCOL_OVERHEAD;
pub const DEVICE_SUPPORTED_MOTORS: usize = 4;
pub const DEVICE_MOTOR_BLOCK_COUNT: usize = 9; // Number of registers per motor
pub const DEVICE_REG_SIZE_BYTES: usize = 4; // Each register is 4 bytes
pub const DEVICE_MOTOR_BLOCK_SIZE_BYTES: usize = DEVICE_MOTOR_BLOCK_COUNT * DEVICE_REG_SIZE_BYTES; // Size of each motor block in bytes
pub const DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES: usize = DEVICE_SUPPORTED_MOTORS * DEVICE_MOTOR_BLOCK_SIZE_BYTES;

pub const PROTOCOL_MAX_BUFFER_SIZE: usize = PROTOCOL_OVERHEAD + REGISTERS_SIZE_BYTES + PROTOCOL_CRC_SIZE;
pub const PROTOCOL_MAX_WRITE_BUFF_SIZE: usize = PROTOCOL_CRC_SIZE + REGISTERS_SIZE_BYTES; 
pub const DEVICE_ID_REG_SIZE: usize = 4;
pub const FIRMWARE_VERSION_REG_SIZE: usize = 4;
pub const SYSTEM_INFO_SIZE: usize = DEVICE_ID_REG_SIZE + FIRMWARE_VERSION_REG_SIZE;
pub const MOTORS_DATA_START_OFFSET: usize = SYSTEM_INFO_SIZE;
pub const INTERNAL_LOOP_TIME_OFFSET: usize = MOTORS_DATA_START_OFFSET + DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES;
#[repr(u8)]
#[derive(Copy, Clone, Debug, Format, PartialEq, Eq)]
pub enum SpiPackOpType {
    Read = 0u8,
    Write = 1u8,
}



impl TryFrom<u8> for SpiPackOpType {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SpiPackOpType::Read),
            1 => Ok(SpiPackOpType::Write),
            other => Err(other),
        }
    }
}


pub fn populate_crc(buff: &[u8], crc: &mut[u8])
{
    assert!(crc.len() == PROTOCOL_CRC_SIZE,
            "CRC buffer size should be exaclty 2 received {}",
            crc.len());
    let crc_algo = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let crc_val = crc_algo.checksum(&buff[0..buff.len()]);
    crc.copy_from_slice(&crc_val.to_le_bytes());
}

pub fn validate_crc(buff: &[u8], recv_crc: u16) -> bool {
    let crc_algo = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let crc = crc_algo.checksum(&buff[0..buff.len()]);
    crc == recv_crc
}

pub fn process_spi_header(packet: &[u8]) -> (u8, SpiPackOpType, u16) {
    assert!(packet.len() == PROTOCOL_OVERHEAD,
            "Packet size should be at least PROTOCOL_OVERHEAD received {}",
            packet.len());
    let reg_rw = packet[0];
    let crc_offset = PROTOCOL_OVERHEAD - PROTOCOL_CRC_SIZE;
    let reg = reg_rw & 0x7F;
    let rw_len = u16::from_le_bytes([packet[1], packet[2]]);
    let rw_type = SpiPackOpType::try_from((reg_rw >> 7) & 0x01).unwrap();
    
    let recv_crc = u16::from_le_bytes([packet[crc_offset], packet[crc_offset + 1]]);
    let is_valid = validate_crc(&packet[0..crc_offset], recv_crc);
    assert!(is_valid, "CRC validation failed for received packet");
    (reg, rw_type, rw_len)
}

pub fn process_spi_packet(packet: &[u8]) -> &[u8] {
    let data = &packet[0..packet.len() - PROTOCOL_CRC_SIZE];
    let recv_crc = u16::from_le_bytes([packet[packet.len() - 2], packet[packet.len() - 1]]);
    let is_valid = validate_crc(data, recv_crc);
    assert!(is_valid, "CRC validation failed for received packet");
    data
}
