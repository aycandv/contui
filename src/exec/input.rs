//! Encode key events for exec sessions

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Encode a KeyEvent into bytes to write to exec stdin.
pub fn encode_key_event(key: KeyEvent) -> Option<Vec<u8>> {
    if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
    }

    let bytes = match key.code {
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Backspace => b"\x7f".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::Esc => b"\x1b".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                vec![(c as u8) & 0x1f]
            } else {
                c.to_string().into_bytes()
            }
        }
        _ => return None,
    };

    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_basic_keys() {
        assert_eq!(
            encode_key_event(KeyEvent::from(KeyCode::Enter)).unwrap(),
            b"\r".to_vec()
        );
        assert_eq!(
            encode_key_event(KeyEvent::from(KeyCode::Char('a'))).unwrap(),
            b"a".to_vec()
        );
    }

    #[test]
    fn encodes_arrow_keys() {
        let up = encode_key_event(KeyEvent::from(KeyCode::Up)).unwrap();
        assert_eq!(up, b"\x1b[A".to_vec());
    }

    #[test]
    fn ctrl_e_is_none_for_exec() {
        let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert!(encode_key_event(key).is_none());
    }
}
