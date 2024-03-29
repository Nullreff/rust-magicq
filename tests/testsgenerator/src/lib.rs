use proc_macro::{TokenStream};
use quote::{quote};
use syn::{parse_macro_input, Ident, LitStr};
use walkdir::WalkDir;

#[proc_macro]
pub fn generate_tests(input: TokenStream) -> TokenStream {
    // Parse the input as a `LitStr`
    let dir = parse_macro_input!(input as LitStr);

    // Get the directory path as a string
    let dir_path = dir.value();

    // Find all show files in the directory
    let shw_files = WalkDir::new(&dir_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().map(|ext| ext == "shw").unwrap_or(false))
        .map(|entry| entry.path().to_owned())
        .collect::<Vec<_>>();

    // Generate a test for each file
    let tests = shw_files.iter().map(|file| {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        let file_path = file.as_path().to_str().unwrap();

        let safe_file_name = file_name.replace(&['/', '.', '\\', '-', ' '][..], "_");
        let test_name = Ident::new(&format!("test{}", safe_file_name), proc_macro2::Span::call_site());

        quote! {
            #[test]
            fn #test_name() {
                println!("Running tests on {}", #file_path);
                // Read in a showfile, check we parsed it then write it back out
                let input = std::fs::read_to_string(&#file_path).unwrap();

                let showfile = Showfile::from_str(&input).unwrap_or_else(|e|
                    panic!("\n{}", e)
                );

                let result = showfile.to_string();

                assert_eq!(input.replace("\r\n", "\n"), result.replace("\r\n", "\n"));
            }
        }
    });

    // Concatenate all the tests into a single `TokenStream`
    let output = quote! {
        #(#tests)*
    };

    // Return the generated code as a `TokenStream`
    TokenStream::from(output)
}