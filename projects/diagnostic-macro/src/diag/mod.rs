// use diagnostic::{
//     term::{
//         emit,
//         termcolor::{ColorChoice, StandardStream},
//         TerminalConfig,
//     },
//     DiagnosticLevel, FileID, TextStorage,
// };
// use proc_macro::Span;
// use proc_macro2::{TokenStream, TokenTree};
// use std::convert::TryFrom;
//
// pub fn my_macro(input: TokenStream) {
//     let call_file = FileID::try_from(Span::call_site().source_file().path()).unwrap();
//     let mix_file = FileID::try_from(Span::mixed_site().source_file().path()).unwrap();
//     let def_file = FileID::try_from(Span::def_site().source_file().path()).unwrap();
//
//     match input.into_iter().next() {
//         None => {}
//         Some(TokenTree::Literal(s)) => {
//             eprintln!("{:?}", s)
//         }
//         _ => {}
//     }
//
//     let mut store = TextStorage::default();
//     // let id1 = store.file(call_file).unwrap();
//     let id1 = store.file(call_file).unwrap();
//     let id2 = store.file(mix_file).unwrap();
//     let id3 = store.file(def_file).unwrap();
//     let raw = diagnostic::Diagnostic::new(DiagnosticLevel::Error)
//         .with_message("unknown builtin: 1")
//         .with_primary(id1, 0..10, "unknown builtin")
//         .with_secondary(id2, 0..10, "unknown builtin")
//         .with_secondary(id3, 0..10, "unknown builtin")
//         .with_note("there is a builtin with a similar name: `NATURAL`");
//
//     let writer = StandardStream::stderr(ColorChoice::Always);
//     let config = TerminalConfig::default();
//     for diagnostic in &vec![raw.clone()] {
//         emit(&mut writer.lock(), &config, &store, diagnostic).unwrap();
//     }
//     //  Err(input.source_text().error("there's a problem here..."))
// }
