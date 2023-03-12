mod cluster;
//mod kdtree;
mod bounding;
mod shape;

use arma_rs::{arma, Extension};
use std::result::Result;
use uuid::Uuid;

#[arma]
fn init() -> Extension {
    Extension::build()
        // .command("cluster", cluster::entrypoint)
        // .command("datetime", Utc::now)
        .command("ok", || -> Result<String, String> { Ok("OK".to_owned()) })
        .command("err", || -> Result<String, String> {
            Err("ERR".to_owned())
        })
        .command("echo", echo)
        .command("uuid", Uuid::new_v4)
        .finish()
}

fn echo(input: Vec<String>) -> String {
    format!("echo({})", input.join(", "))
}

#[cfg(test)]
mod tests {
    use arma_rs::IntoArma;

    use super::init;

    #[test]
    fn ok() {
        let extension = init().testing();
        let (output, code) = unsafe { extension.call("ok", None) };
        assert_eq!(code, 0);
        assert_eq!(output, "OK");
    }

    #[test]
    fn err() {
        let extension = init().testing();
        let (output, code) = unsafe { extension.call("err", None) };
        assert_eq!(code, 9);
        assert_eq!(output, "ERR");
    }

    #[test]
    fn echo() {
        let extension = init().testing();
        let args = vec!["a".to_string(), "b".to_string()];
        let (output, code) =
            unsafe { extension.call("echo", Some(vec![args.to_arma().to_string()])) };
        assert_eq!(code, 0);
        assert_eq!(output, "echo(a, b)");
    }
}
