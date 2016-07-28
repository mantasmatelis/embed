# embed
`embed` is a Rust compiler plugin that lets you embed directory trees into compiled binaries. This is useful in order to ship a single self-contained binary.

First:

    [dependencies]
    embed = "0.1.1";
Then:

    #![feature(plugin)]
    #![plugin(embed)]

    use std::collections::HashMap;

    fn main() {
       let files: HashMap<Vec<u8>, Vec<u8>> = embed!("assets");
       for (name, content) in files {
           println!("{}: \"{}\"", String::from_utf8(name).unwrap(), String::from_utf8(content).unwrap().trim());
      }
    }

The output from this varies depending on what is contained in your projects `assets` directory, but could look something like:

    oneword: "hello"
    twowords: "hello world!"
    dir/file: "foo bar baz"
    dir/empty_file: ""
