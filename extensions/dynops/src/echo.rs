pub fn echo(args: &[String]) -> std::string::String {
    format!("echo({})", args.join(", "))
}
