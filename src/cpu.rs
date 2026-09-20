//! CPU algilama: mantiksal thread sayisi + islemci model adi.
//!
//! Amaç: paralel isleme ve thread dagilimini makineye göre otomatik ayarlamak.
//! ör: 4C/8T i5-4590 → 4 paralel is × 2 thread/iş
//!     28T i7-14700F → 8 paralel is × 3-4 thread/iş
//!
//! Model adi yalnizca GÖRÜNÜM icin kullanilir; karar core/thread sayisindan
//! cikar (model string'ine bakmak kırılgan olurdu).

pub struct CpuInfo {
    /// ör: "Intel(R) Core(TM) i7-14700F CPU @ 2.10GHz" (okunamadiysa bos)
    pub model: String,
    /// Fiziksel cekirdek (Linux'ta /proc/cpuinfo'dan; Windows'ta None)
    pub physical: Option<u32>,
    /// Mantiksal thread (HyperThreading dahil)
    pub logical: u32,
}

impl CpuInfo {
    pub fn detect() -> CpuInfo {
        let logical = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);
        let (model, physical) = raw_info();
        CpuInfo {
            model,
            physical,
            logical,
        }
    }

    /// Otomatik paralel is (worker) sayisi:
    /// mantiksal thread'in yarisi, 2..=8 arasina kilitli.
    pub fn auto_workers(&self) -> usize {
        ((self.logical as usize) / 2).clamp(2, 8)
    }
}

// ---------------------------------------------------------------------------
// Platform ozel okuma
// ---------------------------------------------------------------------------

#[cfg(not(windows))]
fn raw_info() -> (String, Option<u32>) {
    // Linux (ve benzeri): /proc/cpuinfo
    let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") else {
        return (String::new(), None);
    };
    let mut model = String::new();
    let mut cores: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    let mut phys = String::new();
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("model name") {
            if let Some((_, val)) = v.split_once(':') {
                if model.is_empty() {
                    model = val.trim().to_string();
                }
            }
        } else if let Some(v) = line.strip_prefix("physical id") {
            if let Some((_, val)) = v.split_once(':') {
                phys = val.trim().to_string();
            }
        } else if let Some(v) = line.strip_prefix("core id") {
            if let Some((_, val)) = v.split_once(':') {
                cores.insert((phys.clone(), val.trim().to_string()));
            }
        }
    }
    (model, (!cores.is_empty()).then(|| cores.len() as u32))
}

#[cfg(windows)]
fn raw_info() -> (String, Option<u32>) {
    // CentralProcessor anahtarinin "Description" seviyesi WINDOWS DILINE GORE
    // lokalizedir (Açıklama = TR, Beschreibung = DE) — sirayla dene.
    const PATHS: [&str; 3] = [
        r"Hardware\Description\System\CentralProcessor\0",
        r"Hardware\Açıklama\System\CentralProcessor\0",
        r"Hardware\Beschreibung\System\CentralProcessor\0",
    ];
    for p in PATHS {
        if let Some(m) = winreg_sz(p, "ProcessorNameString") {
            return (m, None);
        }
    }
    (String::new(), None)
}

#[cfg(windows)]
fn winreg_sz(subkey: &str, value: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RegGetValueW};
    const RRF_RT_REG_SZ: u32 = 0x0002_0000;

    let to_w = |s: &str| -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    };
    let sub = to_w(subkey);
    let val = to_w(value);

    let mut size: u32 = 0;
    // 1) buyuklugu sor
    if unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            sub.as_ptr(),
            val.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut size,
        )
    } != 0 ||
        size == 0
    {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    if unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            sub.as_ptr(),
            val.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut _,
            &mut size,
        )
    } != 0
    {
        return None;
    }
    let wide: Vec<u16> = buf
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    Some(String::from_utf16_lossy(&wide[..end]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_workers_sinirlari() {
        // 8 thread (i5-4590) -> 4 paralel
        let c = CpuInfo {
            model: "i5-4590".into(),
            physical: Some(4),
            logical: 8,
        };
        assert_eq!(c.auto_workers(), 4);
        // 28 thread (i7-14700F) -> 8 (ust sinir)
        let c = CpuInfo {
            model: "i7-14700F".into(),
            physical: Some(20),
            logical: 28,
        };
        assert_eq!(c.auto_workers(), 8);
        // 2 thread (eski laptop) -> 2 (alt sinir)
        let c = CpuInfo {
            model: "old".into(),
            physical: None,
            logical: 2,
        };
        assert_eq!(c.auto_workers(), 2);
    }
}
