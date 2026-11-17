//! Plan `two-tab-speech`: which built-in Whisper models this Mac runs well.
//!
//! [`detect`] reads the chip, the memory and the free disk space; [`offer_models`] applies the
//! table in `docs/plans/2026-09-25-two-tab-speech/layout.md` to decide which models the speech
//! screens offer, and why the larger one is left out. The rule is a pure function so it is
//! unit-tested.

use std::path::Path;

use serde::Serialize;

use super::models::{self, KnownModel};

/// Id of the larger, more accurate model ("Best accuracy").
pub const BEST_MODEL_ID: &str = "large-v3-turbo";
/// Id of the smaller model ("Faster").
pub const FASTER_MODEL_ID: &str = "small";

/// The larger model needs at least this much memory (8 GB).
pub const MIN_MEMORY_FOR_BEST: u64 = 8 * 1024 * 1024 * 1024;
/// Dedicated DXGI VRAM thresholds for Windows Vulkan GPUs. Real 4/6/8 GB GDDR cards
/// typically reserve ~100-350 MiB for VBIOS and driver allocations before reporting
/// `DedicatedVideoMemory`, so each tier gives ~500 MiB of headroom.
pub const MIN_VRAM_4GB: u64 = 3_500 * 1024 * 1024;
pub const MIN_VRAM_6GB: u64 = 5_500 * 1024 * 1024;
pub const MIN_VRAM_8GB: u64 = 7_500 * 1024 * 1024;
const MIN_DISCRETE_VRAM_FOR_VULKAN_CLASSIFICATION: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChipKind {
    /// An Apple M-series chip. Metal makes whisper.cpp fast on it.
    AppleSilicon,
    /// An Intel Mac. It has no Apple GPU that whisper.cpp can use well.
    Intel,
    /// A Windows PC with a Vulkan-capable discrete GPU.
    WindowsGpu,
    /// A Windows PC without a usable discrete GPU (integrated graphics, software renderer, or CPU only).
    WindowsCpu,
    /// The check could not tell (for example on another operating system).
    Unknown,
}

/// What [`detect`] found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub chip_kind: ChipKind,
    /// For example "Apple M1 Pro", "NVIDIA GeForce RTX 4060", or "Intel(R) Core(TM) i7-9750H CPU @ 2.60GHz". Can be empty.
    pub chip_name: String,
    /// Installed memory in bytes; 0 when unknown.
    pub memory_bytes: u64,
    /// Free bytes on the disk that holds the models folder.
    pub free_bytes: u64,
    /// Windows only: DXGI adapter description (`None` on macOS/Linux to preserve JSON shape).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_name: Option<String>,
    /// Windows only: DXGI `DedicatedVideoMemory` in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vram_bytes: Option<u64>,
    /// Windows only: DXGI `SharedSystemMemory` in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_gpu_memory_bytes: Option<u64>,
}

/// Why the larger model is not offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeftOutReason {
    NeedsAppleSilicon,
    NeedsGpu,
    NeedsVram,
    NeedsMemory,
    NeedsDiskSpace,
}

/// One model the speech screens show as an option card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferedModel {
    pub id: String,
    pub size_bytes: u64,
    /// The first card when there are two; it is selected by default.
    pub recommended: bool,
}

/// The models to offer, in display order, and what the hardware note says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelOffer {
    pub models: Vec<OfferedModel>,
    /// Set when the larger model is left out but another one is offered.
    pub left_out: Option<LeftOutReason>,
    /// Set when no model fits on the disk: the free space the smallest model needs.
    pub needed_bytes: Option<u64>,
}

/// The result of the `get_speech_hardware` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareCheck {
    pub hardware: Hardware,
    pub offer: ModelOffer,
}

/// What one model needs from the hardware. Speech and AI (plan `ai-polish-setup`) each have a list
/// of these, largest model first.
#[derive(Debug, Clone, Copy)]
pub struct ModelRule {
    pub model: KnownModel,
    /// Requires Apple Silicon (`AppleSilicon`) or a discrete Vulkan GPU (`WindowsGpu`).
    pub needs_apple_silicon: bool,
    /// Installed unified/system memory the model needs on macOS (0: no minimum).
    pub min_memory_bytes: u64,
    /// Dedicated GPU VRAM the model needs on Windows (`ChipKind::WindowsGpu`; 0: no minimum).
    pub min_vram_bytes: u64,
}

