use std::{
    fs::File,
    path::PathBuf,
    process::{Command, Stdio},
};

use fluvio::config::TlsPolicy;
use fluvio_command::CommandExt;
use tracing::info;

use super::{FluvioLocalProcess, LocalRuntimeError};

#[derive(Debug)]
pub struct ScProcess {
    pub log_dir: PathBuf,
    pub launcher: Option<PathBuf>,
    pub tls_policy: TlsPolicy,
    pub rust_log: String,
    pub mode: ScMode,
}

#[derive(Debug)]
pub enum ScMode {
    Local(PathBuf),
    ReadOnly(PathBuf),
    K8s,
}

impl FluvioLocalProcess for ScProcess {}

impl ScProcess {
    pub fn start(&self) -> Result<(), LocalRuntimeError> {
        let outputs = File::create(format!("{}/flv_sc.log", self.log_dir.display()))?;
        let errors = outputs.try_clone()?;

        let launcher = self.launcher.clone();
        let mut binary = {
            let base = launcher.ok_or(LocalRuntimeError::MissingFluvioRunner)?;
            let mut cmd = Command::new(base);
            cmd.arg("run").arg("sc");
            cmd
        };

        match &self.mode {
            ScMode::Local(path) => {
                binary.arg("--local").arg(path);
            }
            ScMode::ReadOnly(path) => {
                binary.arg("--read-only").arg(path);
            }
            ScMode::K8s => {
                binary.arg("--k8");
            }
        };

        if let TlsPolicy::Verified(tls) = &self.tls_policy {
            self.set_server_tls(&mut binary, tls, 9005)?;
        }
        binary.env("RUST_LOG", &self.rust_log);

        info!(cmd = %binary.display(),"Invoking command");
        binary
            .stdout(Stdio::from(outputs))
            .stderr(Stdio::from(errors))
            .spawn()?;

        Ok(())
    }
}
