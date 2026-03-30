mod shell;
use shell::Shell;

fn main() {
    let new_shell = Shell::new();
    new_shell.run();
}
