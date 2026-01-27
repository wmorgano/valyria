use assert_cmd::cargo;

fn get_repl_test_command() -> assert_cmd::Command {
    cargo::cargo_bin_cmd!()
}

fn make_repl_input(repl_cmds: &[&str]) -> String {
    repl_cmds.join("\n") + "\nquit"
}

fn make_repl_output(repl_cmds: &[&str]) -> String {
    let mut outputs = String::from(">> ");
    outputs.push_str(&repl_cmds.join("\n>> "));
    outputs.push_str("\n>> ");
    outputs
}

fn make_repl_test_io(repl_cmds: &[(&str, &str)]) -> (String, String) {
    let (inputs, outputs): (Vec<&str>, Vec<&str>) = repl_cmds.iter().copied().unzip();
    (make_repl_input(&inputs), make_repl_output(&outputs))
}

#[test]
fn run_repl() {
    let mut cmd = get_repl_test_command();
    let assert = cmd.write_stdin("quit").assert();
    assert.success();
}

#[test]
fn run_repl_with_integer_input() {
    let mut cmd = get_repl_test_command();
    let (repl_input, repl_output) = make_repl_test_io(&[("123", "123")]);
    let assert = cmd.write_stdin(repl_input).assert();
    assert.stdout(repl_output).success();
}
