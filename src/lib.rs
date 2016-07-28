#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private, quote)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
extern crate walkdir;

use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use rustc_plugin::Registry;

use syntax::ast::{Stmt, LitKind};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager, get_single_str_from_tts};
use syntax::ext::build::AstBuilder;
use syntax::tokenstream::TokenTree;

use walkdir::WalkDir;
use walkdir::DirEntry;

fn encode_entry(cx: &mut ExtCtxt,
                sp: Span,
                relative_path: &str,
                entry: &DirEntry)
                -> Result<Option<Stmt>, String> {
    let name = match entry.path().strip_prefix(Path::new(relative_path)).unwrap().to_str() {
        Some(s) => s,
        None => {
            return Err("one of the paths we tried to embed! is not a valid string".to_string());
        }
    };
    if !entry.path().is_file() {
        return Ok(None);
    }
    let mut content = Vec::new();
    let mut file = match File::open(entry.path()) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("couldn't open file {} to embed! it because: {}", name, e));
        }
    };
    match file.read_to_end(&mut content) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("couldn't read file {} to end to embed! it because: {}",
                               name,
                               e));
        }
    };

    let args = vec![name.as_bytes().to_vec(), content];
    let args_wrapped = args.into_iter()
        .map(|a| {
            cx.expr_method_call(sp,
                                cx.expr_lit(sp, LitKind::ByteStr(Rc::new(a))),
                                cx.ident_of("to_vec"),
                                Vec::new())
        })
        .collect::<Vec<_>>();

    Ok(Some(cx.stmt_expr(cx.expr_method_call(sp,
                                             cx.expr_ident(sp, cx.ident_of("files")),
                                             cx.ident_of("insert"),
                                             args_wrapped))))
}

fn expand_embed(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    if args.len() != 1 {
        cx.span_err(sp,
                    &format!("argument to embed! should be a single string, but got {} arguments",
                             args.len()));
        return DummyResult::any(sp);
    }

    let relative_path = match get_single_str_from_tts(cx, sp, args, "embed!") {
        None => {
            cx.span_err(sp, "argument to embed! should be a string");
            return DummyResult::any(sp);
        }
        Some(p) => p,
    };

    let mut stmts = Vec::new();

    for entry in WalkDir::new(&relative_path) {
        match entry {
            Ok(ref entry) => {
                match encode_entry(cx, sp, &relative_path, entry) {
                    Ok(Some(stmt)) => {
                        stmts.push(stmt);
                    }
                    Ok(None) => {}
                    Err(err) => {
                        cx.span_err(sp, &err);
                        return DummyResult::any(sp);
                    }
                }
            }
            Err(e) => {
                cx.span_err(sp,
                            &format!("error while walking directory tree to embed!: {}", e));
                return DummyResult::any(sp);
            }
        }
    }

    let block = quote_block!(cx, {
        use std::collections::HashMap;
        let mut files: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        $stmts

        files
    });
    MacEager::expr(cx.expr_block(block))
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("embed", expand_embed);
}
