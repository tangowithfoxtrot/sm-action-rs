use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;

use anyhow::Result;
use bitwarden_core::auth::login::AccessTokenLoginRequest;
use bitwarden_core::{Client, ClientSettings};
use bitwarden_sm::ClientSecretsExt;
use bitwarden_sm::secrets::SecretsGetRequest;

use config::{Config, infer_urls};
use uuid::Uuid;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;
    let (api_url, identity_url) = infer_urls(&config)?;

    let client = Client::new(Some(ClientSettings {
        identity_url,
        api_url,
        user_agent: "bitwarden/sm-action".to_string(),
        device_type: bitwarden_core::DeviceType::SDK,
    }));

    client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token: config.access_token,
            state_file: None,
        })
        .await?;

    let id_to_name_map = parse_secret_input(config.secrets);

    let secret_ids: Vec<Uuid> = id_to_name_map.keys().cloned().collect();

    let secrets = client
        .secrets()
        .get_by_ids(SecretsGetRequest { ids: secret_ids })
        .await?;

    for secret in secrets.data.iter() {
        id_to_name_map
            .get(&secret.id)
            .map(|name| set_secrets(name, &secret.value))
            .transpose()?;
    }

    Ok(())
}

/// Parses the secret input from the GitHub Actions environment variable.
fn parse_secret_input(secret_lines: Vec<String>) -> HashMap<Uuid, String> {
    let mut map: HashMap<Uuid, String> = HashMap::with_capacity(secret_lines.capacity());

    for line in secret_lines.iter() {
        let uuid_part = line.split('>').next().unwrap_or_default().trim();
        let uuid_parse_result = Uuid::from_str(uuid_part);

        if let Err(err) = uuid_parse_result {
            eprintln!("Warning: Invalid UUID format: {err}");
            continue;
        }

        let desired_name = line.split('>').nth(1).unwrap_or_default().trim();

        let uuid = uuid_parse_result.expect(
            "Error has already been checked, and if it is an error it should not reach here.",
        );

        if let Some(old_value) = map.insert(uuid, desired_name.to_string()) {
            eprintln!(
                "Warning: Duplicate UUID found: {uuid}. Old value: {old_value}, New value: {desired_name}"
            );
        }
    }

    map
}

/// Masks a value in the GitHub Actions logs to prevent it from being displayed.
fn mask_value(value: &str) {
    println!("::add-mask::{}", value);
}

fn issue_file_command(mut file: std::fs::File, key: &str, value: &str) -> Result<()> {
    let delimiter = format!("ghadelimiter_{}", uuid::Uuid::new_v4());
    file.write_fmt(format_args!("{key}<<{delimiter}\n{value}\n{delimiter}\n"))?;
    Ok(())
}

/// Sets a secret in the GitHub Actions environment.
fn set_secrets(secret_name: &str, secret_value: &str) -> Result<()> {
    let binding = github_escape(secret_value);
    let escaped_secret = binding.as_str();
    mask_value(escaped_secret); // ensure the value is masked in the logs

    let env_path = std::env::var("GITHUB_ENV")?;
    let output_path = std::env::var("GITHUB_OUTPUT")?;

    let env_file = OpenOptions::new()
        .create(true) // needed for unit tests
        .append(true)
        .open(env_path)?;

    issue_file_command(env_file, secret_name, secret_value)?;

    let output_file = OpenOptions::new()
        .create(true) // needed for unit tests
        .append(true)
        .open(output_path)?;

    issue_file_command(output_file, secret_name, secret_value)?;

    // writeln!(
    //     env_file,
    //     "{secret_name}<<{delimiter}\n{secret_value}\n{delimiter}"
    // )
    // .map_err(|e| {
    //     eprintln!("Error writing to {env_file:?}: {}", e);
    //     e
    // })?;

    // writeln!(
    //     output_file,
    //     "{secret_name}<<{delimiter}\n{secret_value}\n{delimiter}",
    // )
    // .map_err(|e| {
    //     eprintln!("Error writing to {output_file:?}: {}", e);
    //     e
    // })?;

    Ok(())
}

fn github_escape(secret_value: &str) -> String {
    // secret_value.escape_debug().to_string()
    secret_value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_secrets() {
        let secret_name = "TEST_SECRET";
        let secret_value = r#"BrowserSettings__EnvironmentUrl=https://example.com

# Browser Settings 2
BrowserSettings__EnvironmentUrl=https://example2.com"#;

        // Temporarily set the environment variables for testing
        unsafe {
            std::env::set_var("GITHUB_ENV", "/tmp/test_env");
            std::env::set_var("GITHUB_OUTPUT", "/tmp/test_output");
        };

        // Ensure the file does not exist before the test
        let env_path = std::env::var("GITHUB_ENV").unwrap();
        let output_path = std::env::var("GITHUB_OUTPUT").unwrap();

        let result = set_secrets(secret_name, secret_value);

        // Improved error reporting
        assert!(result.is_ok(), "set_secrets failed: {:?}", result);

        // Check if the file was created and contains the expected value
        let env_content = std::fs::read_to_string(&env_path).unwrap();
        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(env_content.contains(&format!("{}={}", secret_name, github_escape(secret_value))));
        assert!(output_content.contains(&format!(
            "{}={}",
            secret_name,
            secret_value.escape_debug()
        )));
    }

    #[test]
    fn test_github_escape() {
        github_escape_with_special_chars(
            "percent % percent % cr \r cr \r lf \n lf \n",
            "percent %25 percent %25 cr %0D cr %0D lf %0A lf %0A",
        );
        github_escape_with_special_chars(
            "%25 %25 %0D %0D %0A %0A %3A %3A %2C %2C",
            "%2525 %2525 %250D %250D %250A %250A %253A %253A %252C %252C",
        );
        github_escape_with_special_chars("normal text", "normal text");
    }

    fn github_escape_with_special_chars(input: &str, expected: &str) {
        assert_eq!(github_escape(input), expected)
    }

    #[test]
    fn test_parse_secret_lines() {
        let id_to_name_map = parse_secret_input(vec![
            "91ba3f10-a9a2-4795-bacf-0eee2d39a074 > ONE".to_string(),
            "bfd7aa33-54f2-487b-bbbf-4a69b49fdc0d > TWO".to_string(),
        ]);

        assert_eq!(id_to_name_map.len(), 2);
        assert_eq!(
            id_to_name_map.get(&Uuid::from_str("91ba3f10-a9a2-4795-bacf-0eee2d39a074").unwrap()),
            Some(&"ONE".to_string())
        );

        assert_eq!(
            id_to_name_map.get(&Uuid::from_str("bfd7aa33-54f2-487b-bbbf-4a69b49fdc0d").unwrap()),
            Some(&"TWO".to_string())
        );
    }

    #[test]
    fn test_parse_secret_lines_two() {
        let id_to_name_map = parse_secret_input(vec![
            "91ba3f10-a9a2-4795-bacf-0eee2d39a074 > ONE".to_string(),
            "91ba3f10-a9a2-4795-bacf-0eee2d39a074 > TWO".to_string(),
        ]);

        assert_eq!(id_to_name_map.len(), 2);
        assert_eq!(
            id_to_name_map.get(&Uuid::from_str("91ba3f10-a9a2-4795-bacf-0eee2d39a074").unwrap()),
            Some(&"ONE".to_string())
        );

        assert_eq!(
            id_to_name_map.get(&Uuid::from_str("91ba3f10-a9a2-4795-bacf-0eee2d39a074").unwrap()),
            Some(&"TWO".to_string())
        );
    }
}
