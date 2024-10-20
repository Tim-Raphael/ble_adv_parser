#[derive(Debug, PartialEq)]
pub enum ADType {
    Flags,
    ServiceClassUUIDs,
    ShortendLocalName,
    CompleteLocalName,
    Unknown(u8),
}

#[derive(Debug, PartialEq)]
pub struct ADStructure {
    pub ad_type: ADType,
    pub data: Vec<u8>,
}

pub fn parse_ad(data: &[u8]) -> Vec<ADStructure> {
    let mut structures = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // first byte contains length
        let l = data[i] as usize;
        if l == 0 || i + l >= data.len() {
            // a length of 0 or a length that extends beyond the data buffer indicates a
            // malformed packet
            break;
        }

        // second byte contains advertisement data type
        let t = match data[i + 1] {
            0x01 => ADType::Flags,
            0x03 => ADType::ServiceClassUUIDs,
            0x08 => ADType::ShortendLocalName,
            0x09 => ADType::CompleteLocalName,
            other => ADType::Unknown(other),
        };

        // the rest contains the actual data
        let d = data[i + 2..i + 1 + l].to_vec();
        structures.push(ADStructure {
            ad_type: t,
            data: d,
        });

        i += l + 1;
    }

    structures
}

pub fn parse_flags(data: &[u8]) -> Option<u8> {
    // extract advertisement data flag with bitwise flags
    if data.len() == 1 {
        // bit 0 = limited discoverable
        // bit 1 = general discoverable
        // bit 2 = BR/EDR not supported (Basic Rate is 721 kbps and Enhanced Data Rate is 2.1 mbps)
        Some(data[0])
    } else {
        None
    }
}

pub fn parse_local_name(data: &[u8]) -> Option<String> {
    // some devices advertise their local name as a readable string
    String::from_utf8(data.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ad() {
        let d = vec![
            0x02, 0x01, 0x06, 0x05, 0x09, b'T', b'e', b's', b't', 0x04, 0x08, 0x01, 0x02, 0x03,
        ];

        let e = vec![
            ADStructure {
                ad_type: ADType::Flags,
                data: vec![0x06],
            },
            ADStructure {
                ad_type: ADType::CompleteLocalName,
                data: vec![b'T', b'e', b's', b't'],
            },
            ADStructure {
                ad_type: ADType::ShortendLocalName,
                data: vec![0x01, 0x02, 0x03],
            },
        ];

        let result = parse_ad(&d);
        assert_eq!(result, e);
    }

    #[test]
    fn test_parse_flags() {
        assert_eq!(parse_flags(&[0x06]), Some(0x06));
        assert_eq!(parse_flags(&[0x06, 0x01]), None);
        assert_eq!(parse_flags(&[]), None);
    }

    #[test]
    fn test_parse_local_name() {
        assert_eq!(parse_local_name(b"Test"), Some("Test".to_string()));
        assert_eq!(
            parse_local_name(&[0xF0, 0x9F, 0xA6, 0x80]),
            Some("🦀".to_string())
        );
        assert_eq!(parse_local_name(&[0x54]), Some("T".to_string()));
        assert_eq!(parse_local_name(&[]), Some("".to_string()));
    }
}