/// Applies the table of layout.md. A model that is already installed needs no free space.
///
/// | Condition | Best accuracy | Faster |
/// |---|---|---|
/// | Apple Silicon, ≥ 8 GB memory, free disk ≥ 1.1 × size | offered, recommended | offered |
/// | Apple Silicon < 8 GB, or Intel | not offered | offered |
/// | Free disk < 1.1 × large but ≥ 1.1 × small | not offered | offered |
/// | Not enough disk for either | none; say how much space is needed | |
pub fn offer_models(hardware: &Hardware, installed_ids: &[String]) -> ModelOffer {
    let (Some(best), Some(faster)) = (
        models::known_model(BEST_MODEL_ID),
        models::known_model(FASTER_MODEL_ID),
    ) else {
        return offer_by_rules(hardware, &[], installed_ids);
    };
    let mut rules = vec![
        ModelRule {
            model: *best,
            needs_apple_silicon: true,
            min_memory_bytes: MIN_MEMORY_FOR_BEST,
            min_vram_bytes: MIN_VRAM_4GB,
        },
        ModelRule {
            model: *faster,
            needs_apple_silicon: false,
            min_memory_bytes: 0,
            min_vram_bytes: 0,
        },
    ];
    // Plan `language-model-suggestions`: the `llama-server` models come last, so the Whisper
    // models keep their places (and the recommendation) and `left_out` still explains Whisper.
    rules.extend(models::ASR_MODELS.iter().map(|model| ModelRule {
        model: model.as_known_model(),
        needs_apple_silicon: true,
        min_memory_bytes: model.min_memory_bytes,
        min_vram_bytes: model.min_vram_bytes,
    }));
    offer_by_rules(hardware, &rules, installed_ids)
}

/// The models of `rules` (largest first) this device is offered. Each needs its chip/GPU, VRAM
/// (on `ChipKind::WindowsGpu`) or unified memory (on macOS), and free disk space of 1.1 × its size
/// unless it is installed. With two offered the first is recommended. `left_out` says why the first
/// rule's model is missing; when none is offered, `needed_bytes` is set if only the disk is short,
/// otherwise `left_out` says why the least demanding model does not suit this device.
pub fn offer_by_rules(
    hardware: &Hardware,
    rules: &[ModelRule],
    installed_ids: &[String],
) -> ModelOffer {
    let chip_ok = |rule: &ModelRule| {
        !rule.needs_apple_silicon
            || matches!(
                hardware.chip_kind,
                ChipKind::AppleSilicon | ChipKind::WindowsGpu
            )
    };
    let vram_ok = |rule: &ModelRule| {
        hardware.chip_kind != ChipKind::WindowsGpu
            || rule.min_vram_bytes == 0
            || hardware.vram_bytes.unwrap_or(0) >= rule.min_vram_bytes
    };
    // On Windows with a discrete GPU, weights and KV cache reside in DXGI DedicatedVideoMemory,
    // while `GlobalMemoryStatusEx().ullTotalPhys` reports slightly under nominal RAM (e.g. 15.8 GiB
    // on a 16 GB PC), so gate discrete GPU models on `vram_ok` instead of macOS unified RAM tiers.
    let memory_ok = |rule: &ModelRule| {
        hardware.chip_kind == ChipKind::WindowsGpu || hardware.memory_bytes >= rule.min_memory_bytes
    };
    let fits = |rule: &ModelRule| {
        installed_ids.iter().any(|id| id == rule.model.id)
            || hardware.free_bytes >= models::needed_free_bytes(rule.model.size_bytes, 0)
    };
    let reason = |rule: &ModelRule| {
        if !chip_ok(rule) {
            if hardware.chip_kind == ChipKind::WindowsCpu {
                LeftOutReason::NeedsGpu
            } else {
                LeftOutReason::NeedsAppleSilicon
            }
        } else if !vram_ok(rule) {
            LeftOutReason::NeedsVram
        } else if !memory_ok(rule) {
            LeftOutReason::NeedsMemory
        } else {
            LeftOutReason::NeedsDiskSpace
        }
    };
    let offered: Vec<&ModelRule> = rules
        .iter()
        .filter(|rule| chip_ok(rule) && vram_ok(rule) && memory_ok(rule) && fits(rule))
        .collect();
    let count = offered.len();
    let models = offered
        .iter()
        .enumerate()
        .map(|(index, rule)| OfferedModel {
            id: rule.model.id.to_string(),
            size_bytes: rule.model.size_bytes,
            recommended: index == 0 && count > 1,
        })
        .collect::<Vec<_>>();

    if models.is_empty() {
        // The smallest model the chip, VRAM and memory allow; only the disk is short for it.
        let disk_short = rules
            .iter()
            .filter(|rule| chip_ok(rule) && vram_ok(rule) && memory_ok(rule))
            .min_by_key(|rule| rule.model.size_bytes);
        return match disk_short {
            Some(rule) => ModelOffer {
                models,
                left_out: None,
                needed_bytes: Some(models::needed_free_bytes(rule.model.size_bytes, 0)),
            },
            None => ModelOffer {
                models,
                left_out: rules.last().map(reason),
                needed_bytes: None,
            },
        };
    }
    let left_out = rules
        .first()
        .filter(|first| !offered.iter().any(|rule| rule.model.id == first.model.id))
        .map(reason);
    ModelOffer {
        models,
        left_out,
        needed_bytes: None,
    }
}

