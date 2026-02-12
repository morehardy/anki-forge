use crate::AnkiForgeError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContractReport {
    pub checks: usize,
    pub failures: usize,
}

pub fn run_contract_checks() -> Result<ContractReport, AnkiForgeError> {
    Ok(ContractReport::default())
}
