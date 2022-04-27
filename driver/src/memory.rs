#[allow(dead_code)]
pub struct PSProtection {
    pub protection_type: u8,
    pub protection_audit: u8,
    pub protection_signer: u8,
}

impl Default for PSProtection {
    fn default() -> Self {
        Self { protection_type: 3, protection_audit: 1, protection_signer: 4 }
    }
}

pub struct ProcessProtectionInformation {
    pub signature_level: u8,
	pub section_signature_level: u8,
	pub protection: PSProtection,
}