/// Summary of one DXGI display adapter on Windows, kept pure for cross-platform unit testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DxgiAdapter {
    pub name: String,
    pub vendor_id: u32,
    pub dedicated_video_memory: u64,
    pub shared_system_memory: u64,
    /// True when `DXGI_ADAPTER_FLAG_SOFTWARE` or `DXGI_ADAPTER_FLAG_REMOTE` is set.
    pub is_software: bool,
}

/// Selected Windows chip/GPU identity from DXGI adapters + CPU registry name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsChipInfo {
    pub chip_kind: ChipKind,
    pub chip_name: String,
    pub gpu_name: Option<String>,
    pub vram_bytes: Option<u64>,
    pub shared_gpu_memory_bytes: Option<u64>,
}

/// True for Microsoft Basic Render Driver (`0x1414`), software/remote DXGI adapters, or virtual
/// display drivers (Parsec, SudoMaker, Virtual Desktop, etc.) that do not execute compute.
pub fn is_software_adapter(adapter: &DxgiAdapter) -> bool {
    if adapter.is_software || adapter.vendor_id == 0x1414 {
        return true;
    }
    let lower = adapter.name.to_ascii_lowercase();
    [
        "microsoft basic render",
        "microsoft remote display",
        "parsec",
        "sudomaker",
        "virtual display",
        "virtual desktop",
        "virtual monitor",
        "indirect display",
        "iddsampledriver",
        "spacedesk",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

/// True for discrete GPUs (or high-VRAM carve-out devices) that can offload models via Vulkan.
/// Software adapters and typical low-VRAM laptop iGPUs return `false` (`ChipKind::WindowsCpu`),
/// while 2 GB discrete GPUs return `true` so offer rules can report `LeftOutReason::NeedsVram`.
pub fn is_discrete_vulkan_gpu(adapter: &DxgiAdapter) -> bool {
    if is_software_adapter(adapter) {
        return false;
    }
    if adapter.dedicated_video_memory >= MIN_VRAM_4GB {
        return true;
    }
    if adapter.dedicated_video_memory < MIN_DISCRETE_VRAM_FOR_VULKAN_CLASSIFICATION {
        return false;
    }
    let lower = adapter.name.to_ascii_lowercase();
    adapter.vendor_id == 0x10DE
        || lower.contains("radeon rx")
        || lower.contains("radeon pro")
        || lower.contains("arc(tm) a")
        || lower.contains("arc(tm) b")
}

/// Picks the best discrete GPU (if present) or primary hardware adapter on Windows.
pub fn select_windows_gpu(adapters: &[DxgiAdapter], cpu_name: &str) -> WindowsChipInfo {
    let cpu = cpu_name.trim();
    let hardware_adapters: Vec<&DxgiAdapter> = adapters
        .iter()
        .filter(|adapter| !is_software_adapter(adapter))
        .collect();
    if let Some(best) = hardware_adapters
        .iter()
        .copied()
        .filter(|adapter| is_discrete_vulkan_gpu(adapter))
        .max_by_key(|adapter| (adapter.dedicated_video_memory, adapter.shared_system_memory))
    {
        let gpu = best.name.trim().to_string();
        return WindowsChipInfo {
            chip_kind: ChipKind::WindowsGpu,
            chip_name: if gpu.is_empty() {
                cpu.to_string()
            } else {
                gpu.clone()
            },
            gpu_name: (!gpu.is_empty()).then_some(gpu),
            vram_bytes: Some(best.dedicated_video_memory),
            shared_gpu_memory_bytes: Some(best.shared_system_memory),
        };
    }
    if let Some(igpu) = hardware_adapters
        .into_iter()
        .max_by_key(|adapter| (adapter.dedicated_video_memory, adapter.shared_system_memory))
    {
        let gpu = igpu.name.trim().to_string();
        return WindowsChipInfo {
            chip_kind: ChipKind::WindowsCpu,
            chip_name: if cpu.is_empty() {
                gpu.clone()
            } else {
                cpu.to_string()
            },
            gpu_name: (!gpu.is_empty()).then_some(gpu),
            vram_bytes: Some(igpu.dedicated_video_memory),
            shared_gpu_memory_bytes: Some(igpu.shared_system_memory),
        };
    }
    WindowsChipInfo {
        chip_kind: ChipKind::WindowsCpu,
        chip_name: cpu.to_string(),
        gpu_name: None,
        vram_bytes: None,
        shared_gpu_memory_bytes: None,
    }
}

static SPEECH_BACKEND_CHOICE: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// Updates the active Built-in Speech backend preference (`"auto"`, `"cpu"`, or `"VulkanN"`).
pub fn set_speech_backend_choice(choice: &str) {
    let normalized = crate::storage::normalize_builtin_backend(choice);
    *SPEECH_BACKEND_CHOICE
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = normalized;
}

/// Current Built-in Speech backend preference (`"auto"`, `"cpu"`, or `"VulkanN"`).
pub fn speech_backend_choice() -> String {
    let guard = SPEECH_BACKEND_CHOICE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    crate::storage::normalize_builtin_backend(&guard)
}

/// Pure resolver for in-process `whisper.cpp` GPU flags from persisted backend preference and
/// detected hardware capabilities.
pub fn resolve_whisper_gpu_params(
    choice: &str,
    platform_is_macos: bool,
    platform_is_windows: bool,
    has_win_vk_gpu: bool,
    auto_vk_index: Option<i32>,
) -> (bool, i32) {
    let normalized = crate::storage::normalize_builtin_backend(choice);
    if normalized == "cpu" {
        return (false, 0);
    }
    if platform_is_windows {
        if let Some(rest) = normalized
            .strip_prefix("Vulkan")
            .or_else(|| normalized.strip_prefix("vulkan"))
        {
            if let Ok(index) = rest.trim().parse::<i32>() {
                return (true, index);
            }
        }
        return (has_win_vk_gpu, auto_vk_index.unwrap_or(0));
    }
    if platform_is_macos {
        return (true, 0);
    }
    (false, 0)
}

fn current_whisper_gpu_params() -> (bool, i32) {
    let choice = speech_backend_choice();
    #[cfg(target_os = "macos")]
    let params = resolve_whisper_gpu_params(&choice, true, false, false, None);
    #[cfg(target_os = "windows")]
    let params = resolve_whisper_gpu_params(
        &choice,
        false,
        true,
        has_windows_vulkan_gpu(),
        crate::llm::builtin::preferred_windows_vulkan_device_index(),
    );
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let params = resolve_whisper_gpu_params(&choice, false, false, false, None);
    params
}

/// Whether in-process whisper.cpp should use the GPU backend on this platform.
pub fn use_whisper_gpu() -> bool {
    current_whisper_gpu_params().0
}

/// Which GPU device index in-process whisper.cpp should select when `use_whisper_gpu()` is true.
pub fn whisper_gpu_device() -> i32 {
    current_whisper_gpu_params().1
}

/// Reads the chip, memory and free space of the disk that holds `models_dir` (or of its
/// nearest existing parent, when the folder does not exist yet).
pub fn detect(models_dir: &Path) -> Hardware {
    let info = detect_chip_and_memory();
    let mut dir = models_dir;
    while !dir.exists() {
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    let free_bytes = models::disk_available_space(dir).unwrap_or(0);
    Hardware {
        chip_kind: info.chip_kind,
        chip_name: info.chip_name,
        memory_bytes: info.memory_bytes,
        free_bytes,
        gpu_name: info.gpu_name,
        vram_bytes: info.vram_bytes,
        shared_gpu_memory_bytes: info.shared_gpu_memory_bytes,
    }
}

struct DetectedChipAndMemory {
    chip_kind: ChipKind,
    chip_name: String,
    memory_bytes: u64,
    gpu_name: Option<String>,
    vram_bytes: Option<u64>,
    shared_gpu_memory_bytes: Option<u64>,
}

/// macOS: `sysctlbyname` reads kernel values by name, the same ones the `sysctl` command
/// prints:
///
/// - `hw.optional.arm64` is 1 on Apple Silicon. It is 1 even when the app runs under Rosetta,
///   which is what we want: the question is what the Mac can do. Intel Macs do not have it.
/// - `machdep.cpu.brand_string` is the chip's marketing name ("Apple M1 Pro").
/// - `hw.memsize` is the installed memory in bytes (a 64-bit number).
#[cfg(target_os = "macos")]
fn detect_chip_and_memory() -> DetectedChipAndMemory {
    let arm64 = sysctl_u64("hw.optional.arm64").unwrap_or(0) == 1;
    let name = sysctl_string("machdep.cpu.brand_string").unwrap_or_default();
    let chip_kind = if arm64 || name.starts_with("Apple") {
        ChipKind::AppleSilicon
    } else if name.contains("Intel") {
        ChipKind::Intel
    } else {
        ChipKind::Unknown
    };
    DetectedChipAndMemory {
        chip_kind,
        chip_name: name,
        memory_bytes: sysctl_u64("hw.memsize").unwrap_or(0),
        gpu_name: None,
        vram_bytes: None,
        shared_gpu_memory_bytes: None,
    }
}

#[cfg(target_os = "windows")]
fn dxgi_adapters() -> Vec<DxgiAdapter> {
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_REMOTE, DXGI_ADAPTER_FLAG_SOFTWARE,
    };

    let mut adapters = Vec::new();
    // SAFETY: `CreateDXGIFactory1` creates a COM `IDXGIFactory1` interface and manages its
    // reference count via `windows::core::Interface` RAII wrappers.
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        return adapters;
    };
    for index in 0u32..64 {
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(index) }) else {
            break;
        };
        let Ok(desc) = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        let len = desc
            .Description
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(desc.Description.len());
        let name = String::from_utf16_lossy(&desc.Description[..len])
            .trim()
            .to_string();
        let software_flags =
            DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 | DXGI_ADAPTER_FLAG_REMOTE.0 as u32;
        adapters.push(DxgiAdapter {
            name,
            vendor_id: desc.VendorId,
            dedicated_video_memory: desc.DedicatedVideoMemory as u64,
            shared_system_memory: desc.SharedSystemMemory as u64,
            is_software: (desc.Flags & software_flags) != 0,
        });
    }
    adapters
}

