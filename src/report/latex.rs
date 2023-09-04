use std::convert::Infallible;
use std::io::Write;
use std::str::FromStr;

pub enum LatexElement {
    Environment {
        name: String,
        elements: Vec<LatexElement>,
    },
    Directive {
        name: String,
        opts: Vec<String>,
        args: Vec<String>,
    },
    Raw(String),
}

impl FromStr for LatexElement {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::raw(s))
    }
}

impl LatexElement {
    pub fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        match self {
            Self::Directive { name, opts, args } => {
                write!(writer, "\\{name}")?;
                for opt in opts {
                    write!(writer, "[{opt}]")?;
                }
                for arg in args {
                    write!(writer, "{{{arg}}}")?;
                }
                write!(writer, " ")
            }
            Self::Environment { name, elements } => {
                write!(writer, "\\begin{{{name}}} ")?;
                for element in elements {
                    element.write(writer)?;
                }
                write!(writer, "\\end{{{name}}} ")
            }
            Self::Raw(value) => {
                write!(writer, " {value} ")
            }
        }
    }

    #[inline]
    pub fn directive<S>(name: S, opts: Vec<String>, args: Vec<String>) -> Self
    where
        S: ToString,
    {
        let name = name.to_string();
        LatexElement::Directive { name, opts, args }
    }

    #[inline]
    pub fn raw<S: ToString>(value: S) -> Self {
        LatexElement::Raw(value.to_string())
    }

    #[inline]
    pub fn environment<S: ToString>(name: S, contents: Vec<LatexElement>) -> Self {
        Self::Environment {
            name: name.to_string(),
            elements: contents,
        }
    }
}

#[cfg(test)]
mod tests {
    

    use super::*;

    #[test]
    fn test_latex_empty_directive_writing() {
        let mut result: Vec<u8> = vec![];

        let directive = LatexElement::directive("test", vec![], vec![]);

        directive.write(&mut result).unwrap();

        assert_eq!(result, br"\test ");
    }

    #[test]
    fn test_latex_argument_directive_writing() {
        let mut result: Vec<u8> = vec![];

        let directive = LatexElement::directive("test", vec![], vec!["apple".to_string()]);

        directive.write(&mut result).unwrap();

        assert_eq!(result, br"\test{apple} ");
    }
    #[test]
    fn test_latex_options_directive_writing() {
        let mut result: Vec<u8> = vec![];

        let directive = LatexElement::directive("test", vec!["apple".to_string()], vec![]);

        directive.write(&mut result).unwrap();

        assert_eq!(result, br"\test[apple] ");
    }
    #[test]
    fn test_latex_args_and_opts_directive_writing() {
        let mut result: Vec<u8> = vec![];

        let directive = LatexElement::directive(
            "test",
            vec!["banana".to_string(), "pair".to_string()],
            vec!["apple".to_string(), "cucumber".to_string()],
        );

        directive.write(&mut result).unwrap();

        assert_eq!(result, br"\test[banana][pair]{apple}{cucumber} ");
    }

    #[test]
    fn test_latex_environment() {
        let mut result: Vec<u8> = vec![];
        let env = LatexElement::environment("center", vec![LatexElement::raw("some text")]);
        env.write(&mut result).unwrap();
        assert_eq!(result, br"\begin{center}  some text \end{center} ");
    }
}
