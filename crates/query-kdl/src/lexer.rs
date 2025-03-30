#[derive(Debug, PartialEq, Eq)]
pub enum TokenType<'a> {
    String(&'a str),
    Alphanumeric(&'a str),
    Slash,
    DoubleSlash,
    Point,
    DoublePoint,
    Star,
    DoubleStar,
    EnterSquareBracket,
    LeaveSquareBracket,
    EnterCurlyBracket,
    LeaveCurlyBracket,
    Equal,
    Pipe,
    Unknown(&'a str),
}

impl<'a> std::fmt::Display for TokenType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::String(v) => write!(f, "{}", v),
            TokenType::Alphanumeric(v) => write!(f, "{}", v),
            TokenType::Slash => write!(f, "/"),
            TokenType::DoubleSlash => write!(f, "//"),
            TokenType::Point => write!(f, "."),
            TokenType::DoublePoint => write!(f, ".."),
            TokenType::Star => write!(f, "*"),
            TokenType::DoubleStar => write!(f, "**"),
            TokenType::EnterSquareBracket => write!(f, "["),
            TokenType::LeaveSquareBracket => write!(f, "]"),
            TokenType::EnterCurlyBracket => write!(f, "{{"),
            TokenType::LeaveCurlyBracket => write!(f, "}}"),
            TokenType::Equal => write!(f, "="),
            TokenType::Pipe => write!(f, "|"),
            TokenType::Unknown(v) => write!(f, "<unknown: {}>", v),
        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
}

impl<'a, T: AsRef<str> + ?Sized> From<&'a T> for Lexer<'a> {
    fn from(value: &'a T) -> Self {
        Self {
            input: value.as_ref(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = TokenType<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespaces
        let byte_offset = self
            .input
            .chars()
            .take_while(|c| c.is_whitespace())
            .map(|c| c.len_utf8())
            .sum();

        self.input = &self.input[byte_offset..];
        let c_token = self.input.chars().next();
        match c_token {
            None => None,
            Some('"' | '\'') => self.get_text(),
            Some(c) if c.is_alphanumeric() => self.get_alphanumeric(),
            Some(_) => self.get_token(),
        }
    }
}
impl<'a> Lexer<'a> {
    fn get_text(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut iter_chars = self.input.chars();
        let c_str = iter_chars.next().unwrap();
        let mut escaped = false;
        let mut len_str = c_str.len_utf8();
        for c in iter_chars {
            len_str += c.len_utf8();
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
            } else if c == c_str {
                break;
            }
        }
        self.advance_and_return(len_str).map(TokenType::String)
    }
    fn get_alphanumeric(&mut self) -> Option<<Self as Iterator>::Item> {
        let it = self
            .input
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
            .map(|c| (c.len_utf8(), c));
        let mut n_points = 0;
        let mut len = 0;
        for (clen, c) in it {
            len += clen;
            if c != '.' {
                n_points = 0;
                continue;
            }
            n_points += 1;
            if n_points == 2 {
                len -= 2;
                break;
            }
        }
        self.advance_and_return(len).map(TokenType::Alphanumeric)
    }
    fn get_token(&mut self) -> Option<<Self as Iterator>::Item> {
        use TokenType::*;
        let mut iter_chars = self.input.chars().map(|c| (c.len_utf8(), c));
        let (mut offset, c) = iter_chars.next()?;
        let result = match c {
            '/' => match iter_chars.next() {
                Some((l, '/')) => {
                    offset += l;
                    DoubleSlash
                }
                Some(_) | None => Slash,
            },
            '[' => EnterSquareBracket,
            ']' => LeaveSquareBracket,
            '{' => EnterCurlyBracket,
            '}' => LeaveCurlyBracket,
            '.' => match iter_chars.next() {
                Some((l, '.')) => {
                    offset += l;
                    DoublePoint
                }
                Some(_) | None => Point,
            },
            '*' => match iter_chars.next() {
                Some((l, '*')) => {
                    offset += l;
                    DoubleStar
                }
                Some(_) | None => Star,
            },
            '=' => Equal,
            '|' => Pipe,
            c => Unknown(&self.input[0..c.len_utf8()]),
        };
        self.input = &self.input[offset..];
        Some(result)
    }
    #[inline]
    fn advance_and_return(&mut self, offset: usize) -> Option<&'a str> {
        let result = &self.input[0..offset];
        self.input = &self.input[offset..];
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, TokenType};
    #[test]
    fn text() {
        let mut lexer = Lexer::from("\"hello\"");
        assert_eq!(lexer.next(), Some(TokenType::String("\"hello\"")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn spaced_text() {
        let mut lexer = Lexer::from("   \"hello\"  ");
        assert_eq!(lexer.next(), Some(TokenType::String("\"hello\"")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn wrong_text() {
        let mut lexer = Lexer::from("\"hello");
        assert_eq!(lexer.next(), Some(TokenType::String("\"hello")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn text_escaped() {
        let mut lexer = Lexer::from(r#""hello\"world""#);
        assert_eq!(lexer.next(), Some(TokenType::String(r#""hello\"world""#)));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_text() {
        let mut lexer = Lexer::from(r#""hello" "world""#);
        assert_eq!(lexer.next(), Some(TokenType::String(r#""hello""#)));
        assert_eq!(lexer.next(), Some(TokenType::String(r#""world""#)));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn alpha() {
        let mut lexer = Lexer::from("abc");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn alphanumeric() {
        let mut lexer = Lexer::from("abc123");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc123")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn numeric() {
        let mut lexer = Lexer::from("123");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("123")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn numalpha() {
        let mut lexer = Lexer::from("123abc");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("123abc")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_alpha() {
        let mut lexer = Lexer::from("abc def ghijk");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("def")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("ghijk")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_numeric() {
        let mut lexer = Lexer::from("123 456 10938");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("123")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("456")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("10938")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_alphanumeric() {
        let mut lexer = Lexer::from("abc 4476 ghijk 73ab35");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("4476")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("ghijk")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("73ab35")));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn token_sqbrackets() {
        let mut lexer = Lexer::from("[]");
        assert_eq!(lexer.next(), Some(TokenType::EnterSquareBracket));
        assert_eq!(lexer.next(), Some(TokenType::LeaveSquareBracket));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn token_relatives() {
        let mut lexer = Lexer::from("/./../**/*");
        assert_eq!(lexer.next(), Some(TokenType::Slash));
        assert_eq!(lexer.next(), Some(TokenType::Point));
        assert_eq!(lexer.next(), Some(TokenType::Slash));
        assert_eq!(lexer.next(), Some(TokenType::DoublePoint));
        assert_eq!(lexer.next(), Some(TokenType::Slash));
        assert_eq!(lexer.next(), Some(TokenType::DoubleStar));
        assert_eq!(lexer.next(), Some(TokenType::Slash));
        assert_eq!(lexer.next(), Some(TokenType::Star));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn entries() {
        let mut lexer = Lexer::from(r#"[name=value 1 "2" name1 = value1 | name = value1]"#);
        assert_eq!(lexer.next(), Some(TokenType::EnterSquareBracket));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("name")));
        assert_eq!(lexer.next(), Some(TokenType::Equal));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("value")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("1")));
        assert_eq!(lexer.next(), Some(TokenType::String(r#""2""#)));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("name1")));
        assert_eq!(lexer.next(), Some(TokenType::Equal));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("value1")));
        assert_eq!(lexer.next(), Some(TokenType::Pipe));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("name")));
        assert_eq!(lexer.next(), Some(TokenType::Equal));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("value1")));
        assert_eq!(lexer.next(), Some(TokenType::LeaveSquareBracket));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn token_curly_brackets() {
        let mut lexer = Lexer::from("{}");
        assert_eq!(lexer.next(), Some(TokenType::EnterCurlyBracket));
        assert_eq!(lexer.next(), Some(TokenType::LeaveCurlyBracket));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn numbers() {
        let mut lexer = Lexer::from("name 1 1.2 1.2.3 1..2");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("name")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("1")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("1.2")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("1.2.3")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("1")));
        assert_eq!(lexer.next(), Some(TokenType::DoublePoint));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("2")));
        assert_eq!(lexer.next(), None);
    }
}
