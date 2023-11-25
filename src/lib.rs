mod bounding;
mod chat;
mod cluster;
mod kdtree;
mod shape;

use arma_rs::{arma, Context, Extension};

use std::result::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime");
    static ref CONTEXT: RwLock<Option<Context>> = RwLock::new(None);
}

#[arma]
fn init() -> Extension {
    let ext = Extension::build()
        .group("chat", chat::group())
        // .command("cluster", cluster::entrypoint)
        .command("ok", || -> Result<String, String> { Ok("OK".to_owned()) })
        .command("err", || -> Result<String, String> {
            Err("ERR".to_owned())
        })
        .command("echo", echo)
        .command("hint", hint)
        .command("call", call)
        .command("uuid", Uuid::new_v4)
        .finish();

    let ctx_tokio = ext.context();
    std::thread::spawn(move || {
        RUNTIME.block_on(async {
            *CONTEXT.write().await = Some(ctx_tokio);
            // tokio::join!(handler::start(), listener::start());
        });
    });
    ext
}

fn echo(input: Vec<String>) -> String {
    format!("echo({})", input.join(", "))
}

fn hint(context: Context, input: String) -> Result<String, String> {
    match context.callback_data("dynops", "hint", input) {
        Ok(_) => Ok("OK".to_owned()),
        Err(e) => Err(e.to_string()),
    }
}

fn call(context: Context, input: String) -> Result<String, String> {
    match context.callback_data("dynops", "dynops_extension_fnc_callback", input) {
        Ok(_) => Ok("OK".to_owned()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use arma_rs::IntoArma;

    use super::init;

    #[test]
    fn ok() {
        let extension = init().testing();
        let (output, code) = extension.call("ok", None);
        assert_eq!(code, 0);
        assert_eq!(output, "OK");
    }

    #[test]
    fn err() {
        let extension = init().testing();
        let (output, code) = extension.call("err", None);
        assert_eq!(code, 9);
        assert_eq!(output, "ERR");
    }

    #[test]
    fn echo() {
        let extension = init().testing();
        let args = vec!["a".to_string(), "b".to_string()];
        let (output, code) = extension.call("echo", Some(vec![args.to_arma().to_string()]));
        assert_eq!(code, 0);
        assert_eq!(output, "echo(a, b)");
    }
}
