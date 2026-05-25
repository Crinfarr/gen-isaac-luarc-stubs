mod json_to_enum;

use std::{
    fs::File,
    io::{Result, Write},
    path::Path,
    process::Stdio,
};

use tracing::{debug, error, info, warn};
use tracing_subscriber::util::SubscriberInitExt;
use typed_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::FmtSubscriber::new().init();
    info!("Testing git availability");
    // git -v is the fastest possible git command
    if let Err(git_run_err) = tokio::process::Command::new("git")
        .arg("-v")
        .stdout(Stdio::null())
        .spawn()?
        .wait()
        .await
    {
        error!("Git must be in your $PATH to run this");
        return Err(std::io::Error::other(format!(
            "Failed to run git: {git_run_err:?}"
        )));
    } else {
        info!("Git is available");
    };
    //clone extension into tmp directory
    info!("Cloning isaac-lua-api-vscode");
    if std::fs::exists(".ext_repo.tmp")? {
        warn!("Clearing existing .ext_repo.tmp");
        if let Err(e) = std::fs::remove_dir_all(".ext_repo.tmp") {
            error!("Failed to remove existing .ext_repo.tmp: {e:?}");
            return Err(std::io::Error::other(
                "repo clone already exists and cannot be regenerated",
            ));
        };
    }
    if let Err(clone_error) = tokio::process::Command::new("git")
        .args(vec![
            "clone",
            "https://github.com/filloax/isaac-lua-api-vscode",
            ".ext_repo.tmp",
        ])
        .stderr(Stdio::null())
        .spawn()?
        .wait()
        .await
    {
        error!("Failed to clone filloax/isaac-lua-api-vscode over https");
        return Err(std::io::Error::other(format!(
            "Failed to clone repo: {clone_error:?}"
        )));
    } else {
        info!("Cloned repo into .ext_repo.tmp")
    };
    //create lib dir
    let lib_dir = format!(
        "{}/isaac_luarc",
        dirs::data_dir()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find user data directory"
            ))?
            .to_string_lossy()
    );
    if std::fs::exists(&lib_dir)? {
        warn!("lib already exists, regenerating");
        if std::fs::remove_dir_all(&lib_dir).is_err() || std::fs::create_dir(&lib_dir).is_err() {
            error!("Failed to remove existing lib directory");
            return Err(std::io::Error::other(
                "lib directory exists and cannot be regenerated",
            ));
        }
    } else {
        std::fs::create_dir(&lib_dir)?;
    }
    let mut lib_paths: Vec<String> = Vec::default();
    //switch which dirs need copying based on if repentagon is repentagoff
    let use_repentagon = !(std::env::var("NO_REPENTAGON").is_ok()
        || std::env::var("REPENTAGONE").is_ok()
        || std::env::var("REPENTAGOFF").is_ok());
    let required_dirs = if !use_repentagon {
        vec!["vanilla", "no_repentogon_only"]
    } else {
        vec!["vanilla", "repentogon_changes", "repentogon_new"]
    };
    for dir in required_dirs {
        let sourcepath = format!(".ext_repo.tmp/src/docs/{dir}");
        let targetpath = format!("{lib_dir}/{dir}");
        info!("Moving {dir} into {lib_dir}/{dir}");
        if let Err(e) = std::fs::rename(Path::new(&sourcepath), Path::new(&targetpath)) {
            error!("Failed to move required stub directory: {e:?}");
            return Err(e);
        };
        lib_paths.push(
            // there's a chance this fails if a user somehow has invalid unicode in a parent dir
            // but i choose to not fix that because i don't understand how that could possibly happen
            Path::new(&targetpath)
                .canonicalize()?
                .to_string_lossy()
                .to_string(),
        );
    }
    info!("Generating enums");
    let required_enums = if !use_repentagon {
        vec!["vanilla.json"]
    } else {
        vec!["vanilla.json", "repentogon.json"]
    };
    if !std::fs::exists(&format!("{lib_dir}/enums"))? {
        info!("Creating library folder");
        std::fs::create_dir(format!("{lib_dir}/enums"))?
    };
    for enum_filename in required_enums {
        //                                              unwrap is safe, all splits have at least 1 element
        let ofilename = format!(
            "{lib_dir}/enums/{}.lua",
            enum_filename.split(".").nth(0).unwrap()
        );
        info!("Generating {ofilename}");
        let file_path = Path::new(&ofilename);
        let enum_content = json_to_enum::decode_enum_file(Path::new(&format!(
            ".ext_repo.tmp/src/docs/enums/{enum_filename}"
        )))?;
        let lua_content = json_to_enum::to_lua(&enum_content)?;

        match File::create(&file_path) {
            Ok(mut f) => {
                if let Err(e) = f.write(lua_content.as_bytes()) {
                    error!("Failed to write data to {ofilename}: {e}");
                    return Err(e);
                }
            }
            Err(e) => {
                error!("Failed to open {ofilename} to write: {e}");
                return Err(e);
            }
        }
    }
    lib_paths.push(
        Path::new(&format!("{lib_dir}/enums"))
            .canonicalize()?
            .to_string_lossy()
            .to_string(),
    );
    info!("Generating .luarc.json in {lib_dir}");
    let luarc_obj = json!({
        "$schema": "https://raw.githubusercontent.com/LuaLS/vscode-lua/master/setting/schema.json",
        "runtime": {
          "version": "Lua 5.3"
        },
        "workspace": {
          "library": lib_paths
        }
    });
    std::fs::write(format!("{lib_dir}/.luarc.json"), luarc_obj.to_string())?;
    info!("Generating symlink in current dir");

    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_file(
        Path::new(&format!("{lib_dir}/.luarc.json")),
        ".luarc.json",
    )?;
    #[cfg(not(target_os = "windows"))]
    std::os::unix::fs::symlink(Path::new(&format!("{lib_dir}/.luarc.json")), ".luarc.json")?;

    info!("Cleaning up work dir");
    if std::fs::remove_dir_all(".ext_repo.tmp").is_err() {
        error!("Failed to remove .ext_repo.tmp; you might have to manually delete it");
    };
    Ok(())
}