/// Cached check for a Vulkan-usable discrete GPU on Windows.
#[cfg(target_os = "windows")]
pub fn has_windows_vulkan_gpu() -> bool {
    static HAS_GPU: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *HAS_GPU.get_or_init(|| dxgi_adapters().iter().any(is_discrete_vulkan_gpu))
}

/// Cached preferred discrete GPU name on Windows (empty on systems with only an iGPU or VM adapter).
#[cfg(target_os = "windows")]
pub fn preferred_windows_gpu_name() -> Option<String> {
    static GPU_NAME: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    GPU_NAME
        .get_or_init(|| {
            let selected = select_windows_gpu(&dxgi_adapters(), "");
            (selected.chip_kind == ChipKind::WindowsGpu)
                .then_some(selected.gpu_name)
                .flatten()
        })
        .clone()
}

#[cfg(target_os = "windows")]
fn detect_chip_and_memory() -> DetectedChipAndMemory {
    use windows_sys::Win32::System::Registry::HKEY_LOCAL_MACHINE;
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    let memory_bytes = if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
        status.ullTotalPhys
    } else {
        0
    };

    let cpu_name = crate::platform::read_reg_sz(
        HKEY_LOCAL_MACHINE,
        r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
        "ProcessorNameString",
    )
    .unwrap_or_default();

    let adapters = dxgi_adapters();
    let selected = select_windows_gpu(&adapters, &cpu_name);
    DetectedChipAndMemory {
        chip_kind: selected.chip_kind,
        chip_name: selected.chip_name,
        memory_bytes,
        gpu_name: selected.gpu_name,
        vram_bytes: selected.vram_bytes,
        shared_gpu_memory_bytes: selected.shared_gpu_memory_bytes,
    }
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn detect_chip_and_memory() -> DetectedChipAndMemory {
    DetectedChipAndMemory {
        chip_kind: ChipKind::Unknown,
        chip_name: String::new(),
        memory_bytes: 0,
        gpu_name: None,
        vram_bytes: None,
        shared_gpu_memory_bytes: None,
    }
}

