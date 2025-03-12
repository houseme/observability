use tracing::error;

// UCS2 解码
pub fn decode_ucs2(src: &str) -> String {
    let mut utf16_words = Vec::with_capacity(src.len() / 4);
    for i in (0..src.len()).step_by(4) {
        if i + 4 <= src.len() {
            let substr = &src[i..i + 4];
            match u16::from_str_radix(substr, 16) {
                Ok(word) => utf16_words.push(word),
                Err(e) => {
                    error!("Failed to parse UCS2 hex string '{}': {}", substr, e);
                    utf16_words.push(0);
                }
            }
        }
    }

    String::from_utf16(&utf16_words).unwrap_or_else(|e| {
        error!("UCS2 解码失败：{}", e);
        String::new()
    })
}

// UCS2 编码
pub fn encode_ucs2(src: &str) -> String {
    let bytes = src.encode_utf16().collect::<Vec<u16>>();
    let mut result = String::new();
    for &byte in &bytes {
        let hex_str = format!("{:02X}", byte);
        result.push_str(&hex_str);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_ucs2() {
        let src = "6D4B8BD5";
        let result = decode_ucs2(src);
        println!("解码：{}", result);
        assert_eq!(result, "测试");
    }

    #[test]
    fn test_encode_ucs2() {
        let src = "测试";
        let result = encode_ucs2(src);
        println!("编码：{}", result);
        assert_eq!(result, "6D4B8BD5");
    }

    #[test]
    fn test_transfer_0x00() {
        let src = "测试\u{0000}一下";
        let result = transfer_0x00(src.to_string());
        println!("替换：{}", result);
        assert_eq!(result, "测试 一下");
    }

    #[test]
    fn test_char() {
        let c = char::from_u32(0x6D4B).unwrap();
        println!("字符：{}", c);
        assert_eq!(c, "测试".chars().next().unwrap());
    }

    #[test]
    fn test_encode_decode() {
        let src = "测试一下！";
        let encode = encode_ucs2(src);
        println!("加密：{}", encode);
        let decode = decode_ucs2(&encode);
        println!("解密：{}", decode);
        assert_eq!(decode, src);
    }
}
fn main() {
    let content = "测试一下！";
    let encode = encode_ucs2(content);
    println!("加密：{}", encode);
    let decode = decode_ucs2(&encode);
    println!("解密：{}", decode);
}
