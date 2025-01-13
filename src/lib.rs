mod entrypoint;
mod instruction;
mod processor;
mod state;
#[cfg(test)]
mod test {
    pub mod instruction_test;
    pub mod state_test;
}
