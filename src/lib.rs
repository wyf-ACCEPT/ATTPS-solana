mod constants;
mod entrypoint;
mod error;
mod instruction;
mod processor;
pub mod state;
mod utils;

#[cfg(test)]
mod test {
    pub mod instruction_test;
    pub mod state_test;
    pub mod utils_agent_test;
    pub mod utils_manager_test;
}
