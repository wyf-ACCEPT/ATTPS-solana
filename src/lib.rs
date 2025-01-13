mod entrypoint;
mod instruction;
mod processor;
mod state;
#[cfg(test)]
mod test {
    pub mod state_test;
    pub mod instruction_test;
}
