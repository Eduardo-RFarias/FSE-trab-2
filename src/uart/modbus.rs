use crate::uart::crc;

const SOURCE_ADDRESS: u8 = 0x00;
const TARGET_ADDRESS: u8 = 0x01;
const REGISTER_CODE: [u8; 4] = [6, 5, 2, 1];

#[derive(Clone, Copy)]
pub struct ModbusOperation {
    code: u8,
    subcode: u8,
    qtd: Option<u8>,
}

pub const READ_ENCODER: ModbusOperation = ModbusOperation {
    code: 0x23,
    subcode: 0xC1,
    qtd: None,
};

pub const SEND_PWM: ModbusOperation = ModbusOperation {
    code: 0x16,
    subcode: 0xC2,
    qtd: None,
};

pub const SEND_TEMP: ModbusOperation = ModbusOperation {
    code: 0x16,
    subcode: 0xD1,
    qtd: None,
};

#[allow(non_snake_case)]
pub fn READ_REGISTERS(address: u8, qtd: u8) -> ModbusOperation {
    ModbusOperation {
        code: 0x03,
        subcode: address,
        qtd: Some(qtd),
    }
}

#[allow(non_snake_case)]
pub fn WRITE_REGISTERS(address: u8, qtd: u8) -> ModbusOperation {
    ModbusOperation {
        code: 0x06,
        subcode: address,
        qtd: Some(qtd),
    }
}

pub fn create_modbus(operation: ModbusOperation, data: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(9 + data.len());

    // Address
    buffer.push(TARGET_ADDRESS);

    // Code
    buffer.push(operation.code);

    // Subcode
    buffer.push(operation.subcode);

    // Data
    for byte in data {
        buffer.push(*byte);
    }

    // Register code (Matrícula)
    for byte in REGISTER_CODE {
        buffer.push(byte);
    }

    // CRC16
    for byte in crc::hash(&buffer).to_le_bytes() {
        buffer.push(byte);
    }

    buffer
}

pub fn read_modbus(operation: ModbusOperation, buffer: &[u8]) -> Result<&[u8], String> {
    if buffer.len() < 5 {
        return Err(format!("Invalid length: {} < 5", buffer.len()));
    }

    // Every response must start with the source address
    if buffer[0] != SOURCE_ADDRESS {
        return Err(format!(
            "Invalid address: {:X} != {:X}",
            buffer[0], SOURCE_ADDRESS
        ));
    }

    // Every response must have the same code as the request
    if buffer[1] != operation.code {
        return Err(format!(
            "Invalid code: {:X} != {:X}",
            buffer[1], operation.code
        ));
    }

    // If the request does not have a quantity, the next byte must be the subcode
    let data;

    if operation.qtd.is_none() {
        if buffer[2] != operation.subcode {
            return Err(format!(
                "Invalid subcode: {:X} != {:X}",
                buffer[2], operation.subcode
            ));
        }

        // The data is the remaining bytes - 2 (CRC16)
        data = &buffer[3..buffer.len() - 2];
    } else {
        data = &buffer[2..buffer.len() - 2];
    }

    // If the request has a quantity, the data length must match
    if let Some(qtd) = operation.qtd {
        if data.len() != qtd as usize {
            return Err(format!("Invalid data length: {} != {}", data.len(), qtd));
        }
    }

    // Calculate the CRC16 and compare with the one provided
    let crc = u16::from_le_bytes([buffer[buffer.len() - 2], buffer[buffer.len() - 1]]);
    let expected_crc = crc::hash(&buffer[..buffer.len() - 2]);

    if crc != expected_crc {
        return Err(format!("Invalid CRC16: {:X} != {:X}", crc, expected_crc));
    }

    Ok(data)
}
