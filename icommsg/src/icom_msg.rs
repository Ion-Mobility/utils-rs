use std::{error::Error};

const ICOM_MSG_PAYLOAD_MAX_LEN: usize = 256;
const ICOM_MSG_MAX_LEN: usize = 259;
const ICOM_FN_MAX_LEN: usize = 128;

#[derive(Debug, Clone)]
pub struct IONICOMPacketType {
    PayloadLen: u16, // Changed to u16
    Payload: [u8; ICOM_MSG_PAYLOAD_MAX_LEN],
    Crc: u8,
}

const CRC_TABLE: [u8; 256] = [
    0x0,  0x7,  0xE,  0x9,  0x1C, 0x1B, 0x12, 0x15, 0x38, 0x3F, 0x36, 0x31,
    0x24, 0x23, 0x2A, 0x2D, 0x70, 0x77, 0x7E, 0x79, 0x6C, 0x6B, 0x62, 0x65,
    0x48, 0x4F, 0x46, 0x41, 0x54, 0x53, 0x5A, 0x5D, 0xE0, 0xE7, 0xEE, 0xE9,
    0xFC, 0xFB, 0xF2, 0xF5, 0xD8, 0xDF, 0xD6, 0xD1, 0xC4, 0xC3, 0xCA, 0xCD,
    0x90, 0x97, 0x9E, 0x99, 0x8C, 0x8B, 0x82, 0x85, 0xA8, 0xAF, 0xA6, 0xA1,
    0xB4, 0xB3, 0xBA, 0xBD, 0xC7, 0xC0, 0xC9, 0xCE, 0xDB, 0xDC, 0xD5, 0xD2,
    0xFF, 0xF8, 0xF1, 0xF6, 0xE3, 0xE4, 0xED, 0xEA, 0xB7, 0xB0, 0xB9, 0xBE,
    0xAB, 0xAC, 0xA5, 0xA2, 0x8F, 0x88, 0x81, 0x86, 0x93, 0x94, 0x9D, 0x9A,
    0x27, 0x20, 0x29, 0x2E, 0x3B, 0x3C, 0x35, 0x32, 0x1F, 0x18, 0x11, 0x16,
    0x3,  0x4,  0xD,  0xA,  0x57, 0x50, 0x59, 0x5E, 0x4B, 0x4C, 0x45, 0x42,
    0x6F, 0x68, 0x61, 0x66, 0x73, 0x74, 0x7D, 0x7A, 0x89, 0x8E, 0x87, 0x80,
    0x95, 0x92, 0x9B, 0x9C, 0xB1, 0xB6, 0xBF, 0xB8, 0xAD, 0xAA, 0xA3, 0xA4,
    0xF9, 0xFE, 0xF7, 0xF0, 0xE5, 0xE2, 0xEB, 0xEC, 0xC1, 0xC6, 0xCF, 0xC8,
    0xDD, 0xDA, 0xD3, 0xD4, 0x69, 0x6E, 0x67, 0x60, 0x75, 0x72, 0x7B, 0x7C,
    0x51, 0x56, 0x5F, 0x58, 0x4D, 0x4A, 0x43, 0x44, 0x19, 0x1E, 0x17, 0x10,
    0x5,  0x2,  0xB,  0xC,  0x21, 0x26, 0x2F, 0x28, 0x3D, 0x3A, 0x33, 0x34,
    0x4E, 0x49, 0x40, 0x47, 0x52, 0x55, 0x5C, 0x5B, 0x76, 0x71, 0x78, 0x7F,
    0x6A, 0x6D, 0x64, 0x63, 0x3E, 0x39, 0x30, 0x37, 0x22, 0x25, 0x2C, 0x2B,
    0x6,  0x1,  0x8,  0xF,  0x1A, 0x1D, 0x14, 0x13, 0xAE, 0xA9, 0xA0, 0xA7,
    0xB2, 0xB5, 0xBC, 0xBB, 0x96, 0x91, 0x98, 0x9F, 0x8A, 0x8D, 0x84, 0x83,
    0xDE, 0xD9, 0xD0, 0xD7, 0xC2, 0xC5, 0xCC, 0xCB, 0xE6, 0xE1, 0xE8, 0xEF,
    0xFA, 0xFD, 0xF4, 0xF3,
];


fn crc8(msg: &[u8]) -> u8 {
    let mut crc: u8 = 0;

    for &byte in msg {
        crc = CRC_TABLE[(crc ^ byte) as usize];
    }

    crc
}

impl IONICOMPacketType {
    pub fn new_from(txdata: Vec<u8>) -> Self {
        let mut payload = [0u8; ICOM_MSG_PAYLOAD_MAX_LEN];
        payload[..txdata.len()].copy_from_slice(&txdata);
    
        let payload_len = txdata.len() as u16;
        
        // Create a buffer with PayloadLen and Payload for CRC calculation
        let mut crc_buffer = Vec::with_capacity(2 + txdata.len()); // Adjust capacity for u16 (2 bytes)
        crc_buffer.push((payload_len & 0xFF) as u8); // Low byte of u16
        crc_buffer.push((payload_len >> 8) as u8);   // High byte of u16
        crc_buffer.extend_from_slice(&payload); // Add Payload to the buffer

        let crc = crc8(&crc_buffer); // Calculate CRC on PayloadLen and Payload
            
        IONICOMPacketType {
            PayloadLen: payload_len,
            Payload: payload,
            Crc: crc,
        }
    }
    
    pub fn new_dummy() -> Self {
        let payload = [0u8; ICOM_MSG_PAYLOAD_MAX_LEN]; // Create a payload with all zeros
        let payload_len = 0;
        let crc: u8 = 0;

        IONICOMPacketType {
            PayloadLen: payload_len,
            Payload: payload,
            Crc: crc,
        }
    }

