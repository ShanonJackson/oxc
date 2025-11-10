#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend { Scalar, Avx2, Avx512 }

fn env_override() -> Option<Backend> {
    if let Ok(v) = std::env::var("METAL_BACKEND") {
        match v.to_ascii_lowercase().as_str() {
            "scalar" => return Some(Backend::Scalar),
            "avx2" => return Some(Backend::Avx2),
            "avx512" | "avx-512" => return Some(Backend::Avx512),
            _ => {}
        }
    }
    None
}

pub fn detect() -> Backend {
    if let Some(b) = env_override() { return b; }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if std::arch::is_x86_feature_detected!("avx512f") {
            return Backend::Avx512;
        }
        if std::arch::is_x86_feature_detected!("avx2") {
            return Backend::Avx2;
        }
    }
    Backend::Scalar
}

pub fn name(b: Backend) -> &'static str {
    match b { Backend::Scalar => "scalar", Backend::Avx2 => "avx2", Backend::Avx512 => "avx512" }
}