/// Reads a numeric sysctl value. The kernel writes 4 bytes for `int` values and 8 for 64-bit
/// ones; the size it reports tells which.
#[cfg(target_os = "macos")]
fn sysctl_u64(name: &str) -> Option<u64> {
    let name = std::ffi::CString::new(name).ok()?;
    let mut value: u64 = 0;
    let mut size = std::mem::size_of::<u64>();
    // SAFETY: `value` is a valid 8-byte buffer and `size` tells the kernel its length; the
    // kernel writes at most `size` bytes and stores the written length back in `size`.
    let result = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&mut value as *mut u64).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    match (result, size) {
        (0, 8) => Some(value),
        // A 4-byte int lands in the low bytes (Apple Silicon and Intel Macs are little-endian).
        (0, 4) => Some(value & 0xffff_ffff),
        _ => None,
    }
}

/// Reads a text sysctl value: first asks for its length, then reads it.
#[cfg(target_os = "macos")]
fn sysctl_string(name: &str) -> Option<String> {
    let name = std::ffi::CString::new(name).ok()?;
    let mut size = 0usize;
    // SAFETY: a null buffer asks only for the length, which the kernel stores in `size`.
    let result = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if result != 0 || size == 0 {
        return None;
    }
    let mut buffer = vec![0u8; size];
    // SAFETY: `buffer` holds `size` bytes, as the kernel asked for.
    let result = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            buffer.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if result != 0 {
        return None;
    }
    buffer.truncate(size);
    let text = String::from_utf8_lossy(&buffer);
    Some(text.trim_end_matches('\0').trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1_000_000;
    const MIB: u64 = 1024 * 1024;

    fn mac(chip_kind: ChipKind, memory_gb: u64, free_bytes: u64) -> Hardware {
        Hardware {
            chip_kind,
            chip_name: String::new(),
            memory_bytes: memory_gb * GB,
            free_bytes,
            gpu_name: None,
            vram_bytes: None,
            shared_gpu_memory_bytes: None,
        }
    }

    fn windows_gpu(gpu_name: &str, vram_mib: u64, ram_gb: u64, free_bytes: u64) -> Hardware {
        Hardware {
            chip_kind: ChipKind::WindowsGpu,
            chip_name: gpu_name.to_string(),
            memory_bytes: ram_gb * GB,
            free_bytes,
            gpu_name: Some(gpu_name.to_string()),
            vram_bytes: Some(vram_mib * MIB),
            shared_gpu_memory_bytes: Some(8 * GB),
        }
    }

    fn ids(offer: &ModelOffer) -> Vec<&str> {
        offer.models.iter().map(|model| model.id.as_str()).collect()
    }

    #[test]
    fn apple_silicon_with_memory_and_disk_gets_both_best_first() {
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 32, 55_000 * MB), &[]);
        assert_eq!(
            ids(&offer),
            [
                "large-v3-turbo",
                "small",
                "qwen3-asr-1.7b",
                "qwen3-asr-0.6b"
            ]
        );
        assert!(offer.models[0].recommended);
        assert!(!offer.models[1].recommended);
        assert!(!offer.models[2].recommended);
        assert!(!offer.models[3].recommended);
        assert_eq!(offer.left_out, None);
        assert_eq!(offer.needed_bytes, None);
        // Exactly 8 GB is enough.
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 8, 20_000 * MB), &[]);
        assert_eq!(
            ids(&offer),
            [
                "large-v3-turbo",
                "small",
                "qwen3-asr-1.7b",
                "qwen3-asr-0.6b"
            ]
        );
    }

    #[test]
    fn qwen3_asr_needs_apple_silicon_and_room_for_both_files() {
        let qwen = models::asr_model(models::QWEN3_ASR_MODEL_ID).unwrap();
        let offer = offer_models(&mac(ChipKind::Intel, 64, 90_000 * MB), &[]);
        assert!(!ids(&offer).contains(&"qwen3-asr-1.7b"));
        let short = models::needed_free_bytes(qwen.size_bytes(), 0) - 1;
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 32, short), &[]);
        // Plan `small-cantonese-asr`: the 0.6B model still fits where the 1.7B does not.
        assert_eq!(ids(&offer), ["large-v3-turbo", "small", "qwen3-asr-0.6b"]);
        let offer = offer_models(
            &mac(ChipKind::AppleSilicon, 32, 0),
            &["qwen3-asr-1.7b".to_string()],
        );
        assert_eq!(ids(&offer), ["qwen3-asr-1.7b"]);
        assert_eq!(offer.models[0].size_bytes, qwen.size_bytes());
    }

    #[test]
    fn small_memory_or_intel_gets_only_the_faster_model() {
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 4, 20_000 * MB), &[]);
        assert_eq!(ids(&offer), ["small"]);
        assert!(!offer.models[0].recommended);
        assert_eq!(offer.left_out, Some(LeftOutReason::NeedsMemory));

        let offer = offer_models(&mac(ChipKind::Intel, 16, 90_000 * MB), &[]);
        assert_eq!(ids(&offer), ["small"]);
        assert_eq!(offer.left_out, Some(LeftOutReason::NeedsAppleSilicon));

        let offer = offer_models(&mac(ChipKind::Unknown, 64, 90_000 * MB), &[]);
        assert_eq!(ids(&offer), ["small"]);
        assert_eq!(offer.left_out, Some(LeftOutReason::NeedsAppleSilicon));
    }

    #[test]
    fn low_disk_leaves_out_the_larger_model() {
        let large = models::known_model(BEST_MODEL_ID).unwrap().size_bytes;
        let small = models::known_model(FASTER_MODEL_ID).unwrap().size_bytes;
        let just_short = models::needed_free_bytes(large, 0) - 1;
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 16, just_short), &[]);
        assert_eq!(ids(&offer), ["small"]);
        assert_eq!(offer.left_out, Some(LeftOutReason::NeedsDiskSpace));

        // 400 MB free: the mock's "low disk" Mac.
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 16, 400 * MB), &[]);
        assert_eq!(ids(&offer), ["small"]);

        let exact = models::needed_free_bytes(small, 0);
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 16, exact), &[]);
        assert_eq!(ids(&offer), ["small"]);
    }

    #[test]
    fn not_enough_disk_for_either_says_how_much_is_needed() {
        let small = models::known_model(FASTER_MODEL_ID).unwrap().size_bytes;
        let offer = offer_models(&mac(ChipKind::AppleSilicon, 32, 100 * MB), &[]);
        assert!(offer.models.is_empty());
        assert_eq!(offer.left_out, None);
        assert_eq!(
            offer.needed_bytes,
            Some(models::needed_free_bytes(small, 0))
        );
    }

    #[test]
    fn an_installed_model_needs_no_free_space() {
        let offer = offer_models(
            &mac(ChipKind::AppleSilicon, 32, 0),
            &["large-v3-turbo".to_string()],
        );
        assert_eq!(ids(&offer), ["large-v3-turbo"]);
        assert_eq!(offer.left_out, None);
        // Installed does not bypass the chip rule.
        let offer = offer_models(
            &mac(ChipKind::Intel, 32, 0),
            &["large-v3-turbo".to_string(), "small".to_string()],
        );
        assert_eq!(ids(&offer), ["small"]);
    }

    #[test]
    fn the_offer_serialises_for_the_frontend() {
        let check = HardwareCheck {
            hardware: Hardware {
                chip_kind: ChipKind::AppleSilicon,
                chip_name: "Apple M1 Pro".into(),
                memory_bytes: 32 * GB,
                free_bytes: 5,
                gpu_name: None,
                vram_bytes: None,
                shared_gpu_memory_bytes: None,
            },
            offer: offer_models(&mac(ChipKind::Intel, 16, 90_000 * MB), &[]),
        };
        assert_eq!(
            serde_json::to_value(&check).unwrap(),
            serde_json::json!({
                "hardware": {
                    "chipKind": "apple_silicon",
                    "chipName": "Apple M1 Pro",
                    "memoryBytes": 32 * GB,
                    "freeBytes": 5,
                },
                "offer": {
                    "models": [{"id": "small", "sizeBytes": 190_085_487, "recommended": false}],
                    "leftOut": "needs_apple_silicon",
                    "neededBytes": null,
                },
            })
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn detect_reads_this_mac() {
        let hardware = detect(&std::env::temp_dir().join("oratype-no-such-folder/models"));
        assert_ne!(hardware.chip_kind, ChipKind::Unknown);
        assert!(!hardware.chip_name.is_empty());
        assert!(hardware.memory_bytes >= GB);
        assert!(hardware.free_bytes > 0);
    }

    #[test]
    fn hybrid_laptop_picks_discrete_gpu_over_integrated_and_ignores_software() {
        let adapters = [
            DxgiAdapter {
                name: "Intel(R) Iris(R) Xe Graphics".into(),
                vendor_id: 0x8086,
                dedicated_video_memory: 128 * MIB,
                shared_system_memory: 8 * GB,
                is_software: false,
            },
            DxgiAdapter {
                name: "Microsoft Basic Render Driver".into(),
                vendor_id: 0x1414,
                dedicated_video_memory: 0,
                shared_system_memory: 8 * GB,
                is_software: true,
            },
            DxgiAdapter {
                name: "NVIDIA GeForce RTX 4060".into(),
                vendor_id: 0x10DE,
                dedicated_video_memory: 7900 * MIB,
                shared_system_memory: 8 * GB,
                is_software: false,
            },
        ];
        let picked = select_windows_gpu(&adapters, "13th Gen Intel Core i7");
        assert_eq!(picked.chip_kind, ChipKind::WindowsGpu);
        assert_eq!(picked.chip_name, "NVIDIA GeForce RTX 4060");
        assert_eq!(picked.gpu_name.as_deref(), Some("NVIDIA GeForce RTX 4060"));
        assert_eq!(picked.vram_bytes, Some(7900 * MIB));
        assert_eq!(picked.shared_gpu_memory_bytes, Some(8 * GB));

        // Software-only VM falls back to WindowsCpu with no gpu_name.
        let vm = select_windows_gpu(&adapters[1..2], "AMD EPYC 7763");
        assert_eq!(vm.chip_kind, ChipKind::WindowsCpu);
        assert_eq!(vm.chip_name, "AMD EPYC 7763");
        assert_eq!(vm.gpu_name, None);
        assert_eq!(vm.vram_bytes, None);

        // Integrated-only laptop keeps CPU name as chipName while recording iGPU metadata.
        let igpu = select_windows_gpu(&adapters[0..1], "13th Gen Intel Core i7");
        assert_eq!(igpu.chip_kind, ChipKind::WindowsCpu);
        assert_eq!(igpu.chip_name, "13th Gen Intel Core i7");
        assert_eq!(
            igpu.gpu_name.as_deref(),
            Some("Intel(R) Iris(R) Xe Graphics")
        );
        assert_eq!(igpu.vram_bytes, Some(128 * MIB));
    }

    #[test]
    fn windows_speech_offers_follow_gpu_and_vram_tiers() {
        // 8 GB / 6 GB GPU: gets all Whisper + Qwen3-ASR models.
        let offer = offer_models(
            &windows_gpu("NVIDIA GeForce RTX 3060 Laptop GPU", 5850, 16, 50_000 * MB),
            &[],
        );
        assert_eq!(
            ids(&offer),
            [
                "large-v3-turbo",
                "small",
                "qwen3-asr-1.7b",
                "qwen3-asr-0.6b"
            ]
        );
        assert!(offer.models[0].recommended);
        assert_eq!(offer.left_out, None);

        // 4 GB discrete GPU (3850 MiB reported by DXGI): gets small Qwen3-ASR 0.6B but skips 1.7B.
        let offer = offer_models(
            &windows_gpu("NVIDIA GeForce RTX 3050", 3850, 16, 50_000 * MB),
            &[],
        );
        assert_eq!(ids(&offer), ["large-v3-turbo", "small", "qwen3-asr-0.6b"]);
        assert_eq!(offer.left_out, None);

        // 2 GB discrete GPU: Whisper small only, explains VRAM requirement.
        let offer = offer_models(
            &windows_gpu("NVIDIA GeForce MX450", 1950, 16, 50_000 * MB),
            &[],
        );
        assert_eq!(ids(&offer), ["small"]);
        assert_eq!(offer.left_out, Some(LeftOutReason::NeedsVram));

        // Integrated graphics / no GPU: Whisper small only, explains GPU requirement.
        let offer = offer_models(&mac(ChipKind::WindowsCpu, 32, 50_000 * MB), &[]);
        assert_eq!(ids(&offer), ["small"]);
        assert_eq!(offer.left_out, Some(LeftOutReason::NeedsGpu));
    }

    #[test]
    fn windows_hardware_serialises_gpu_and_vram_fields() {
        let hw = windows_gpu("NVIDIA GeForce RTX 4060", 7900, 16, 50_000 * MB);
        let value = serde_json::to_value(&hw).unwrap();
        assert_eq!(value["chipKind"], "windows_gpu");
        assert_eq!(value["gpuName"], "NVIDIA GeForce RTX 4060");
        assert_eq!(value["vramBytes"], 7900 * MIB);
        assert_eq!(value["sharedGpuMemoryBytes"], 8 * GB);
    }

    #[test]
    fn virtual_display_first_with_amd_igpu_and_rtx_4090_picks_rtx_4090() {
        let adapters = [
            DxgiAdapter {
                name: "Parsec Virtual Display Adapter".into(),
                vendor_id: 0x10DE,
                dedicated_video_memory: 0,
                shared_system_memory: 47_500 * MIB,
                is_software: false,
            },
            DxgiAdapter {
                name: "SudoMaker Virtual Display Adapter".into(),
                vendor_id: 0x10DE,
                dedicated_video_memory: 24_142 * MIB,
                shared_system_memory: 47_500 * MIB,
                is_software: false,
            },
            DxgiAdapter {
                name: "AMD Radeon(TM) Graphics".into(),
                vendor_id: 0x1002,
                dedicated_video_memory: 512 * MIB,
                shared_system_memory: 47_500 * MIB,
                is_software: false,
            },
            DxgiAdapter {
                name: "NVIDIA GeForce RTX 4090".into(),
                vendor_id: 0x10DE,
                dedicated_video_memory: 24_142 * MIB,
                shared_system_memory: 47_500 * MIB,
                is_software: false,
            },
            DxgiAdapter {
                name: "Microsoft Basic Render Driver".into(),
                vendor_id: 0x1414,
                dedicated_video_memory: 0,
                shared_system_memory: 47_500 * MIB,
                is_software: true,
            },
        ];
        assert!(is_software_adapter(&adapters[0]));
        assert!(is_software_adapter(&adapters[1]));
        assert!(!is_software_adapter(&adapters[2]));
        assert!(!is_software_adapter(&adapters[3]));
        assert!(is_software_adapter(&adapters[4]));

        let picked = select_windows_gpu(&adapters, "AMD Ryzen 9 7950X3D 16-Core Processor");
        assert_eq!(picked.chip_kind, ChipKind::WindowsGpu);
        assert_eq!(picked.chip_name, "NVIDIA GeForce RTX 4090");
        assert_eq!(picked.gpu_name.as_deref(), Some("NVIDIA GeForce RTX 4090"));
        assert_eq!(picked.vram_bytes, Some(24_142 * MIB));
    }

    #[test]
    fn resolve_whisper_gpu_params_respects_auto_cpu_and_explicit_vulkan_index() {
        assert_eq!(
            resolve_whisper_gpu_params("auto", true, false, false, None),
            (true, 0)
        );
        assert_eq!(
            resolve_whisper_gpu_params("cpu", true, false, false, None),
            (false, 0)
        );
        assert_eq!(
            resolve_whisper_gpu_params("auto", false, true, true, Some(1)),
            (true, 1)
        );
        assert_eq!(
            resolve_whisper_gpu_params("cpu", false, true, true, Some(1)),
            (false, 0)
        );
        assert_eq!(
            resolve_whisper_gpu_params("Vulkan0", false, true, true, Some(1)),
            (true, 0)
        );
        assert_eq!(
            resolve_whisper_gpu_params("auto", false, true, false, None),
            (false, 0)
        );
    }
}
