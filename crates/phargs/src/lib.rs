/// Splits a string by commas and collects the results into a `Vec<String>`.
///
/// # Examples
///
/// ```
/// let example = "one,two,three";
/// let words = phargs::comma_separated(example);
/// assert_eq!(words, vec!["one".to_string(), "two".to_string(), "three".to_string()]);
/// ```
pub fn comma_separated(s: &str) -> Vec<String> {
    s.split(',').map(|s| s.to_string()).collect()
}

/// Constructs a program path from the first argument to the current process.
///
/// This function attempts to prepend the directory of the current executable
/// to the given `program` name if possible. If not, it returns the `program` name as is.
///
/// # Arguments
///
/// * `program` - The program name to prepend.
/// * `arg0` - Typically `std::env::args().next().unwrap()`, representing the current executable.
///
/// # Examples
///
/// ```
/// let program = "rustc";
/// let arg0 = "/usr/bin/rust";
/// let full_path = phargs::program_from_arg0(program, arg0);
/// assert_eq!(full_path, "/usr/bin/rustc");
/// ```
pub fn program_from_arg0(program: &str, arg0: &str) -> String {
    arg0.rsplitn(2, '/')
        .nth(1)
        .map(|s| format!("{}/{}", s, program))
        .unwrap_or(program.to_string())
}

/// Finds the executable path in the environment.
///
/// If the path exists, it returns the constructed path, otherwise just the program name.
///
/// # Arguments
///
/// * `program` - The program name.
///
/// # Returns
///
/// Returns the fully qualified path if it exists, or the program name if it doesn't.
///
/// # Examples
///
/// ```
/// let found_program = phargs::find_program_from_env("bash");
/// println!("Found program: {}", found_program);
/// ```
pub fn find_program_from_env(program: &str) -> String {
    let p = program_from_arg0(program, &std::env::args().next().unwrap());
    if std::path::Path::new(&p).exists() {
        p
    } else {
        program.to_string()
    }
}

/// A command with placeholders.
///
/// This struct represents a command that may include placeholders (`{}`) for dynamic substitution.
pub struct PhCommand<'p, 'a> {
    program: &'p str,
    args: &'a [String],
    ph: String,
}

impl PhCommand<'_, '_> {
    pub fn program(&self) -> &str {
        self.program
    }

    /// Returns a new Vec of arguments with placeholders substituted.
    ///
    /// This method substitutes any occurrence of `{}` in the arguments with `ph`.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` with all placeholders substituted.
    pub fn args(&self) -> Vec<String> {
        self.args
            .iter()
            .map(|s| s.replace("{}", &self.ph))
            .collect()
    }

    /// Creates a `Command` ready to execute.
    ///
    /// # Returns
    ///
    /// Returns a `std::process::Command` with arguments ready to be executed.
    pub fn command(&self) -> std::process::Command {
        let mut command = std::process::Command::new(self.program);
        command.args(&self.args());
        command
    }

    /// Returns a string representation of the command.
    ///
    /// # Returns
    ///
    /// Returns a `String` that represents the full command to be executed.
    pub fn command_string(&self) -> String {
        let command = self.args().join(" ");
        format!("{} {}", self.program, command)
    }
}

pub struct PhCommandVec {
    program: String,
    args: Vec<String>,
    phargs: Vec<String>,
    args_has_ph: bool,
}

impl PhCommandVec {
    pub fn new<P: Into<String>, A: Into<String>, H: Into<String>>(
        program: P,
        args: Vec<A>,
        phargs: Vec<H>,
    ) -> Self {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        let phargs = phargs.into_iter().map(Into::into).collect::<Vec<_>>();
        let args = extend_row(args.iter(), &phargs);
        Self {
            args_has_ph: row_has_ph(args.iter()),
            program: program.into(),
            args,
            phargs,
        }
    }

    pub fn iter(&self) -> PhCommandIterZero<impl Iterator<Item = &String>> {
        PhCommandIterZero {
            program: &self.program,
            args: &self.args,
            phargs: self.phargs.iter(),
            args_has_ph: self.args_has_ph,
            is_first: true,
        }
    }
}

pub struct PhCommandIterZero<'p, 'a, P>
where
    P: Iterator,
    P::Item: Into<String>,
{
    program: &'p str,
    args: &'a [String],
    phargs: P,
    args_has_ph: bool,
    is_first: bool,
}

impl<'p, 'a, P> Iterator for PhCommandIterZero<'p, 'a, P>
where
    P: Iterator,
    P::Item: Into<String>,
{
    type Item = PhCommand<'p, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.args_has_ph && !self.is_first {
            None
        } else {
            self.is_first = false;
            self.phargs.next().map(|ph| PhCommand {
                program: self.program,
                args: self.args,
                ph: ph.into(),
            })
        }
    }
}

