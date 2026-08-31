use minisign::{SecretKeyBox, sign};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::fs;
use std::io::Cursor;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = Path::new("d:/Devs/project_silo/attempt_VI/silo/silo/~/.tauri/silo.key");
    let installer_path = Path::new("d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/target/release/bundle/nsis/SILO_1.1.0_x64-setup.exe");

    println!("Reading private key from {:?}", key_path);
    let key_file_content = fs::read_to_string(key_path)?;
    
    let cleaned_content = key_file_content.trim();
    println!("Base64 decoding private key...");
    let decoded_bytes = STANDARD.decode(cleaned_content)?;
    let key_str = String::from_utf8(decoded_bytes)?;

    println!("Parsing private key...");
    let sk_box = SecretKeyBox::from_string(&key_str)?;
    
    let passwords = vec![
        "".to_string(),
        "silo".to_string(),
        "YaraAstro".to_string(),
        "yara".to_string(),
        "password".to_string(),
        "admin".to_string(),
    ];

    let mut decrypted_sk = None;
    for (i, pwd) in passwords.iter().enumerate() {
        println!("Attempting decryption candidate #{} ({:?})...", i, pwd);
        match sk_box.clone().into_secret_key(Some(pwd.clone())) {
            Ok(sk) => {
                println!("Decryption succeeded with candidate #{}!", i);
                decrypted_sk = Some(sk);
                break;
            }
            Err(e) => {
                println!("Candidate #{} failed: {:?}", i, e);
            }
        }
    }

    let sk = match decrypted_sk {
        Some(sk) => sk,
        None => {
            return Err("All password candidates failed.".into());
        }
    };

    println!("Reading installer file from {:?}", installer_path);
    let installer_data = fs::read(installer_path)?;
    let reader = Cursor::new(installer_data);

    println!("Signing installer...");
    let sig_box = sign(None, &sk, reader, None, None)?;

    println!("Signature successfully generated:\n{}", sig_box.to_string());
    Ok(())
}
