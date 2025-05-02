use std::path::Path;
use std::sync::{Arc, RwLock};
use log::Log;
use solana_program_test::{find_file, ProgramTest};
use crate::utils::program_input::ProgramInput;

struct OutputLogger {
    output: Arc<RwLock<Vec<String>>>
}

impl log::Log for OutputLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() == log::Level::Debug && metadata.target() == "solana_runtime::message_processor::stable_log"
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.output.write().unwrap().push(format!("{}", record.args()));
        }
    }

    fn flush(&self) {}
}

pub fn set_output_logger() -> Result<Arc<RwLock<Vec<String>>>, Box<dyn std::error::Error>> {
    let output_logger = OutputLogger { output: Arc::new(RwLock::new(Vec::new())) };
    let output_clone = Arc::clone(&output_logger.output);
    let logger = Box::new(output_logger);
    log::set_boxed_logger(logger)?;
    log::set_max_level(log::LevelFilter::Debug);

    Ok(output_clone)
}

pub async fn generate_program_output(
    program_dir: &Path,
    program_name: &str,
    input: ProgramInput,
    output_log: Arc<RwLock<Vec<String>>>
) -> Result<Vec<String>, Box<dyn std::error::Error>> {

    std::env::set_var("BPF_OUT_DIR", program_dir.to_str().unwrap());
    let program_so_filename = format!("{program_name}.so");
    if !find_file(&program_so_filename).is_some() {
        Err(format!("No shared object {program_so_filename} found"))?;
    }

    let mut program_test = ProgramTest::default();
    program_test.set_compute_max_units(std::i64::MAX as u64);

    let program_name_static: &'static str = program_name.to_string().leak();
    program_test.add_program(program_name_static, input.program_id, None);
    for (pubkey, account) in input.accounts {
        program_test.add_account(pubkey, account);
    }

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    let mut transaction = input.transaction;
    transaction.sign(&input.keypairs, recent_blockhash);
    let tx_result = banks_client.process_transaction(transaction).await?;
    //dbg!(&tx_result);

    let output = output_log.read().unwrap().clone();
    Ok(output)
}