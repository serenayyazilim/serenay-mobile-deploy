use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

pub struct ExecOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

fn combined_message(output: &ExecOutput) -> String {
    [output.stdout.as_str(), output.stderr.as_str()].concat()
}

fn is_rate_limit(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("429") || lower.contains("quota exceeded") || lower.contains("rate_limit_exceeded") || lower.contains("resource_exhausted")
}

async fn run_shell(command: &str) -> Result<ExecOutput, String> {
    let output = Command::new("sh").arg("-c").arg(command).output().await.map_err(|e| e.to_string())?;
    Ok(ExecOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    })
}

/// Runs a `firebase` CLI call, retrying on 429/quota errors (up to 3 attempts, 20s apart).
pub async fn exec_with_retry(command: &str, max_retries: u32, retry_delay: Duration) -> Result<ExecOutput, String> {
    let mut last_output: Option<ExecOutput> = None;
    for attempt in 1..=max_retries {
        let output = run_shell(command).await?;
        if output.success {
            return Ok(output);
        }
        let msg = combined_message(&output);
        if is_rate_limit(&msg) && attempt < max_retries {
            sleep(retry_delay).await;
            last_output = Some(output);
            continue;
        }
        return Ok(output);
    }
    last_output.ok_or_else(|| "Max retries exceeded".to_string())
}

pub async fn exec(command: &str) -> Result<ExecOutput, String> {
    run_shell(command).await
}
