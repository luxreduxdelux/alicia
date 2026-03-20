#[derive(Debug)]
pub enum Token {
    String(String),
    Immediate(Immediate),
    Function,
    Let,
    ParenthesisBegin,
    ParenthesisClose,
    BracketBegin,
    BracketClose,
    Colon,
    Comma,
    Assignment,
}

impl Token {
    pub fn parse_line(line: &str, list: &mut Vec<Token>) {
        if line.is_empty() {
            return;
        }

        let mut buffer = String::new();
        let mut inside_string = false;

        for character in line.chars() {
            match character {
                ' ' => {
                    if !inside_string {
                        if !buffer.is_empty() {
                            list.push(Self::parse_text(&buffer));
                        }

                        buffer.clear();
                    } else {
                        buffer.push(character);
                    }
                }
                '"' => {
                    if inside_string {
                        if !buffer.is_empty() {
                            list.push(Self::parse_text(&buffer));
                        }

                        buffer.clear();
                    }

                    inside_string = !inside_string;
                }
                '(' | ')' | ',' => {
                    buffer.push(character);
                    list.push(Self::parse_text(&buffer));
                    buffer.clear();
                }
                _ => buffer.push(character),
            }
        }

        if !buffer.is_empty() {
            list.push(Self::parse_text(&buffer));
        }
    }

    fn parse_text(text: &str) -> Self {
        match text {
            "function" => Self::Function,
            "let" => Self::Let,
            "(" => Self::ParenthesisBegin,
            ")" => Self::ParenthesisClose,
            "{" => Self::BracketBegin,
            "}" => Self::BracketClose,
            ":" => Self::Colon,
            "," => Self::Comma,
            ":=" => Self::Assignment,
            _ => {
                if let Some(immediate) = Immediate::parse(text) {
                    Self::Immediate(immediate)
                } else {
                    Self::String(text.to_string())
                }
            }
        }
    }
}

//================================================================

#[derive(Debug)]
enum Immediate {
    String(String),
    Integer(i32),
    Decimal(f32),
    Boolean(bool),
}

impl Immediate {
    fn parse(text: &str) -> Option<Self> {
        if text == "true" {
            return Some(Self::Boolean(true));
        } else if text == "false" {
            return Some(Self::Boolean(false));
        } else if text.starts_with("\"") && text.ends_with("\"") {
            let text = &text[1..text.len() - 1];
            return Some(Self::String(text.to_string()));
        } else if let Ok(integer) = text.parse::<i32>() {
            return Some(Self::Integer(integer));
        } else if let Ok(decimal) = text.parse::<f32>() {
            return Some(Self::Decimal(decimal));
        }

        None
    }
}
