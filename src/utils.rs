use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::Command;

#[derive(Default, Debug)]
pub struct LedgerAppInfos {
    pub api_level: u8,
    pub size: u64,
}

pub fn retrieve_infos(
    file: &std::path::Path,
) -> Result<LedgerAppInfos, io::Error> {
    let buffer = fs::read(file)?;
    let elf = goblin::elf::Elf::parse(&buffer).unwrap();

    let mut infos = LedgerAppInfos::default();

    // All infos coming from the SDK are expected to be regrouped
    // in various `.ledger.<field_name>` section of the binary.
    // For now we only need the API_LEVEL
    for section in elf.section_headers.iter() {
        if let Some(Ok(".ledger.api_level")) =
            elf.shdr_strtab.get(section.sh_name)
        {
            infos.api_level = buffer[section.sh_offset as usize];
        }
    }

    let mut nvram_data = 0;
    let mut envram_data = 0;
    for s in elf.syms.iter() {
        let symbol_name = elf.strtab.get(s.st_name);
        let name = symbol_name.unwrap().unwrap();
        match name {
            "_nvram_data" => nvram_data = s.st_value,
            "_envram_data" => envram_data = s.st_value,
            _ => (),
        }
    }
    infos.size = envram_data - nvram_data;
    Ok(infos)
}

pub fn export_binary(elf_path: &std::path::Path, dest_bin: &std::path::Path) {
    let objcopy = env::var_os("CARGO_TARGET_THUMBV6M_NONE_EABI_OBJCOPY")
        .unwrap_or_else(|| "arm-none-eabi-objcopy".into());

    Command::new(objcopy)
        .arg(elf_path)
        .arg(dest_bin)
        .args(["-O", "ihex"])
        .output()
        .expect("Objcopy failed");

    let size = env::var_os("CARGO_TARGET_THUMBV6M_NONE_EABI_SIZE")
        .unwrap_or_else(|| "arm-none-eabi-size".into());

    // print some size info while we're here
    let out = Command::new(size)
        .arg(elf_path)
        .output()
        .expect("Size failed");

    io::stdout().write_all(&out.stdout).unwrap();
    io::stderr().write_all(&out.stderr).unwrap();
}

pub fn install_with_ledgerctl(
    dir: &std::path::Path,
    app_json: &std::path::Path,
) {
    let out = Command::new("ledgerctl")
        .current_dir(dir)
        .args(["install", "-f", app_json.to_str().unwrap()])
        .output()
        .expect("fail");

    io::stdout().write_all(&out.stdout).unwrap();
    io::stderr().write_all(&out.stderr).unwrap();
}
