use crate::dsl::File;

pub trait DslError {
    /// Prints the error to stdout using [ariadne].
    ///
    /// `file`: the [File] this error was generated for.
    ///
    /// `lexer_spans`: the [spans](logos::Span) that were generated along with the [tokens](Token)
    /// in the function [process_tokens](crate::dsl::lexer::process_tokens)
    ///
    /// # Example
    /// ```ignore
    /// use postar::dsl::{File, lexer::process_tokens, parser::string_matcher};
    /// let file = File {
    ///     file_name: "test".to_string(),
    ///     contents: "contains \"test\"".to_string(),
    /// };
    /// let tokens = process_tokens(&file);
    /// if let Ok(tokens) = tokens {
    ///     let (only_tokens, only_spans): (Vec<Token>, Vec<Span>) = tokens.into_iter().unzip();
    ///     let res = string_matcher().parse(&only_tokens);
    ///     dbg!(
    ///         res.errors()
    ///             .for_each(|err| err.print_error(&file, &only_spans)),
    ///     );
    /// }
    /// ```
    fn print_error(&self, file: &File);
}