    pub fn is_dummy() -> bool {
        return self.payload_len == 0;
    }

    pub fn dump(&self) {
        println!("====================================================");
        println!("Payload Length: {}", self.PayloadLen);
        println!("CRC: 0x{:02X}", self.Crc);
        println!("Payload Dump:");

        for (i, chunk) in self.Payload.chunks(ICOM_FN_MAX_LEN).enumerate() {
            // Display the function index
            println!("Function {}:", i);

            // Print the chunk in hexadecimal
            for byte in chunk {
                print!("{:02X} ", byte);
            }

            println!(); // Newline after each function chunk
        }
        println!("====================================================");
    }

    pub fn get_func(&self, fncode: u8) -> Result<Vec<u8>, &'static str> {
        let start = fncode as usize * ICOM_FN_MAX_LEN;
        let end = start + ICOM_FN_MAX_LEN;

        // Ensure that the requested function is within bounds
        if end > self.PayloadLen as usize || start >= self.PayloadLen as usize {
            return Err("Function code out of bounds");
        }
        if self.Payload[start] != 0 {
            // Extract the corresponding function slice
            let func_data = self.Payload[start..end].to_vec();
            Ok(func_data)
        } else {
            return Err("Function code fncode empty");
        }
    }

    pub fn set_func(&mut self, fncode: u8, data: Vec<u8>) -> Result<(), &'static str> {
        let start = fncode as usize * ICOM_FN_MAX_LEN;
        let end = start + data.len();

        // Ensure that the data will fit within the payload
        if end > ICOM_MSG_PAYLOAD_MAX_LEN || data.len() > ICOM_FN_MAX_LEN {
            return Err("Data exceeds function or payload bounds");
        }

        // Insert the data into the correct portion of the payload
        self.Payload[start..start + data.len()].copy_from_slice(&data);

        // Update the payload length if necessary
        self.PayloadLen = 256;

        // Recalculate the CRC
        let mut crc_buffer = Vec::with_capacity(2 + self.PayloadLen as usize);
        crc_buffer.push((self.PayloadLen & 0xFF) as u8);  // Low byte
        crc_buffer.push((self.PayloadLen >> 8) as u8);    // High byte
        crc_buffer.extend_from_slice(&self.Payload[..self.PayloadLen as usize]);
        self.Crc = crc8(&crc_buffer);

        Ok(())
    }

    pub fn to_byte_array(&self) -> [u8; ICOM_MSG_MAX_LEN] {
        let mut buffer = [0u8; ICOM_MSG_MAX_LEN];
        
        // First two bytes are the payload length (u16)
        buffer[0] = (self.PayloadLen & 0xFF) as u8;  // Low byte
        buffer[1] = (self.PayloadLen >> 8) as u8;    // High byte
        
        // Next ICOM_MSG_PAYLOAD_MAX_LEN bytes are the payload
        buffer[2..ICOM_MSG_PAYLOAD_MAX_LEN+2].copy_from_slice(&self.Payload);
        
        // Last byte is the CRC
        buffer[ICOM_MSG_PAYLOAD_MAX_LEN+2] = self.Crc;

        buffer
    }

    pub fn payload_to_array(&self) -> [u8; ICOM_MSG_PAYLOAD_MAX_LEN] {
        let mut buffer = [0u8; ICOM_MSG_PAYLOAD_MAX_LEN];

        // Copy the payload data
        buffer.copy_from_slice(&self.Payload);
        
        buffer
    }
    
    // Converts a byte array (Vec<u8>) back to IONICOMPacketType
    pub fn from_byte_array(rxdata: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        if rxdata.len() != ICOM_MSG_MAX_LEN {
            return Err("Invalid byte array length. Expected ICOM_MSG_MAX_LEN bytes.".into());
        }

        // Extract PayloadLen (u16)
        let payload_len = u16::from_le_bytes([rxdata[0], rxdata[1]]);
        
        if payload_len == 0 {
            return Err("Dummy package received".into());
        } else if payload_len as usize > ICOM_MSG_PAYLOAD_MAX_LEN {
            return Err("Package corrupted".into());
        }

        // Extract Payload
        let mut payload = [0u8; ICOM_MSG_PAYLOAD_MAX_LEN];
        payload.copy_from_slice(&rxdata[2..ICOM_MSG_PAYLOAD_MAX_LEN+2]);

        // Extract CRC
        let crc = rxdata[ICOM_MSG_PAYLOAD_MAX_LEN+2];

        // Create IONICOMPacketType
        let packet = IONICOMPacketType {
            PayloadLen: payload_len,
            Payload: payload,
            Crc: crc,
        };

        // Verify CRC
        if !packet.verify_crc() {
            return Err("CRC check failed.".into());
        }

        Ok(packet)
    }

    // Verifies the CRC for the current payload and payload length
    pub fn verify_crc(&self) -> bool {
        if self.PayloadLen as usize > ICOM_MSG_PAYLOAD_MAX_LEN {
            return false;
        }
        let mut crc_buffer = Vec::with_capacity(2 + self.PayloadLen as usize);
        crc_buffer.push((self.PayloadLen & 0xFF) as u8); // Low byte
        crc_buffer.push((self.PayloadLen >> 8) as u8);   // High byte
        crc_buffer.extend_from_slice(&self.Payload[..self.PayloadLen as usize]); // Add actual payload data

        let computed_crc = crc8(&crc_buffer);

        // Return true if the computed CRC matches the stored CRC
        computed_crc == self.Crc
    }
}
