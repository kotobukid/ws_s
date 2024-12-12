use uuid::{Uuid, Version};

fn main() -> Result<(), uuid::Error> {
    let my_uuid = Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8")?;

    assert_eq!(Some(Version::Random), my_uuid.get_version());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_uuid() {
        let my_uuid = Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();
        assert_eq!(my_uuid.get_version(), Some(Version::Random));
    }

    #[test]
    fn test_uuid_parse_invalid() {
        // v4は第3セクションの頭の文字が4である必要があるのでパースに失敗させる
        let parsed_uuid = Uuid::parse_str("67e55044-10b1-126f-9247-bb680e5fe0c8").unwrap();
        // Version::Random は4(doc.rs参照)
        assert_ne!(parsed_uuid.get_version(), Some(Version::Random));
    }
}
