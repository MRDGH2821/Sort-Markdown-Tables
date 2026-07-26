diff --git a/src/cli.rs b/src/cli.rs
index fff3e78..b831354 100644
--- a/src/cli.rs
+++ b/src/cli.rs
@@ -5,14 +5,25 @@
 //! The process entrypoint parses with [`finalize_cli`] (or [`parse_args`] wrapping
 //! [`Args::parse`]).
 use crate::error::SmtError;
-use clap::Parser;
+use clap::{Parser, Subcommand};
 use std::path::PathBuf;
 
+/// Subcommands supported by smt
+#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
+pub enum SmtSubcommand {
+    /// Display version information
+    Version,
+}
+
 /// Args struct for CLI argument parsing with clap
 #[derive(Parser, Debug)]
-#[command(name = "smt")]
+#[command(name = "Sort Markdown Tables")]
+#[command(version = concat!("v", env!("CARGO_PKG_VERSION")))]
 #[command(about = "Sort Markdown Tables", long_about = None)]
 pub struct Args {
+    /// Subcommand to run
+    #[command(subcommand)]
+    pub subcommand: Option<SmtSubcommand>,
     /// Input files or glob patterns
     pub inputs: Vec<String>,
     /// Sort files in-place
@@ -68,7 +79,9 @@ fn output_target_from_args(args: &Args) -> OutputTarget {
 ///
 /// Used by [`parse_args`] and by **binary** tests (and `main`) via
 /// [`Args::try_parse_from`].
-pub fn finalize_cli(args: Args) -> Result<(InputSource, OutputTarget, bool, bool), SmtError> {
+pub fn finalize_cli(
+    args: Args,
+) -> Result<(InputSource, OutputTarget, bool, bool, Option<SmtSubcommand>), SmtError> {
     let input_source = detect_input_source(args.inputs.clone())?;
     let output_target = output_target_from_args(&args);
 
@@ -84,13 +97,13 @@ pub fn finalize_cli(args: Args) -> Result<(InputSource, OutputTarget, bool, bool
         (OutputTarget::InPlace, InputSource::Stdin) => {
             return Err(SmtError::InPlaceWithStdin);
         },
-        _ => { },
+        _ => {},
     }
-    Ok((input_source, output_target, args.check, args.verbose))
+    Ok((input_source, output_target, args.check, args.verbose, args.subcommand))
 }
 
 /// Parse command-line arguments and validate flag combinations
-pub fn parse_args() -> Result<(InputSource, OutputTarget, bool, bool), SmtError> {
+pub fn parse_args() -> Result<(InputSource, OutputTarget, bool, bool, Option<SmtSubcommand>), SmtError> {
     finalize_cli(Args::parse())
 }
 
@@ -170,6 +183,12 @@ mod tests {
         Args::try_parse_from(argv.iter().copied())
     }
 
+    #[test]
+    fn test_cli_version_subcommand_parsing() {
+        let args = Args::try_parse_from(["smt", "version"]).unwrap();
+        assert_eq!(args.subcommand, Some(SmtSubcommand::Version));
+    }
+
     #[test]
     fn parse_flags_defaults_and_verbose() {
         let a = parse_args_from(&["smt"]).unwrap();
@@ -268,7 +287,7 @@ mod tests {
         std::fs::write(&f, "# x").unwrap();
         let argv = vec!["smt", f.to_str().unwrap(), "-w", "out.md"];
         let args = Args::try_parse_from(argv).unwrap();
-        let (src, out, check, verbose) = finalize_cli(args).unwrap();
+        let (src, out, check, verbose, _subcommand) = finalize_cli(args).unwrap();
         assert!(!check && !verbose);
         match (src, out) {
             (InputSource::Files(files), OutputTarget::File { path, append }) => {
diff --git a/src/main.rs b/src/main.rs
index 7c0b4c6..96ee35d 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -47,6 +47,7 @@ use smt::{
         Args,
         InputSource,
         OutputTarget,
+        SmtSubcommand,
     },
     error::SmtError,
     parser::{
@@ -104,8 +105,16 @@ fn main() {
 ///    which files are unsorted c. Exit 1 if any unsorted, 0 if all sorted
 ///
 /// 6. Otherwise: a. Write all documents to their targets b. Exit 0 on success
-fn run_with_routing(routing: (InputSource, OutputTarget, bool, bool), stdin_is_tty: bool) -> i32 {
-    let (input_source, output_target, check_mode, verbose) = routing;
+fn run_with_routing(
+    routing: (InputSource, OutputTarget, bool, bool, Option<SmtSubcommand>),
+    stdin_is_tty: bool,
+) -> i32 {
+    let (input_source, output_target, check_mode, verbose, subcommand) = routing;
+
+    if let Some(SmtSubcommand::Version) = subcommand {
+        println!("Sort Markdown Tables v{}", env!("CARGO_PKG_VERSION"));
+        return 0;
+    }
 
     // Special case: no positional args + TTY stdin -> print help, exit 0. (clap
     // already handles explicit `--help` / `--version`.)