/// Transforms an array format string into individual strings with placeholders substituted.
///
/// This function interprets a format string and applies it to each item in `args`. If the format string
/// is enclosed in brackets, each item replaces a `{}` placeholder within the format.
///
/// # Arguments
///
/// * `fmt` - The format string, potentially enclosed in brackets.
/// * `args` - An iterator of items that will replace the `{}` placeholder.
///
/// # Returns
///
/// Returns a vector of strings with each `arg` formatted according to `fmt`.
///
/// # Examples
///
/// ```
/// let result = phargs::extend_array("[{}.txt]", vec!["file1", "file2"]);
/// assert_eq!(result, vec!["file1.txt", "file2.txt"]);
/// ```
pub fn extend_array<'a, S: AsRef<str>, T: AsRef<str> + 'a + ?Sized>(
    fmt: S,
    args: impl IntoIterator<Item = &'a T>,
) -> Vec<String> {
    let (first, last) = (
        fmt.as_ref().chars().next(),
        fmt.as_ref().chars().next_back(),
    );
    if Some('[') == first && Some(']') == last {
        let fmt = &fmt.as_ref()[1..fmt.as_ref().len() - 1];
        args.into_iter()
            .map(|s| fmt.replace("{}", s.as_ref()))
            .collect::<Vec<_>>()
    } else {
        vec![fmt.as_ref().to_string()]
    }
}

pub fn row_has_ph<'a, T: AsRef<str> + 'a>(row: impl IntoIterator<Item = &'a T>) -> bool {
    row.into_iter().any(|s| s.as_ref().contains("{}"))
}

/// Extends a row of format strings into a flat list of formatted strings.
///
/// This function takes a collection of format strings, some of which may include bracketed placeholders,
/// and flattens the result after applying each placeholder substitution.
///
/// # Arguments
///
/// * `row` - An iterable collection of format strings.
/// * `args` - An array of strings to substitute into format strings.
///
/// # Returns
///
/// Returns a flattened vector of strings after substituting placeholders.
///
/// # Examples
///
/// ```
/// let formats = vec!["plain text", "[{}.txt]"];
/// let args = vec!["file1", "file2"];
/// let extended = phargs::extend_row(formats, &args);
/// assert_eq!(extended, vec!["plain text", "file1.txt", "file2.txt"]);
/// ```
pub fn extend_row<'r, 'a, R: AsRef<str> + 'r + ?Sized, A: AsRef<str> + 'a>(
    row: impl IntoIterator<Item = &'r R>,
    args: &'a [A],
) -> Vec<String> {
    row.into_iter()
        .flat_map(|s| extend_array(s, args.iter()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comma_separated() {
        assert_eq!(comma_separated("a,b,c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_program_from_arg0() {
        assert_eq!(program_from_arg0("A", "/a"), "/A");
        assert_eq!(program_from_arg0("A", "/b/c/a"), "/b/c/A");
        assert_eq!(program_from_arg0("A", "b/c/a"), "b/c/A");
        assert_eq!(program_from_arg0("A", "./c/a"), "./c/A");
        assert_eq!(program_from_arg0("A", "../c/a"), "../c/A");
        assert_eq!(program_from_arg0("A", "a"), "A");
    }

    #[test]
    #[allow(clippy::useless_vec)]
    fn test_rows() {
        assert_eq!(
            extend_array("[{}.txt]", &vec!["a", "b"]),
            vec!["a.txt", "b.txt"]
        );
        assert_eq!(
            extend_array("{}.txt", vec!["a".to_string(), "b".to_string()].iter()),
            vec!["{}.txt"]
        );
    }

    #[test]
    #[allow(clippy::useless_vec)]
    fn test_row_has_ph() {
        assert!(row_has_ph(&vec!["a", "{}"]));
        assert!(row_has_ph(&vec!["a", "{}.txt"]));
        assert!(!row_has_ph(&vec!["a", "b", "c"]));
        let v = vec!["a".to_string(), "b".to_string()];
        assert!(!row_has_ph(v.iter()));
        assert!(!row_has_ph(v.iter()));
    }

    #[test]
    #[allow(clippy::useless_vec)]
    fn test_extend_row() {
        assert_eq!(
            extend_row(["a", "[{}.txt]"], &["1", "2"]),
            ["a", "1.txt", "2.txt"]
        );
        assert_eq!(
            extend_row(
                vec!["a".to_string(), "[{}.txt]".to_string()].iter(),
                &vec!["1".to_string(), "2".to_string()]
            ),
            vec!["a", "1.txt", "2.txt"]
        );
    }

    #[test]
    fn test_ph_command() {
        let pc = PhCommand {
            program: "echo",
            args: &["{}".to_string(), "b".to_string()],
            ph: "a".to_string(),
        };
        assert_eq!(pc.args(), vec!["a", "b"]);
        assert_eq!(pc.command_string(), "echo a b");
    }

    #[test]
    fn test_ph_command_vec() {
        let pcv = PhCommandVec::new(
            "echo".to_string(),
            vec!["{}".to_string(), "b".to_string()],
            vec!["a".to_string(), "c".to_string()],
        );
        let mut iter = pcv.iter();
        assert_eq!(iter.next().unwrap().command_string(), "echo a b");
        assert_eq!(iter.next().unwrap().command_string(), "echo c b");
        assert!(iter.next().is_none());

        let pcv = PhCommandVec::new("echo", vec!["[{}.txt]", "b"], vec!["a", "c"]);
        let mut iter = pcv.iter();
        assert_eq!(iter.next().unwrap().command_string(), "echo a.txt c.txt b");
        assert!(iter.next().is_none());

        let pcv = PhCommandVec::new("echo", vec!["[{}.txt]", "{}"], vec!["a", "c"]);
        let mut iter = pcv.iter();
        assert_eq!(iter.next().unwrap().command_string(), "echo a.txt c.txt a");
        assert_eq!(iter.next().unwrap().command_string(), "echo a.txt c.txt c");
        assert!(iter.next().is_none());
    }
}
