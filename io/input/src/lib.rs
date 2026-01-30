use std::io::StdinLock;

pub struct FastIn<'a> {
    stdin_lock: StdinLock<'a>,
}
