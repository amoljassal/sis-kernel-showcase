use bootloader::{BiosBoot, BootConfig};
use std::{env, fs, path::PathBuf, time::SystemTime};

fn main() {
    // OUT_DIR is per-target; keep artifacts under project /out for QEMU scripts
    let out_dir = PathBuf::from("out");
    fs::create_dir_all(&out_dir).ok();

    let kernel_elf = artifact_path("target", "x86_64-unknown-none", "debug", "sis_kernel");
    let bios_img = out_dir.join("boot-bios.img");
    let stamp = out_dir.join("boot-bios.stamp");

    // Rebuild policy:
    // 1) FORCE_BOOTIMG=1 always rebuilds
    // 2) If kernel ELF is newer than boot image, rebuild
    // 3) Otherwise reuse cached image (huge speedup)
    let force = env::var("FORCE_BOOTIMG")
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false);
    let need_rebuild = force || newer(&kernel_elf, &bios_img).unwrap_or(true);

    // Check if kernel ELF exists before trying to create image
    if !kernel_elf.exists() {
        println!("cargo:warning=boot.rs: kernel ELF not found, skipping image creation");
        return;
    }

    if need_rebuild {
        println!("cargo:warning=boot.rs: (re)building BIOS disk image …");
        let mut config = BootConfig::default();
        // Keep logging minimal and avoid framebuffer; we rely on serial
        config.serial_logging = true;
        config.frame_buffer_logging = false;
        // BIOS only image
        let mut bios = BiosBoot::new(&kernel_elf);
        // NOTE: set_boot_config returns &mut BiosBoot in 0.11.11, chain safely
        let bios = bios.set_boot_config(&config);
        bios.create_disk_image(&bios_img)
            .expect("bootloader BIOS image build failed");
        // write/refresh stamp
        let _ = fs::write(&stamp, b"ok");
    } else {
        println!("cargo:warning=boot.rs: reusing cached BIOS image");
    }

    println!("cargo:rustc-env=SIS_BOOT_BIOS_IMG={}", bios_img.display());
}

fn artifact_path(root: &str, triple: &str, profile: &str, name: &str) -> PathBuf {
    PathBuf::from(root).join(triple).join(profile).join(name)
}

fn newer(a: &PathBuf, b: &PathBuf) -> std::io::Result<bool> {
    let ma = fs::metadata(a)?;
    let mb = fs::metadata(b).ok();
    if mb.is_none() {
        return Ok(true);
    }
    let ta = ma.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let tb = mb.unwrap().modified().unwrap_or(SystemTime::UNIX_EPOCH);
    Ok(ta > tb)
}
